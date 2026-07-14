//! FFI 接口（C ABI）。
//!
//! 该模块定义面向 C/移动端的 C-friendly 函数签名，供桌面（C#/.NET、Qt）、
//! 移动（Kotlin/JNI、Swift/NAPI）等宿主通过动态库（cdylib）调用核心能力。
//!
//! ## 设计约定
//! - 凡返回 `*mut c_char` 的函数，调用方必须使用 [`music_core_free_string`] 释放。
//! - 传入核心的指针所有权不转移，核心不负责释放外部内存。
//! - 所有 FFI 入口均为 `extern "C"`，并以 `#[no_mangle]` 导出稳定符号。
//! - 异步核心能力通过内部全局 `tokio::runtime::Runtime` 的 `block_on` 驱动，
//!   对 C 侧表现为同步调用。核心状态（SourceManager / FeiniuClient /
//!   协议源表 / 本地音源 / 缓存）以全局单例持有。
//!
//! ## Safety
//! 任何解引用原始指针的入口均标记为 `pub unsafe extern "C" fn`，调用方须保证：
//! - 传入的 `*const c_char` 为 NULL 或指向合法 NUL 结尾的 UTF-8（或至少 ASCII）C 字符串；
//! - 返回的 `*mut c_char` 仅可由 [`music_core_free_string`] 释放，不可用其它分配器释放。
//! 跨 FFI 边界的 panic 是未定义行为，故所有入口均不 panic：失败时返回 JSON 错误串或空指针。

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Arc, Mutex, OnceLock};

use serde::Serialize;

use crate::cache::{CacheManager, CacheStats};
use crate::error::{CoreError, Result};
use crate::feiniu::{FeiniuClient, NasFile};
use crate::models::{Leaderboard, Lyric, SearchResult, Song};
use crate::protocols::ProtocolClient;
use crate::sources::local::LocalSource;
use crate::sources::{Source, SourceInfo, SourceManager};

use base64::{engine::general_purpose, Engine as _};

// =====================================================================
// 全局状态
// =====================================================================

/// 已注册协议源条目：客户端 + 展示信息。
struct ProtocolEntry {
    client: Arc<dyn ProtocolClient>,
    protocol: String,
    root: String,
}

/// FFI 全局状态：tokio 运行时 + 各核心组件。
///
/// 使用 `OnceLock<Option<FfiState>>`：运行时创建失败时存 `None`，后续调用返回
/// FFI 错误而非 panic。所有互斥量在锁中毒化时返回错误而非 abort。
struct FfiState {
    rt: tokio::runtime::Runtime,
    sm: Mutex<SourceManager>,
    feiniu: Mutex<FeiniuClient>,
    protocols: Mutex<HashMap<String, ProtocolEntry>>,
    protocol_seq: Mutex<u64>,
    local: Mutex<Option<LocalSource>>,
    cache: Mutex<Option<CacheManager>>,
}

static STATE: OnceLock<Option<FfiState>> = OnceLock::new();

/// 获取全局状态；运行时初始化失败时返回 `None`（不 panic）。
fn state() -> Option<&'static FfiState> {
    STATE
        .get_or_init(|| {
            // 多线程运行时：驱动 reqwest 等异步核心能力。
            // `new` 失败极少见（如系统资源耗尽），失败时返回 None 而非 panic。
            let rt = tokio::runtime::Runtime::new().ok()?;
            Some(FfiState {
                rt,
                sm: Mutex::new(SourceManager::new()),
                feiniu: Mutex::new(FeiniuClient::new(String::new())),
                protocols: Mutex::new(HashMap::new()),
                protocol_seq: Mutex::new(1),
                local: Mutex::new(None),
                cache: Mutex::new(None),
            })
        })
        .as_ref()
}

// =====================================================================
// 辅助函数（私有，不导出 C 符号）
// =====================================================================

/// 将 `*const c_char` 转为拥有的 `String`；NULL 或非法 UTF-8 返回 `None`。
///
/// # Safety
/// `ptr` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串（由调用方保证）。
unsafe fn cstr_to_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    // 安全：调用方保证 ptr 指向合法 NUL 结尾的 C 字符串
    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(String::from)
}

/// 将字符串构造为 `CString` 并交出所有权（`into_raw`）。
/// 内嵌 NUL 会被移除以避免构造失败（FFI panic 规避）。
fn to_raw(s: &str) -> *mut c_char {
    let cleaned: String = s.replace('\0', "");
    CString::new(cleaned).unwrap_or_default().into_raw()
}

/// 将可序列化值序列化为 JSON 并交出所有权指针；失败返回空指针。
fn to_json_ptr<T: Serialize>(val: &T) -> *mut c_char {
    match serde_json::to_string(val) {
        Ok(s) => to_raw(&s),
        Err(_) => std::ptr::null_mut(),
    }
}

