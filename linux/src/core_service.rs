//! 核心服务层：music-core 能力的纯 Rust 封装。
//!
//! 本模块面向 NGHMusic Linux 客户端（GTK4 + Rust），将 music-core 提供的
//! 音源管理、聚合搜索、飞牛 NAS、远程协议、本地音乐、缓存等能力整合为
//! 一个线程安全的 [`CoreService`]。与 FFI 模块不同，本模块直接返回强类型
//! 结果，无需 JSON 序列化与 C ABI 边界。
//!
//! ## 设计要点
//! - 内部持有 `tokio::runtime::Runtime`，通过 `block_on` 驱动 async 能力，
//!   对调用方表现为同步方法，便于在 GTK 主线程直接调用。
//! - 共享状态（SourceManager / FeiniuClient / 协议源表 / 本地音源 / 缓存）
//!   以 `Arc<Mutex<>>` 包裹，保证线程安全。
//! - 提供 [`CoreService::instance`] 单例入口（`OnceLock`），也支持
//!   [`CoreService::new`] 自行构造实例。
//! - 异步操作前先在锁内克隆所需句柄（`Arc` 或字段快照）并释放锁，
//!   避免持有锁跨越 await。

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex, OnceLock};

use music_core::cache::CacheManager;
use music_core::feiniu::{FeiniuClient, NasFile};
use music_core::protocols::ProtocolClient;
use music_core::protocols::{
    dlna::DlnaClient, ftp::FtpClient, nfs::NfsClient, smb::SmbClient, webdav::WebDavClient,
};
use music_core::sources::local::LocalSource;
use music_core::sources::schema::{validate_config, SoundSourceConfig};
use music_core::sources::{Source, SourceInfo, SourceManager};
use music_core::{CoreError, Leaderboard, Lyric, Result, SearchResult, Song};

/// 已注册协议源条目：客户端 + 展示元信息。
struct ProtocolEntry {
    client: Arc<dyn ProtocolClient>,
    protocol: String,
    root: String,
}

/// 远程协议源展示信息。
///
/// 由 [`CoreService::protocol_add`] / [`CoreService::protocol_list`] 返回，
/// 供上层展示已加载的 NAS/远程协议源。
#[derive(Debug, Clone)]
pub struct ProtocolSourceInfo {
    /// 协议源唯一标识（如 `proto-1`）
    pub id: String,
    /// 协议类型（`webdav` / `ftp` / `smb` / `dlna` / `nfs`）
    pub protocol: String,
    /// 根路径
    pub root: String,
    /// 是否启用（当前实现恒为 true）
    pub enabled: bool,
    /// 是否为占位实现（smb/dlna/nfs 为占位）
    pub placeholder: bool,
}

/// 核心服务：music-core 能力的线程安全封装。
///
/// 持有 tokio 运行时与各核心组件，提供同步方法驱动 async 能力。
/// 通过 [`CoreService::instance`] 获取全局单例，或通过 [`CoreService::new`]
/// 自行构造。所有方法均为 `&self`，可在多线程中共享调用。
pub struct CoreService {
    rt: tokio::runtime::Runtime,
    sm: Arc<Mutex<SourceManager>>,
    feiniu: Arc<Mutex<FeiniuClient>>,
    protocols: Arc<Mutex<HashMap<String, ProtocolEntry>>>,
    protocol_seq: Arc<Mutex<u64>>,
    // 使用 Arc<LocalSource>：扫描时仅在锁内 clone Arc，释放锁后扫描，
    // 使 local_progress 能并发查询进度而不被长时间阻塞。
    local: Arc<Mutex<Option<Arc<LocalSource>>>>,
    cache: Arc<Mutex<Option<CacheManager>>>,
}

