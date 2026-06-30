//! 桌面端 Tauri 命令实现。
//!
//! 通过 `#[tauri::command]` 暴露给前端，内部调用 `music-core` 核心。
//! 当前为骨架实现，实际业务逻辑（SourceManager 注入、播放、缓存等）后续填充。

use music_core::models::{Song, SearchResult};
use music_core::sources::community::adapt_with_report;
use music_core::sources::schema::SoundSourceConfig;

/// 返回应用版本（来自 Cargo.toml）。
#[tauri::command]
fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 音源导入：接收 JSON 字符串，经社区适配 + schema 严格校验后返回标准配置 JSON。
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

/// 构建 Tauri 应用并注册全部 command。
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            app_version,
            import_source,
            search,
            list_local_songs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