/// 构造统一错误 JSON：`{"error":{"kind":"...","message":"..."}}`，交出所有权。
fn error_ptr(kind: &str, msg: &str) -> *mut c_char {
    let json = serde_json::json!({ "error": { "kind": kind, "message": msg } });
    to_json_ptr(&json)
}

/// 将 `CoreError` 映射为 `kind` 字符串（与 `error.rs` 变体名一致）。
fn error_kind(e: &CoreError) -> &'static str {
    match e {
        CoreError::Io(_) => "Io",
        CoreError::Json(_) => "Json",
        CoreError::Http(_) => "Http",
        CoreError::Source(_) => "Source",
        CoreError::Schema(_) => "Schema",
        CoreError::NotFound(_) => "NotFound",
        CoreError::Cache(_) => "Cache",
        CoreError::Protocol(_) => "Protocol",
        CoreError::Feiniu(_) => "Feiniu",
        CoreError::Ffi(_) => "Ffi",
    }
}

/// 将 `Result<T>` 转为指针：成功序列化值，失败返回错误 JSON。
fn result_ptr<T: Serialize>(res: Result<T>) -> *mut c_char {
    match res {
        Ok(v) => to_json_ptr(&v),
        Err(e) => {
            let kind = error_kind(&e);
            error_ptr(kind, &e.to_string())
        }
    }
}

/// 阻塞驱动异步 future 至完成（在全局 runtime 上）。
fn block_on<F: std::future::Future>(st: &FfiState, fut: F) -> F::Output {
    st.rt.block_on(fut)
}

// =====================================================================
// 版本与内存释放
// =====================================================================

/// 返回核心库版本字符串。
///
/// 调用方负责用 [`music_core_free_string`] 释放返回的指针。
///
/// 该函数不会 panic：版本号取自 `CARGO_PKG_VERSION`，由 Cargo 在编译期校验为
/// 合法 semver（不含 NUL）。即便极端情况下版本号含 NUL，也会回退为空 C 字符串
/// 而非触发 panic（panic 跨 FFI 边界是未定义行为）。
#[no_mangle]
pub extern "C" fn music_core_version() -> *mut c_char {
    let version = env!("CARGO_PKG_VERSION");
    // Cargo 保证 version 为合法 semver，不含 NUL；防御性回退为空串以避免 FFI panic
    let c_string = CString::new(version).unwrap_or_default();
    // 将所有权转移给调用方
    c_string.into_raw()
}

/// 释放由 `music_core_*` 系列函数返回的 C 字符串。
///
/// 传入 NULL 是安全空操作。调用方不应再使用已释放的指针。
///
/// # Safety
/// `ptr` 必须为 NULL 或由 `CString::into_raw` 产生（即本模块 `music_core_*`
/// 返回的指针）。传入任何其他指针（包括由 `malloc`、栈地址或其它分配器产生的指针）
/// 均会导致未定义行为。函数被标记为 `unsafe` 是为了在 Rust 侧强制调用方显式承诺
/// 该前置条件；C 侧调用本身不需额外的 unsafe 语法（C 无此概念），但语义上等价。
#[no_mangle]
pub unsafe extern "C" fn music_core_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    // 安全：调用方约定 ptr 来自本模块的 into_raw，重新构造并丢弃以回收内存
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}

// =====================================================================
// 音源管理
// =====================================================================

/// 导入音源 JSON（社区格式自动适配为标准配置），返回导入后的 `SourceInfo` JSON。
/// 失败返回 `{"error":{"kind":...,"message":...}}`。
///
/// # Safety
/// `json` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。返回的指针须由
/// [`music_core_free_string`] 释放。
#[no_mangle]
pub unsafe extern "C" fn music_core_source_import(json: *const c_char) -> *mut c_char {
    let json_str = match unsafe { cstr_to_string(json) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "json 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let mut sm = match st.sm.lock() {
        Ok(g) => g,
        Err(_) => return error_ptr("Ffi", "音源管理器锁获取失败"),
    };
    result_ptr(sm.import_source_from_json(&json_str))
}