impl CoreService {
    /// 创建核心服务实例：初始化 tokio 多线程运行时与各空核心组件。
    pub fn new() -> Self {
        let rt = tokio::runtime::Runtime::new().expect("创建 tokio 运行时失败");
        Self {
            rt,
            sm: Arc::new(Mutex::new(SourceManager::new())),
            feiniu: Arc::new(Mutex::new(FeiniuClient::new(String::new()))),
            protocols: Arc::new(Mutex::new(HashMap::new())),
            protocol_seq: Arc::new(Mutex::new(1)),
            local: Arc::new(Mutex::new(None)),
            cache: Arc::new(Mutex::new(None)),
        }
    }

    /// 获取全局单例引用（首次调用时惰性初始化）。
    pub fn instance() -> &'static CoreService {
        static INSTANCE: OnceLock<CoreService> = OnceLock::new();
        INSTANCE.get_or_init(CoreService::new)
    }

    /// 在 tokio 运行时上阻塞驱动 async future 至完成。
    fn block_on<F: Future>(&self, fut: F) -> F::Output {
        self.rt.block_on(fut)
    }

    /// 通用锁获取辅助：将锁中毒错误映射为 [`CoreError::Ffi`]。
    fn lock<'a, T>(
        &self,
        name: &str,
        m: &'a Arc<Mutex<T>>,
    ) -> Result<std::sync::MutexGuard<'a, T>> {
        m.lock().map_err(|_| CoreError::Ffi(format!("{name} 锁获取失败")))
    }

    // =================================================================
    // 音源管理
    // =================================================================

    /// 导入音源 JSON（社区格式自动适配为标准配置），返回导入后的音源信息。
    ///
    /// 若 id 已存在则覆盖旧的同 id 音源。新导入的音源默认启用并置于列表最前。
    pub fn source_import(&self, json: &str) -> Result<SourceInfo> {
        let mut sm = self.lock("音源管理器", &self.sm)?;
        sm.import_source_from_json(json)
    }

    /// 校验音源 JSON 是否符合标准 Schema（不加载、不持久化）。
    ///
    /// 先做严格 schema 校验，再做语义校验（`validate_config`），收集所有错误。
    /// 返回 `(是否合法, 错误信息列表)`；JSON 解析失败返回 `Err`。
    pub fn source_validate(&self, json: &str) -> Result<(bool, Vec<String>)> {
        let value: serde_json::Value = serde_json::from_str(json)?;

        if let Err(e) = SoundSourceConfig::validate_strict(&value) {
            return Ok((false, vec![e.to_string()]));
        }
        let config = match SoundSourceConfig::from_json(json) {
            Ok(c) => c,
            Err(e) => return Ok((false, vec![e.to_string()])),
        };
        match validate_config(&config) {
            Ok(()) => Ok((true, Vec::new())),
            Err(e) => Ok((false, vec![e.to_string()])),
        }
    }

    /// 列出所有音源（按 priority 降序），含启停项。
    pub fn source_list(&self) -> Vec<SourceInfo> {
        match self.sm.lock() {
            Ok(sm) => sm.list_sources_ordered(),
            Err(_) => Vec::new(),
        }
    }

    /// 启用指定 id 的音源；id 不存在返回 `NotFound` 错误。
    pub fn source_enable(&self, id: &str) -> Result<()> {
        let mut sm = self.lock("音源管理器", &self.sm)?;
        sm.set_source_enabled(id, true)
    }

    /// 禁用指定 id 的音源；id 不存在返回 `NotFound` 错误。
    pub fn source_disable(&self, id: &str) -> Result<()> {
        let mut sm = self.lock("音源管理器", &self.sm)?;
        sm.set_source_enabled(id, false)
    }

    /// 删除指定 id 的音源；id 不存在返回 `NotFound` 错误。
    pub fn source_delete(&self, id: &str) -> Result<()> {
        let mut sm = self.lock("音源管理器", &self.sm)?;
        sm.delete_source(id)
    }

    // =================================================================
    // 搜索与歌曲（异步，经 runtime block_on 驱动）
    // =================================================================

    /// 聚合搜索：跨所有启用音源搜索，单个音源失败记录警告并跳过。
    pub fn search(&self, keyword: &str, page: u32, page_size: u32) -> Result<SearchResult> {
        // 先在锁内克隆各启用音源的 Arc，立即释放锁，避免持有锁跨越 await。
        let sources: Vec<Arc<dyn Source>> = {
            let sm = self.lock("音源管理器", &self.sm)?;
            sm.ordered_sources().iter().map(Arc::clone).collect()
        };
        let kw = keyword.to_string();
        self.block_on(async move {
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
        })
    }

    /// 获取指定音源下歌曲的完整元数据。
    pub fn get_metadata(&self, source_id: &str, song_id: &str) -> Result<Song> {
        let source = self.get_source_by_id(source_id)?;
        let song_id = song_id.to_string();
        self.block_on(async move { source.get_metadata(&song_id).await })
    }

    /// 获取指定音源下歌曲的可播放 URL。
    pub fn get_play_url(&self, source_id: &str, song_id: &str) -> Result<String> {
        let source = self.get_source_by_id(source_id)?;
        let song_id = song_id.to_string();
        self.block_on(async move { source.get_play_url(&song_id).await })
    }

    /// 获取指定音源下歌曲的歌词。
    pub fn get_lyric(&self, source_id: &str, song_id: &str) -> Result<Lyric> {
        let source = self.get_source_by_id(source_id)?;
        let song_id = song_id.to_string();
        self.block_on(async move { source.get_lyric(&song_id).await })
    }

    /// 获取指定音源的排行榜列表。
    pub fn get_leaderboards(&self, source_id: &str) -> Result<Vec<Leaderboard>> {
        let source = self.get_source_by_id(source_id)?;
        self.block_on(async move { source.get_leaderboards().await })
    }

    /// 按 id 查找音源句柄（锁内克隆 `Arc`，立即释放锁）。
    fn get_source_by_id(&self, source_id: &str) -> Result<Arc<dyn Source>> {
        let sm = self.lock("音源管理器", &self.sm)?;
        sm.get_source(source_id)
            .ok_or_else(|| CoreError::NotFound(format!("音源不存在: {source_id}")))
    }

    // =================================================================
    // 飞牛 NAS
    // =================================================================

    /// 登录飞牛 NAS，返回 `(token, base_url)`。
    ///
    /// 内部创建独立客户端完成登录后写回全局状态，避免持有锁跨越网络 await。
    pub fn feiniu_login(
        &self,
        base_url: &str,
        username: &str,
        password: &str,
    ) -> Result<(String, String)> {
        let mut client = FeiniuClient::new(base_url.to_string());
        let user = username.to_string();
        let pass = password.to_string();
        self.block_on(async { client.login(&user, &pass).await })?;
        let token = client.token.clone().unwrap_or_default();
        let base = base_url.to_string();
        // 写回全局状态；锁中毒时返回错误而非静默吞错
        {
            let mut g = self.lock("飞牛客户端", &self.feiniu)?;
            *g = client;
        }
        Ok((token, base))
    }

    /// 列出飞牛 NAS 指定路径下的文件。
    pub fn feiniu_list_files(&self, path: &str) -> Result<Vec<NasFile>> {
        let snap = self.feiniu_snapshot()?;
        let p = path.to_string();
        self.block_on(async move { snap.list_files(&p).await })
    }

    /// 生成飞牛 NAS 文件的可流式播放 URL。
    pub fn feiniu_stream(&self, path: &str) -> Result<String> {
        let snap = self.feiniu_snapshot()?;
        let p = path.to_string();
        self.block_on(async move { snap.get_stream_url(&p).await })
    }

    /// 飞牛服务健康检查。
    pub fn feiniu_health(&self) -> Result<()> {
        let snap = self.feiniu_snapshot()?;
        self.block_on(async move { snap.ping().await })
    }

    /// 克隆飞牛客户端快照（字段均为 `Clone`），释放锁后再异步。
    fn feiniu_snapshot(&self) -> Result<FeiniuClient> {
        let g = self.lock("飞牛客户端", &self.feiniu)?;
        Ok(FeiniuClient {
            base_url: g.base_url.clone(),
            client: g.client.clone(),
            token: g.token.clone(),
        })
    }

    // =================================================================
    // 协议源管理（SMB/WebDAV/FTP/DLNA/NFS）
    // =================================================================

    /// 添加一个远程协议源，返回其展示信息。
    ///
    /// 请求体为协议源配置 JSON。WebDAV/FTP 为完整实现，SMB/DLNA/NFS 为占位实现
    ///（创建成功但 list/read/stream 返回 Protocol 占位错误）。
    pub fn protocol_add(&self, config_json: &str) -> Result<ProtocolSourceInfo> {
        let value: serde_json::Value = serde_json::from_str(config_json)?;
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
                let host = opt_str("host")
                    .or_else(|| value.get("host").and_then(|v| v.as_str()).map(String::from));
                let host =
                    host.ok_or_else(|| CoreError::Protocol("FTP 缺少 host 字段".into()))?;
                let port_val = value
                    .get("port")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(21);
                if port_val > 65535 {
                    return Err(CoreError::Protocol(format!(
                        "FTP 端口超出有效范围: {port_val}（最大 65535）"
                    )));
                }
                let port = port_val as u16;
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
                return Err(CoreError::Protocol(format!("不支持的协议类型: {other}")));
            }
        };
        let placeholder = matches!(protocol.as_str(), "smb" | "dlna" | "nfs");

        // 生成 id 并存入表
        let id = {
            let mut seq = self.lock("协议源序号", &self.protocol_seq)?;
            let n = *seq;
            *seq = n + 1;
            format!("proto-{n}")
        };
        {
            let mut protos = self.lock("协议源表", &self.protocols)?;
            protos.insert(
                id.clone(),
                ProtocolEntry {
                    client,
                    protocol: protocol.clone(),
                    root: root.clone(),
                },
            );
        }
        Ok(ProtocolSourceInfo {
            id,
            protocol,
            root,
            enabled: true,
            placeholder,
        })
    }

    /// 列出已加载的协议源。
    pub fn protocol_list(&self) -> Vec<ProtocolSourceInfo> {
        match self.protocols.lock() {
            Ok(protos) => protos
                .iter()
                .map(|(id, e)| ProtocolSourceInfo {
                    id: id.clone(),
                    protocol: e.protocol.clone(),
                    root: e.root.clone(),
                    enabled: true,
                    placeholder: matches!(e.protocol.as_str(), "smb" | "dlna" | "nfs"),
                })
                .collect(),
            Err(_) => Vec::new(),
        }
    }

    /// 删除指定 id 的协议源，返回是否删除成功。
    pub fn protocol_delete(&self, id: &str) -> bool {
        match self.protocols.lock() {
            Ok(mut protos) => protos.remove(id).is_some(),
            Err(_) => false,
        }
    }

    /// 列出协议源下指定路径的条目名称。
    pub fn protocol_list_files(&self, id: &str, path: &str) -> Result<Vec<String>> {
        let client = self.get_protocol_client(id)?;
        let p = path.to_string();
        self.block_on(async move { client.list(&p).await })
    }

    /// 生成协议源下指定文件的可流式播放 URL。
    pub fn protocol_stream(&self, id: &str, path: &str) -> Result<String> {
        let client = self.get_protocol_client(id)?;
        let p = path.to_string();
        self.block_on(async move { client.stream_url(&p).await })
    }

    /// 按 id 查找协议源客户端（锁内克隆 `Arc`，立即释放锁）。
    fn get_protocol_client(&self, id: &str) -> Result<Arc<dyn ProtocolClient>> {
        let protos = self.lock("协议源表", &self.protocols)?;
        protos
            .get(id)
            .map(|e| Arc::clone(&e.client))
            .ok_or_else(|| CoreError::NotFound(format!("协议源不存在: {id}")))
    }

    // =================================================================
    // 本地音乐
    // =================================================================

    /// 初始化本地音乐源（打开/创建 SQLite 索引库）。
    /// 需在使用 add_dir/rescan/progress 之前调用一次。
    pub fn local_init(&self, db_path: &str) -> Result<()> {
        let local = LocalSource::new(db_path)?;
        let mut g = self.lock("本地音源", &self.local)?;
        *g = Some(Arc::new(local));
        Ok(())
    }

    /// 添加本地扫描目录并递归扫描入库。
    ///
    /// 仅在锁内 clone `Arc<LocalSource>`，立即释放锁后再扫描，
    /// 使 [`Self::local_progress`] 能并发查询进度而不被长时间阻塞。
    pub fn local_add_dir(&self, dir: &str) -> Result<()> {
        let local = {
            let g = self.lock("本地音源", &self.local)?;
            g.as_ref()
                .map(Arc::clone)
                .ok_or_else(|| CoreError::Ffi("本地音源未初始化，请先调用 local_init".into()))?
        };
        // 锁已释放，扫描期间 local_progress 可并发查询
        local.add_directory(dir)
    }

    /// 重新扫描所有已添加目录（增量更新）。
    ///
    /// 与 [`Self::local_add_dir`] 同理：锁内 clone Arc 后释放锁再扫描。
    pub fn local_rescan(&self) -> Result<()> {
        let local = {
            let g = self.lock("本地音源", &self.local)?;
            g.as_ref()
                .map(Arc::clone)
                .ok_or_else(|| CoreError::Ffi("本地音源未初始化，请先调用 local_init".into()))?
        };
        // 锁已释放，扫描期间 local_progress 可并发查询
        local.rescan()
    }

    /// 列出本地音乐库中的全部歌曲（供 UI 浏览，非聚合搜索）。
    ///
    /// 直接调用 [`LocalSource::list_all`]，仅返回本地音源已索引的歌曲，
    /// 避免使用聚合 `search("", ...)` 把远程音源结果混入本地音乐页。
    pub fn local_list_songs(&self) -> Result<Vec<Song>> {
        let local = {
            let g = self.lock("本地音源", &self.local)?;
            g.as_ref()
                .map(Arc::clone)
                .ok_or_else(|| CoreError::Ffi("本地音源未初始化，请先调用 local_init".into()))?
        };
        local.list_all()
    }

    /// 返回本地扫描进度 `(已索引数, 是否扫描中)`。
    pub fn local_progress(&self) -> (usize, bool) {
        match self.local.lock() {
            Ok(g) => match g.as_ref() {
                Some(l) => {
                    let p = l.scan_progress();
                    (p.current_count, p.scanning)
                }
                None => (0, false),
            },
            Err(_) => (0, false),
        }
    }

    // =================================================================
    // 缓存
    // =================================================================

    /// 初始化播放缓存管理器。需在 stats/clear 之前调用一次。
    pub fn cache_init(&self, cache_dir: &str, max_bytes: u64) -> Result<()> {
        let cache = CacheManager::new(cache_dir, max_bytes)?;
        let mut g = self.lock("缓存管理器", &self.cache)?;
        *g = Some(cache);
        Ok(())
    }

    /// 返回缓存统计 `(条目数, 总字节数, 容量上限)`。未初始化时返回零值。
    pub fn cache_stats(&self) -> (usize, u64, u64) {
        match self.cache.lock() {
            Ok(g) => match g.as_ref() {
                Some(c) => {
                    let s = c.stats();
                    (s.entries, s.total_bytes, s.max_bytes)
                }
                None => (0, 0, 0),
            },
            Err(_) => (0, 0, 0),
        }
    }

    /// 清空所有缓存文件与索引。
    pub fn cache_clear(&self) -> Result<()> {
        let g = self.lock("缓存管理器", &self.cache)?;
        let cache = g
            .as_ref()
            .ok_or_else(|| CoreError::Ffi("缓存未初始化，请先调用 cache_init".into()))?;
        cache.clear()
    }
}

impl Default for CoreService {
    fn default() -> Self {
        Self::new()
    }
}
