//! FFI 边界：将核心能力以 C ABI 暴露给各端（iOS Swift / Android JNI / HarmonyOS NAPI / Tauri）。
//!
//! 设计要点：
//! - 字符串以 `\0` 结尾 UTF-8 传递；调用方负责释放。
//! - 复杂返回值统一序列化为 JSON 字符串。
//! - 错误经 [FfiResult] 编码（code + message）。
//! - 长时操作（搜索/播放）由各端自行驱动异步运行时；FFI 仅暴露同步入口包装。

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;
use crate::CoreError;

/// FFI 返回码
#[repr(i32)]
#[derive(Debug, Clone, Copy)]
pub enum FfiCode {
    Ok = 0,
    Err = 1,
    NullPtr = 2,
    Utf8 = 3,
    Panic = 4,
}

/// FFI 通用结果（JSON 化）
#[repr(C)]
pub struct FfiResult {
    pub code: i32,
    /// 指向 JSON 字符串（成功时为数据，失败时为 {"error":"..."}）
    pub data: *mut c_char,
}

impl FfiResult {
    pub fn ok_json<T: serde::Serialize>(v: &T) -> Self {
        let s = serde_json::to_string(v).unwrap_or_else(|_| "{}".into());
        Self {
            code: FfiCode::Ok as i32,
            data: to_cstring_ptr(s),
        }
    }

    pub fn ok_empty() -> Self {
        Self {
            code: FfiCode::Ok as i32,
            data: std::ptr::null_mut(),
        }
    }

    pub fn err(code: FfiCode, msg: impl Into<String>) -> Self {
        let s = format!("{{\"error\":\"{}\"}}", escape_json(msg.into()));
        Self {
            code: code as i32,
            data: to_cstring_ptr(s),
        }
    }
}

/// 从 C 字符串读取 UTF-8 字符串；空指针或非法 UTF-8 返回 None
pub fn read_cstr(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }
    unsafe {
        let cstr = CStr::from_ptr(ptr);
        cstr.to_str().ok().map(|s| s.to_string())
    }
}

/// 分配 CString 并返回裸指针；调用方需用 [free_string] 释放
pub fn to_cstring_ptr(s: impl Into<String>) -> *mut c_char {
    match CString::new(s.into()) {
        Ok(c) => c.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}

fn escape_json(s: String) -> String {
    serde_json::to_string(&s).unwrap_or_else(|_| "null".into())
}

/// 释放由核心返回的字符串
#[no_mangle]
pub extern "C" fn music_core_free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            drop(CString::from_raw(ptr));
        }
    }
}

/// 将 [CoreError] 转 [FfiResult]
impl From<CoreError> for FfiResult {
    fn from(e: CoreError) -> Self {
        FfiResult::err(FfiCode::Err, e.to_string())
    }
}

/// 全局运行时与引擎句柄（简化：单例）
pub struct CoreHandle {
    pub engine: Arc<crate::sources::SourceEngine>,
    pub runtime: tokio::runtime::Runtime,
}

impl CoreHandle {
    pub fn global() -> &'static CoreHandle {
        use std::sync::OnceLock;
        static H: OnceLock<CoreHandle> = OnceLock::new();
        H.get_or_init(|| {
            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("tokio runtime");
            CoreHandle {
                engine: Arc::new(crate::sources::SourceEngine::new()),
                runtime,
            }
        })
    }
}

/// FFI：导入音源 JSON（标准或社区格式）
#[no_mangle]
pub extern "C" fn music_core_import_source(json_ptr: *const c_char) -> FfiResult {
    let Some(json) = read_cstr(json_ptr) else {
        return FfiResult::err(FfiCode::NullPtr, "json_ptr is null");
    };
    let handle = CoreHandle::global();
    // migrate 与 register 均为同步
    let result = (|| {
        let res = crate::sources::migration::migrate(&json)?;
        handle.engine.register(res.config, 0)
    })();
    match result {
        Ok(id) => FfiResult::ok_json(&id),
        Err(e) => FfiResult::from(e),
    }
}

/// FFI：列出音源（返回 JSON）
#[no_mangle]
pub extern "C" fn music_core_list_sources() -> FfiResult {
    let handle = CoreHandle::global();
    let list = handle.engine.list();
    let serialized: Vec<serde_json::Value> = list
        .into_iter()
        .map(|(id, name, enabled, priority)| {
            serde_json::json!({ "id": id, "name": name, "enabled": enabled, "priority": priority })
        })
        .collect();
    FfiResult::ok_json(&serialized)
}

/// FFI：设置音源启用状态
#[no_mangle]
pub extern "C" fn music_core_set_enabled(id_ptr: *const c_char, enabled: i32) -> FfiResult {
    let Some(id) = read_cstr(id_ptr) else {
        return FfiResult::err(FfiCode::NullPtr, "id_ptr is null");
    };
    let handle = CoreHandle::global();
    match handle.engine.set_enabled(&id, enabled != 0) {
        Ok(()) => FfiResult::ok_empty(),
        Err(e) => FfiResult::from(e),
    }
}

/// FFI：搜索（keyword, type_json, page_json） -> 结果 JSON
#[no_mangle]
pub extern "C" fn music_core_search(
    keyword_ptr: *const c_char,
    type_ptr: *const c_char,
    page_ptr: *const c_char,
) -> FfiResult {
    let (Some(keyword), Some(ty_str), Some(page_str)) =
        (read_cstr(keyword_ptr), read_cstr(type_ptr), read_cstr(page_ptr))
    else {
        return FfiResult::err(FfiCode::NullPtr, "argument is null");
    };
    let ty: crate::models::SearchType = match serde_json::from_str(&ty_str) {
        Ok(t) => t,
        Err(_) => crate::models::SearchType::Song,
    };
    let page: crate::models::Page = serde_json::from_str(&page_str).unwrap_or_default();
    let handle = CoreHandle::global();
    let engine = handle.engine.clone();
    match handle.runtime.block_on(async move {
        let svc = crate::search::SearchService::new(&engine);
        svc.search(&keyword, ty, page).await
    }) {
        Ok(p) => FfiResult::ok_json(&p),
        Err(e) => FfiResult::from(e),
    }
}