/// 校验音源 JSON 是否符合标准 Schema（不加载、不持久化）。
/// 返回 `{"valid":bool,"errors":[...]}` JSON。
///
/// # Safety
/// `json` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。返回的指针须由
/// [`music_core_free_string`] 释放。
#[no_mangle]
pub unsafe extern "C" fn music_core_source_validate(json: *const c_char) -> *mut c_char {
    let json_str = match unsafe { cstr_to_string(json) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "json 参数为空指针"),
    };
    if state().is_none() {
        return error_ptr("Ffi", "核心运行时未初始化");
    }

    let value: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => {
            // JSON 解析失败应返回校验失败格式，而非错误 JSON，
            // 与函数文档约定（{"valid":bool,"errors":[...]}）一致。
            return to_json_ptr(&serde_json::json!({
                "valid": false,
                "errors": [format!("JSON 解析失败: {}", e)],
            }));
        }
    };

    use crate::sources::schema::{validate_config, SoundSourceConfig};
    // 先做严格 schema 校验，再做语义校验；收集所有错误信息。
    if let Err(e) = SoundSourceConfig::validate_strict(&value) {
        return to_json_ptr(&serde_json::json!({
            "valid": false,
            "errors": [e.to_string()],
        }));
    }
    let config = match SoundSourceConfig::from_json(&json_str) {
        Ok(c) => c,
        Err(e) => {
            return to_json_ptr(&serde_json::json!({
                "valid": false,
                "errors": [e.to_string()],
            }));
        }
    };
    match validate_config(&config) {
        Ok(()) => to_json_ptr(&serde_json::json!({ "valid": true, "errors": [] })),
        Err(e) => to_json_ptr(&serde_json::json!({
            "valid": false,
            "errors": [e.to_string()],
        })),
    }
}

/// 列出所有音源（按 priority 降序），返回 `Vec<SourceInfo>` JSON 数组。
#[no_mangle]
pub extern "C" fn music_core_source_list() -> *mut c_char {
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let sm = match st.sm.lock() {
        Ok(g) => g,
        Err(_) => return error_ptr("Ffi", "音源管理器锁获取失败"),
    };
    let list: Vec<SourceInfo> = sm.list_sources_ordered();
    to_json_ptr(&list)
}

/// 启用指定 id 的音源，返回 `{"id":...,"enabled":true}`。
///
/// # Safety
/// `id` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_source_enable(id: *const c_char) -> *mut c_char {
    set_source_enabled(id, true)
}

/// 禁用指定 id 的音源，返回 `{"id":...,"enabled":false}`。
///
/// # Safety
/// `id` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_source_disable(id: *const c_char) -> *mut c_char {
    set_source_enabled(id, false)
}

/// 设置指定 id 音源的启用状态，返回 `{"id":...,"enabled":bool}`。
///
/// # Safety
/// `id` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串（由调用方保证）。
unsafe fn set_source_enabled(id: *const c_char, enabled: bool) -> *mut c_char {
    // 安全：调用方保证 id 为 NULL 或合法 C 字符串
    let id_str = match cstr_to_string(id) {
        Some(s) => s,
        None => return error_ptr("Ffi", "id 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let mut sm = match st.sm.lock() {
        Ok(g) => g,
        Err(_) => return error_ptr("Ffi", "音源管理器锁获取失败"),
    };
    match sm.set_source_enabled(&id_str, enabled) {
        Ok(()) => to_json_ptr(&serde_json::json!({ "id": id_str, "enabled": enabled })),
        Err(e) => error_ptr(error_kind(&e), &e.to_string()),
    }
}

/// 删除指定 id 的音源，返回 `{"id":...,"deleted":true}`。
///
/// # Safety
/// `id` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_source_delete(id: *const c_char) -> *mut c_char {
    let id_str = match unsafe { cstr_to_string(id) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "id 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let mut sm = match st.sm.lock() {
        Ok(g) => g,
        Err(_) => return error_ptr("Ffi", "音源管理器锁获取失败"),
    };
    match sm.delete_source(&id_str) {
        Ok(()) => to_json_ptr(&serde_json::json!({ "id": id_str, "deleted": true })),
        Err(e) => error_ptr(error_kind(&e), &e.to_string()),
    }
}

// =====================================================================
// 搜索与歌曲（异步，经全局 runtime block_on 驱动）
// =====================================================================

