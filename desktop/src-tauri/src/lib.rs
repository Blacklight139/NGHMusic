//! 桌面端 Tauri 命令实现。
//!
//! 通过 `#[tauri::command]` 暴露给前端，内部调用 `music-core` 核心。
//! 音源管理类 command 通过共享 `Mutex<SourceManager>` state 聚合调用；
//! 搜索/播放/缓存等其余能力后续填充。

use std::sync::Mutex;

use music_core::models::{Song, SearchResult};
use music_core::sources::community::adapt_with_report;
use music_core::sources::schema::SoundSourceConfig;
use music_core::sources::{SourceInfo, SourceManager};
use tauri::{Manager, State};

/// 共享的音源管理器 state：`Mutex` 提供 interior mutability，供 command 获取可变访问。
type SourceManagerState = Mutex<SourceManager>;

/// 返回应用版本（来自 Cargo.toml）。
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 音源导入：接收 JSON 字符串，经社区适配 + schema 严格校验后返回标准配置 JSON。
///
/// 仅做适配与校验，不注册到 SourceManager。需注册并持久化请使用 `import_source_file`。
#[tauri::command]
fn import_source(json: String) -> Result<String, String> {
    // 解析原始 JSON
    let raw: serde_json::Value = serde_json::from_str(&json).map_err(|e| e.to_string())?;
    // 社区格式适配（standard / community-a / community-b）
    let report = adapt_with_report(&raw).map_err(|e| e.to_string())?;
    // 对适配后的标准配置做严格 schema 校验
    SoundSourceConfig::validate_strict(&report.config).map_err(|e| e.to_string())?;
    // 返回标准配置 JSON 字符串
    serde_json::to_string(&report.config).map_err(|e| e.to_string())
}

/// 关键字搜索（占位）：返回空结果。实际需注入 SourceManager 到 state 后聚合调用。
#[tauri::command]
async fn search(keyword: String, page: u32, page_size: u32) -> Result<SearchResult, String> {
    Ok(SearchResult {
        keyword,
        songs: vec![],
        albums: vec![],
        artists: vec![],
        total: 0,
        page,
        page_size,
    })
}

/// 列出本地歌曲（占位）：返回空列表。
#[tauri::command]
fn list_local_songs() -> Result<Vec<Song>, String> {
    Ok(vec![])
}

/// 返回按优先级降序排列的全部音源信息（含启停项），供前端展示音源列表。
#[tauri::command]
fn list_sources(state: State<'_, SourceManagerState>) -> Result<Vec<SourceInfoDto>, String> {
    let sm = state.lock().map_err(|e| e.to_string())?;
    Ok(sm
        .list_sources_ordered()
        .into_iter()
        .map(SourceInfoDto::from)
        .collect())
}

/// 更新单个音源的优先级；id 不存在返回错误。
#[tauri::command]
fn update_source_priority(
    state: State<'_, SourceManagerState>,
    id: String,
    new_priority: i32,
) -> Result<(), String> {
    let mut sm = state.lock().map_err(|e| e.to_string())?;
    sm.update_source_priority(&id, new_priority)
        .map_err(|e| e.to_string())
}

/// 按给定 id 顺序重排音源优先级（越靠前优先级越高）；任一 id 不存在返回错误。
#[tauri::command]
fn reorder_sources(
    state: State<'_, SourceManagerState>,
    ordered_ids: Vec<String>,
) -> Result<(), String> {
    let mut sm = state.lock().map_err(|e| e.to_string())?;
    sm.reorder_sources(&ordered_ids).map_err(|e| e.to_string())
}

/// 删除指定 id 的音源；id 不存在返回错误。
#[tauri::command]
fn delete_source(state: State<'_, SourceManagerState>, id: String) -> Result<(), String> {
    let mut sm = state.lock().map_err(|e| e.to_string())?;
    sm.delete_source(&id).map_err(|e| e.to_string())
}

/// 设置音源启停状态；id 不存在返回错误。
#[tauri::command]
fn set_source_enabled(
    state: State<'_, SourceManagerState>,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let mut sm = state.lock().map_err(|e| e.to_string())?;
    sm.set_source_enabled(&id, enabled)
        .map_err(|e| e.to_string())
}

/// 从本地 JSON 文件导入音源：读取文件 → 社区适配 + schema 校验 → 注册到 SourceManager。
///
/// 直接用 `std::fs` 读取（不走 tauri fs 插件），故无需额外 fs 权限声明。
/// 返回新音源的 `SourceInfoDto`。
#[tauri::command]
fn import_source_file(
    state: State<'_, SourceManagerState>,
    file_path: String,
) -> Result<SourceInfoDto, String> {
    // 先读文件再持锁，避免持锁期间做 IO
    let json_str = std::fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
    let mut sm = state.lock().map_err(|e| e.to_string())?;
    let info = sm
        .import_source_from_json(&json_str)
        .map_err(|e| e.to_string())?;
    Ok(SourceInfoDto::from(info))
}

/// 桌面端返回给前端的音源信息 DTO，字段与核心 `SourceInfo` 一一对应。
#[derive(serde::Serialize)]
struct SourceInfoDto {
    id: String,
    name: String,
    version: String,
    enabled: bool,
    source_type: String,
    priority: i32,
    description: Option<String>,
}

impl From<SourceInfo> for SourceInfoDto {
    fn from(i: SourceInfo) -> Self {
        Self {
            id: i.id,
            name: i.name,
            version: i.version,
            enabled: i.enabled,
            source_type: i.source_type,
            priority: i.priority,
            description: i.description,
        }
    }
}

/// 构建 Tauri 应用并注册全部 command。
///
/// 在 `setup` 中构造 `SourceManager` 并设置持久化路径（应用数据目录下的
/// `sources_state.json`），使音源顺序与启停状态跨重启保留。获取目录失败时
/// 退化为无持久化（不影响应用启动）。
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let mut sm = SourceManager::new();
            if let Ok(dir) = app.path().app_data_dir() {
                let _ = std::fs::create_dir_all(&dir);
                sm.set_persistence_path(dir.join("sources_state.json"));
            }
            app.manage(Mutex::new(sm));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            app_version,
            import_source,
            search,
            list_local_songs,
            list_sources,
            update_source_priority,
            reorder_sources,
            delete_source,
            set_source_enabled,
            import_source_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