/// 聚合搜索：跨所有启用音源搜索，单个音源失败记录警告并跳过。
/// 返回 `SearchResult` JSON。
///
/// # Safety
/// `keyword` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_search(
    keyword: *const c_char,
    page: u32,
    page_size: u32,
) -> *mut c_char {
    let kw = match unsafe { cstr_to_string(keyword) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "keyword 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    // 先在锁内克隆各启用音源的 Arc，立即释放锁，避免持有锁跨越 await。
    let sources: Vec<Arc<dyn Source>> = {
        let sm = match st.sm.lock() {
            Ok(g) => g,
            Err(_) => return error_ptr("Ffi", "音源管理器锁获取失败"),
        };
        sm.ordered_sources().iter().map(Arc::clone).collect()
    };

    let result: Result<SearchResult> = block_on(st, async move {
        let mut songs = Vec::new();
        let mut albums = Vec::new();
        let mut artists = Vec::new();
        let mut total: u64 = 0;
        for source in &sources {
            match source.search(&kw, page, page_size).await {
                Ok(r) => {
                    songs.extend(r.songs);
                    albums.extend(r.albums);
                    artists.extend(r.artists);
                    total = total.saturating_add(r.total);
                }
                Err(e) => {
                    log::warn!("音源 {} 搜索失败，已跳过: {}", source.id(), e);
                }
            }
        }
        Ok(SearchResult {
            keyword: kw,
            songs,
            albums,
            artists,
            total,
            page,
            page_size,
        })
    });
    result_ptr(result)
}

/// 获取指定音源下歌曲的完整元数据，返回 `Song` JSON。
///
/// # Safety
/// `source_id` / `song_id` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_get_metadata(
    source_id: *const c_char,
    song_id: *const c_char,
) -> *mut c_char {
    let (sid, songid) = match (unsafe { cstr_to_string(source_id) }, unsafe { cstr_to_string(song_id) })
    {
        (Some(a), Some(b)) => (a, b),
        _ => return error_ptr("Ffi", "source_id / song_id 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let source: Arc<dyn Source> = {
        let sm = match st.sm.lock() {
            Ok(g) => g,
            Err(_) => return error_ptr("Ffi", "音源管理器锁获取失败"),
        };
        match sm.get_source(&sid) {
            Some(s) => s,
            None => return error_ptr("NotFound", &format!("音源不存在: {sid}")),
        }
    };
    let result: Result<Song> = block_on(st, async move { source.get_metadata(&songid).await });
    result_ptr(result)
}

/// 获取指定音源下歌曲的可播放 URL，返回 `{"url":...,"cached":false}`。
///
/// # Safety
/// `source_id` / `song_id` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_get_play_url(
    source_id: *const c_char,
    song_id: *const c_char,
) -> *mut c_char {
    let (sid, songid) = match (unsafe { cstr_to_string(source_id) }, unsafe { cstr_to_string(song_id) })
    {
        (Some(a), Some(b)) => (a, b),
        _ => return error_ptr("Ffi", "source_id / song_id 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let source: Arc<dyn Source> = {
        let sm = match st.sm.lock() {
            Ok(g) => g,
            Err(_) => return error_ptr("Ffi", "音源管理器锁获取失败"),
        };
        match sm.get_source(&sid) {
            Some(s) => s,
            None => return error_ptr("NotFound", &format!("音源不存在: {sid}")),
        }
    };
    let result: Result<String> = block_on(st, async move { source.get_play_url(&songid).await });
    match result {
        Ok(url) => to_json_ptr(&serde_json::json!({ "url": url, "cached": false, "play_url": url })),
        Err(e) => error_ptr(error_kind(&e), &e.to_string()),
    }
}

/// 获取指定音源下歌曲的歌词，返回 `Lyric` JSON。
///
/// # Safety
/// `source_id` / `song_id` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_get_lyric(
    source_id: *const c_char,
    song_id: *const c_char,
) -> *mut c_char {
    let (sid, songid) = match (unsafe { cstr_to_string(source_id) }, unsafe { cstr_to_string(song_id) })
    {
        (Some(a), Some(b)) => (a, b),
        _ => return error_ptr("Ffi", "source_id / song_id 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let source: Arc<dyn Source> = {
        let sm = match st.sm.lock() {
            Ok(g) => g,
            Err(_) => return error_ptr("Ffi", "音源管理器锁获取失败"),
        };
        match sm.get_source(&sid) {
            Some(s) => s,
            None => return error_ptr("NotFound", &format!("音源不存在: {sid}")),
        }
    };
    let result: Result<Lyric> = block_on(st, async move { source.get_lyric(&songid).await });
    result_ptr(result)
}

/// 获取指定音源的排行榜列表，返回 `Vec<Leaderboard>` JSON 数组。
///
/// # Safety
/// `source_id` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_get_leaderboards(
    source_id: *const c_char,
) -> *mut c_char {
    let sid = match unsafe { cstr_to_string(source_id) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "source_id 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let source: Arc<dyn Source> = {
        let sm = match st.sm.lock() {
            Ok(g) => g,
            Err(_) => return error_ptr("Ffi", "音源管理器锁获取失败"),
        };
        match sm.get_source(&sid) {
            Some(s) => s,
            None => return error_ptr("NotFound", &format!("音源不存在: {sid}")),
        }
    };
    let result: Result<Vec<Leaderboard>> =
        block_on(st, async move { source.get_leaderboards().await });
    result_ptr(result)
}

// =====================================================================
// 飞牛 NAS
// =====================================================================

/// 登录飞牛 NAS，返回 `{"token":...,"base_url":...}`。
/// 内部创建独立 `FeiniuClient` 完成登录后写回全局状态，避免持有锁跨越 await。
///
/// # Safety
/// `base_url` / `username` / `password` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_feiniu_login(
    base_url: *const c_char,
    username: *const c_char,
    password: *const c_char,
) -> *mut c_char {
    let (base, user, pass) = match (
        unsafe { cstr_to_string(base_url) },
        unsafe { cstr_to_string(username) },
        unsafe { cstr_to_string(password) },
    ) {
        (Some(a), Some(b), Some(c)) => (a, b, c),
        _ => return error_ptr("Ffi", "base_url/username/password 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };

    // 创建独立 owned 客户端完成登录，避免持有全局锁跨越网络 await。
    let mut client = FeiniuClient::new(base.clone());
    let login_result: Result<()> = block_on(st, async {
        client.login(&user, &pass).await
    });
    if let Err(e) = login_result {
        return error_ptr(error_kind(&e), &e.to_string());
    }
    let token = client.token.clone().unwrap_or_default();
    // 写回全局状态
    if let Ok(mut g) = st.feiniu.lock() {
        *g = client;
    }
    to_json_ptr(&serde_json::json!({ "token": token, "base_url": base }))
}

/// 列出飞牛 NAS 指定路径下的文件，返回 `{"path":...,"files":[NasFile...]}`。
///
/// # Safety
/// `path` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_feiniu_list_files(path: *const c_char) -> *mut c_char {
    let p = match unsafe { cstr_to_string(path) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "path 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    // 克隆客户端快照（base_url/token/reqwest 客户端均 Clone），释放锁后再异步。
    let snap = match st.feiniu.lock() {
        Ok(g) => FeiniuClient {
            base_url: g.base_url.clone(),
            client: g.client.clone(),
            token: g.token.clone(),
        },
        Err(_) => return error_ptr("Ffi", "飞牛客户端锁获取失败"),
    };
    // 克隆一份供后续 JSON 拼装使用（原始 p 被 async move 闭包捕获）
    let p_for_json = p.clone();
    let files: Result<Vec<NasFile>> = block_on(st, async move { snap.list_files(&p).await });
    match files {
        Ok(files) => to_json_ptr(&serde_json::json!({ "path": p_for_json, "files": files })),
        Err(e) => error_ptr(error_kind(&e), &e.to_string()),
    }
}

/// 生成飞牛 NAS 文件的可流式播放 URL，返回 `{"url":...}`。
///
/// # Safety
/// `path` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_feiniu_stream(path: *const c_char) -> *mut c_char {
    let p = match unsafe { cstr_to_string(path) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "path 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let snap = match st.feiniu.lock() {
        Ok(g) => FeiniuClient {
            base_url: g.base_url.clone(),
            client: g.client.clone(),
            token: g.token.clone(),
        },
        Err(_) => return error_ptr("Ffi", "飞牛客户端锁获取失败"),
    };
    let url: Result<String> = block_on(st, async move { snap.get_stream_url(&p).await });
    match url {
        Ok(u) => to_json_ptr(&serde_json::json!({ "url": u })),
        Err(e) => error_ptr(error_kind(&e), &e.to_string()),
    }
}

/// 飞牛服务健康检查，返回 `{"healthy":bool,"base_url":...}`。
#[no_mangle]
pub extern "C" fn music_core_feiniu_health() -> *mut c_char {
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let snap = match st.feiniu.lock() {
        Ok(g) => FeiniuClient {
            base_url: g.base_url.clone(),
            client: g.client.clone(),
            token: g.token.clone(),
        },
        Err(_) => return error_ptr("Ffi", "飞牛客户端锁获取失败"),
    };
    let base = snap.base_url.clone();
    let result: Result<()> = block_on(st, async move { snap.ping().await });
    match result {
        Ok(()) => to_json_ptr(&serde_json::json!({ "healthy": true, "base_url": base })),
        Err(e) => to_json_ptr(&serde_json::json!({
            "healthy": false,
            "base_url": base,
            "error": { "kind": error_kind(&e), "message": e.to_string() }
        })),
    }
}

// =====================================================================
// 协议源管理（SMB/WebDAV/FTP/DLNA/NFS）
// =====================================================================

/// 添加一个远程协议源，返回 `{"id":...,"protocol":...,"root":...,"enabled":true,"placeholder":bool}`。
///
/// 请求体为协议源配置 JSON（见 protocol-api.md）。WebDAV/FTP 为完整实现，
/// SMB/DLNA/NFS 为占位实现（创建成功但 list/read/stream 返回 Protocol 占位错误）。
///
/// # Safety
/// `config_json` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_protocol_add(config_json: *const c_char) -> *mut c_char {
    let json_str = match unsafe { cstr_to_string(config_json) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "config_json 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };

    let value: serde_json::Value = match serde_json::from_str(&json_str) {
        Ok(v) => v,
        Err(e) => return error_ptr("Json", &e.to_string()),
    };
    let protocol = value
        .get("protocol")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let root = value
        .get("root")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let auth = value.get("auth").cloned().unwrap_or(serde_json::Value::Null);
    let opt_str = |key: &str| -> Option<String> {
        auth.get(key).and_then(|v| v.as_str()).map(String::from)
    };

    use crate::protocols::{dlna::DlnaClient, ftp::FtpClient, nfs::NfsClient, smb::SmbClient,
        webdav::WebDavClient};
    let client: Arc<dyn ProtocolClient> = match protocol.as_str() {
        "webdav" => {
            let base_url = opt_str("base_url").unwrap_or_default();
            Arc::new(WebDavClient::new(
                base_url,
                opt_str("username"),
                opt_str("password"),
            ))
        }
        "ftp" => {
            let host = opt_str("host").or_else(|| value.get("host").and_then(|v| v.as_str()).map(String::from));
            let host = match host {
                Some(h) => h,
                None => return error_ptr("Protocol", "FTP 缺少 host 字段"),
            };
            let port = value
                .get("port")
                .and_then(|v| v.as_u64())
                .unwrap_or(21) as u16;
            let username = opt_str("username").unwrap_or_default();
            let password = opt_str("password").unwrap_or_default();
            Arc::new(FtpClient::new(host, port, username, password))
        }
        "smb" => {
            let host = value
                .get("host")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let share = opt_str("share").unwrap_or_default();
            let username = opt_str("username").unwrap_or_default();
            let password = opt_str("password").unwrap_or_default();
            Arc::new(SmbClient::new(host, share, username, password))
        }
        "dlna" => {
            let control_url = opt_str("control_url").unwrap_or_default();
            Arc::new(DlnaClient::new(control_url))
        }
        "nfs" => {
            let server = value
                .get("host")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let export = root.clone();
            Arc::new(NfsClient::new(server, export))
        }
        other => {
            return error_ptr("Protocol", &format!("不支持的协议类型: {other}"));
        }
    };
    let placeholder = matches!(protocol.as_str(), "smb" | "dlna" | "nfs");

    // 生成 id 并存入表
    let id = {
        let mut seq = match st.protocol_seq.lock() {
            Ok(g) => g,
            Err(_) => return error_ptr("Ffi", "协议源序号锁获取失败"),
        };
        let n = *seq;
        *seq = n + 1;
        format!("proto-{n}")
    };
    {
        let mut protos = match st.protocols.lock() {
            Ok(g) => g,
            Err(_) => return error_ptr("Ffi", "协议源表锁获取失败"),
        };
        protos.insert(
            id.clone(),
            ProtocolEntry {
                client,
                protocol: protocol.clone(),
                root: root.clone(),
            },
        );
    }
    to_json_ptr(&serde_json::json!({
        "id": id,
        "protocol": protocol,
        "root": root,
        "enabled": true,
        "placeholder": placeholder,
    }))
}

/// 列出已加载的协议源，返回 `{"sources":[...]}`。
#[no_mangle]
pub extern "C" fn music_core_protocol_list() -> *mut c_char {
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let protos = match st.protocols.lock() {
        Ok(g) => g,
        Err(_) => return error_ptr("Ffi", "协议源表锁获取失败"),
    };
    let sources: Vec<serde_json::Value> = protos
        .iter()
        .map(|(id, e)| {
            serde_json::json!({
                "id": id,
                "protocol": e.protocol,
                "root": e.root,
                "enabled": true,
                "placeholder": matches!(e.protocol.as_str(), "smb" | "dlna" | "nfs"),
            })
        })
        .collect();
    to_json_ptr(&serde_json::json!({ "sources": sources }))
}

/// 删除指定 id 的协议源，返回 `{"id":...,"deleted":bool}`。
///
/// # Safety
/// `id` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_protocol_delete(id: *const c_char) -> *mut c_char {
    let id_str = match unsafe { cstr_to_string(id) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "id 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let removed = match st.protocols.lock() {
        Ok(mut g) => g.remove(&id_str).is_some(),
        Err(_) => return error_ptr("Ffi", "协议源表锁获取失败"),
    };
    to_json_ptr(&serde_json::json!({ "id": id_str, "deleted": removed }))
}

/// 列出协议源下指定路径的条目名称，返回 `{"path":...,"entries":[...]}`。
///
/// # Safety
/// `id` / `path` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_protocol_list_files(
    id: *const c_char,
    path: *const c_char,
) -> *mut c_char {
    let (id_str, p) = match (unsafe { cstr_to_string(id) }, unsafe { cstr_to_string(path) }) {
        (Some(a), Some(b)) => (a, b),
        _ => return error_ptr("Ffi", "id / path 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let client: Arc<dyn ProtocolClient> = {
        let protos = match st.protocols.lock() {
            Ok(g) => g,
            Err(_) => return error_ptr("Ffi", "协议源表锁获取失败"),
        };
        match protos.get(&id_str) {
            Some(e) => Arc::clone(&e.client),
            None => return error_ptr("NotFound", &format!("协议源不存在: {id_str}")),
        }
    };
    // 克隆一份供后续 JSON 拼装使用（原始 p 被 async move 闭包捕获）
    let p_for_json = p.clone();
    let result: Result<Vec<String>> =
        block_on(st, async move { client.list(&p).await });
    match result {
        Ok(entries) => to_json_ptr(&serde_json::json!({ "path": p_for_json, "entries": entries })),
        Err(e) => error_ptr(error_kind(&e), &e.to_string()),
    }
}

/// 读取协议源下指定文件为字节，返回 `{"size":N,"data_base64":"..."}`。
/// 字节以 base64 编码嵌入 JSON，便于通过 C 字符串传递二进制数据。
///
/// # Safety
/// `id` / `path` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_protocol_read(
    id: *const c_char,
    path: *const c_char,
) -> *mut c_char {
    let (id_str, p) = match (unsafe { cstr_to_string(id) }, unsafe { cstr_to_string(path) }) {
        (Some(a), Some(b)) => (a, b),
        _ => return error_ptr("Ffi", "id / path 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let client: Arc<dyn ProtocolClient> = {
        let protos = match st.protocols.lock() {
            Ok(g) => g,
            Err(_) => return error_ptr("Ffi", "协议源表锁获取失败"),
        };
        match protos.get(&id_str) {
            Some(e) => Arc::clone(&e.client),
            None => return error_ptr("NotFound", &format!("协议源不存在: {id_str}")),
        }
    };
    let result: Result<Vec<u8>> = block_on(st, async move { client.read(&p).await });
    match result {
        Ok(bytes) => {
            let size = bytes.len();
            let b64 = general_purpose::STANDARD.encode(&bytes);
            to_json_ptr(&serde_json::json!({ "size": size, "data_base64": b64 }))
        }
        Err(e) => error_ptr(error_kind(&e), &e.to_string()),
    }
}

/// 生成协议源下指定文件的可流式播放 URL，返回 `{"url":...}`。
///
/// # Safety
/// `id` / `path` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_protocol_stream(
    id: *const c_char,
    path: *const c_char,
) -> *mut c_char {
    let (id_str, p) = match (unsafe { cstr_to_string(id) }, unsafe { cstr_to_string(path) }) {
        (Some(a), Some(b)) => (a, b),
        _ => return error_ptr("Ffi", "id / path 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let client: Arc<dyn ProtocolClient> = {
        let protos = match st.protocols.lock() {
            Ok(g) => g,
            Err(_) => return error_ptr("Ffi", "协议源表锁获取失败"),
        };
        match protos.get(&id_str) {
            Some(e) => Arc::clone(&e.client),
            None => return error_ptr("NotFound", &format!("协议源不存在: {id_str}")),
        }
    };
    let result: Result<String> = block_on(st, async move { client.stream_url(&p).await });
    match result {
        Ok(url) => to_json_ptr(&serde_json::json!({ "url": url })),
        Err(e) => error_ptr(error_kind(&e), &e.to_string()),
    }
}

// =====================================================================
// 本地音乐
// =====================================================================

/// 初始化本地音乐源（打开/创建 SQLite 索引库），返回 `{"ok":true}`。
/// 需在使用 add_dir/rescan/progress 之前调用一次。
///
/// # Safety
/// `db_path` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_local_init(db_path: *const c_char) -> *mut c_char {
    let p = match unsafe { cstr_to_string(db_path) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "db_path 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let local = match LocalSource::new(&p) {
        Ok(l) => l,
        Err(e) => return error_ptr(error_kind(&e), &e.to_string()),
    };
    match st.local.lock() {
        Ok(mut g) => {
            *g = Some(local);
            to_json_ptr(&serde_json::json!({ "ok": true }))
        }
        Err(_) => error_ptr("Ffi", "本地音源锁获取失败"),
    }
}

/// 添加本地扫描目录并递归扫描入库，返回 `{"ok":true}`。
///
/// # Safety
/// `dir` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_local_add_dir(dir: *const c_char) -> *mut c_char {
    let d = match unsafe { cstr_to_string(dir) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "dir 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let local = match st.local.lock() {
        Ok(g) => g,
        Err(_) => return error_ptr("Ffi", "本地音源锁获取失败"),
    };
    let local = match local.as_ref() {
        Some(l) => l,
        None => return error_ptr("Ffi", "本地音源未初始化，请先调用 music_core_local_init"),
    };
    match local.add_directory(&d) {
        Ok(()) => to_json_ptr(&serde_json::json!({ "ok": true })),
        Err(e) => error_ptr(error_kind(&e), &e.to_string()),
    }
}

/// 重新扫描所有已添加目录（增量更新），返回 `{"ok":true}`。
#[no_mangle]
pub extern "C" fn music_core_local_rescan() -> *mut c_char {
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let local = match st.local.lock() {
        Ok(g) => g,
        Err(_) => return error_ptr("Ffi", "本地音源锁获取失败"),
    };
    let local = match local.as_ref() {
        Some(l) => l,
        None => return error_ptr("Ffi", "本地音源未初始化，请先调用 music_core_local_init"),
    };
    match local.rescan() {
        Ok(()) => to_json_ptr(&serde_json::json!({ "ok": true })),
        Err(e) => error_ptr(error_kind(&e), &e.to_string()),
    }
}

/// 返回本地扫描进度，返回 `{"current_count":N,"scanning":bool}`。
#[no_mangle]
pub extern "C" fn music_core_local_progress() -> *mut c_char {
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let local = match st.local.lock() {
        Ok(g) => g,
        Err(_) => return error_ptr("Ffi", "本地音源锁获取失败"),
    };
    let progress = match local.as_ref() {
        Some(l) => l.scan_progress(),
        None => {
            return to_json_ptr(&serde_json::json!({ "current_count": 0, "scanning": false }))
        }
    };
    to_json_ptr(&serde_json::json!({
        "current_count": progress.current_count,
        "scanning": progress.scanning,
    }))
}

// =====================================================================
// 缓存
// =====================================================================

/// 初始化播放缓存管理器，返回 `{"ok":true}`。需在 stats/clear 之前调用一次。
///
/// # Safety
/// `cache_dir` 必须为 NULL 或指向合法 NUL 结尾的 C 字符串。
#[no_mangle]
pub unsafe extern "C" fn music_core_cache_init(
    cache_dir: *const c_char,
    max_bytes: u64,
) -> *mut c_char {
    let d = match unsafe { cstr_to_string(cache_dir) } {
        Some(s) => s,
        None => return error_ptr("Ffi", "cache_dir 参数为空指针"),
    };
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let cache = match CacheManager::new(&d, max_bytes) {
        Ok(c) => c,
        Err(e) => return error_ptr(error_kind(&e), &e.to_string()),
    };
    match st.cache.lock() {
        Ok(mut g) => {
            *g = Some(cache);
            to_json_ptr(&serde_json::json!({ "ok": true }))
        }
        Err(_) => error_ptr("Ffi", "缓存管理器锁获取失败"),
    }
}

/// 返回缓存统计，返回 `{"entries":N,"total_bytes":N,"max_bytes":N}`。
/// 未初始化时返回零值统计。
#[no_mangle]
pub extern "C" fn music_core_cache_stats() -> *mut c_char {
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let stats: CacheStats = match st.cache.lock() {
        Ok(g) => match g.as_ref() {
            Some(c) => c.stats(),
            None => CacheStats {
                entries: 0,
                total_bytes: 0,
                max_bytes: 0,
            },
        },
        Err(_) => return error_ptr("Ffi", "缓存管理器锁获取失败"),
    };
    to_json_ptr(&serde_json::json!({
        "entries": stats.entries,
        "total_bytes": stats.total_bytes,
        "max_bytes": stats.max_bytes,
    }))
}

/// 清空所有缓存文件与索引，返回 `{"ok":true}`。
#[no_mangle]
pub extern "C" fn music_core_cache_clear() -> *mut c_char {
    let st = match state() {
        Some(s) => s,
        None => return error_ptr("Ffi", "核心运行时未初始化"),
    };
    let cache = match st.cache.lock() {
        Ok(g) => g,
        Err(_) => return error_ptr("Ffi", "缓存管理器锁获取失败"),
    };
    let cache = match cache.as_ref() {
        Some(c) => c,
        None => return error_ptr("Ffi", "缓存未初始化，请先调用 music_core_cache_init"),
    };
    match cache.clear() {
        Ok(()) => to_json_ptr(&serde_json::json!({ "ok": true })),
        Err(e) => error_ptr(error_kind(&e), &e.to_string()),
    }
}
