//! 标准音源 JSON Schema 校验器。
//!
//! 提供与 `schemas/sound-source.schema.json` 对应的反序列化结构 `SoundSourceConfig`，
//! 以及基于 `jsonschema` crate 的严格校验。Schema 文本在编译期通过 `include_str!`
//! 内嵌进二进制，保证 schema 随产物分发、无需运行时读取磁盘文件。

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;

use crate::error::{CoreError, Result};

/// 内嵌的标准音源 JSON Schema 文本。
///
/// 路径：`core/src/sources/schema.rs` 向上三级到 `/workspace`，
/// 再进入 `schemas/sound-source.schema.json`。
const SCHEMA_TEXT: &str = include_str!("../../../schemas/sound-source.schema.json");

/// 音源配置根结构，对应 `sound-source.schema.json` 顶层对象。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoundSourceConfig {
    /// 音源清单（id/name/version/author 等）。
    pub manifest: Manifest,
    /// 音源对外接口定义。
    pub endpoints: Endpoints,
    /// 鉴权配置（可选）。
    #[serde(default)]
    pub auth: Option<Auth>,
    /// 音源返回字段到标准字段的映射。
    pub field_mapping: FieldMapping,
    /// 分页配置（可选）。
    #[serde(default)]
    pub pagination: Option<Pagination>,
    /// 请求超时时间（毫秒，可选，默认 10000）。
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: f64,
}

/// 音源清单/元信息。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    /// 音源唯一标识。
    pub id: String,
    /// 展示名。
    pub name: String,
    /// 语义化版本号。
    pub version: String,
    /// 作者。
    pub author: String,
    /// 描述（可选）。
    #[serde(default)]
    pub description: Option<String>,
    /// 主页地址（可选）。
    #[serde(default)]
    pub homepage: Option<String>,
}

/// 音源对外接口定义。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoints {
    /// 搜索接口。
    pub search: Endpoint,
    /// 元数据接口。
    pub metadata: Endpoint,
    /// 播放 URL 接口。
    pub play_url: Endpoint,
    /// 歌词接口（可选）。
    #[serde(default)]
    pub lyric: Option<Endpoint>,
    /// 排行榜接口（可选）。
    #[serde(default)]
    pub leaderboards: Option<LeaderboardsEndpoint>,
}

/// 通用音源接口（search/metadata/playUrl/lyric 共用）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Endpoint {
    /// 接口 URL。
    pub url: String,
    /// HTTP 方法，默认 GET。
    #[serde(default)]
    pub method: HttpMethod,
    /// 关键词参数名，默认 "keyword"。
    #[serde(default = "default_query_param")]
    pub query_param: String,
    /// 页码参数名，默认 "page"。
    #[serde(default = "default_page_param")]
    pub page_param: String,
    /// 每页数量参数名，默认 "page_size"。
    #[serde(default = "default_page_size_param")]
    pub page_size_param: String,
    /// 自定义请求头（可选）。
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
    /// 响应类型，默认 json。
    #[serde(default)]
    pub response_type: ResponseType,
    /// 歌曲 id 参数名（metadata/playUrl/lyric 使用），默认 "id"。
    #[serde(default = "default_id_param")]
    pub id_param: String,
}

/// 排行榜接口（仅 url + headers）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LeaderboardsEndpoint {
    /// 接口 URL。
    pub url: String,
    /// 自定义请求头（可选）。
    #[serde(default)]
    pub headers: Option<HashMap<String, String>>,
}

/// HTTP 方法。
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
}

impl Default for HttpMethod {
    fn default() -> Self {
        HttpMethod::Get
    }
}

/// 响应类型。
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ResponseType {
    Json,
    Text,
}

impl Default for ResponseType {
    fn default() -> Self {
        ResponseType::Json
    }
}

/// 鉴权配置。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Auth {
    /// 鉴权方式，默认 none。
    #[serde(default)]
    pub r#type: AuthType,
    /// token 参数名（type 为 query/header 时必填）。
    #[serde(default)]
    pub token_param: Option<String>,
    /// token 值（可选，运行时也可注入）。
    #[serde(default)]
    pub token: Option<String>,
}

/// 鉴权方式。
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    None,
    Query,
    Header,
}

impl Default for AuthType {
    fn default() -> Self {
        AuthType::None
    }
}

/// 音源返回字段到标准字段的映射。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldMapping {
    /// 歌曲字段映射。
    pub song: SongMapping,
    /// 专辑字段映射。
    pub album: AlbumMapping,
    /// 艺术家字段映射。
    pub artist: ArtistMapping,
    /// 歌词字段映射。
    pub lyric: LyricMapping,
    /// 搜索结果字段映射（可选）。
    #[serde(default)]
    pub search_result: Option<SearchResultMapping>,
}

/// 歌曲字段映射，每个值为字段名或 JSON 指针。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SongMapping {
    /// 歌曲 id。
    pub id: String,
    /// 标题。
    pub title: String,
    /// 艺术家（数组或字符串→单元素）。
    #[serde(default)]
    pub artists: Option<String>,
    /// 专辑名。
    #[serde(default)]
    pub album: Option<String>,
    /// 封面 URL。
    #[serde(default)]
    pub cover_url: Option<String>,
    /// 时长（数值，单位毫秒）。
    #[serde(default)]
    pub duration_ms: Option<String>,
    /// 歌词 URL。
    #[serde(default)]
    pub lyric_url: Option<String>,
    /// 播放 URL。
    #[serde(default)]
    pub play_url: Option<String>,
}

/// 专辑字段映射。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AlbumMapping {
    /// 专辑 id。
    pub id: String,
    /// 专辑名。
    pub name: String,
    /// 艺术家。
    #[serde(default)]
    pub artists: Option<String>,
    /// 封面 URL。
    #[serde(default)]
    pub cover_url: Option<String>,
    /// 歌曲 id 列表（数组）。
    #[serde(default)]
    pub song_ids: Option<String>,
}

/// 艺术家字段映射。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArtistMapping {
    /// 艺术家 id。
    pub id: String,
    /// 艺术家名。
    pub name: String,
    /// 头像 URL。
    #[serde(default)]
    pub avatar_url: Option<String>,
    /// 歌曲 id 列表（数组）。
    #[serde(default)]
    pub song_ids: Option<String>,
}

/// 歌词字段映射。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricMapping {
    /// 歌词行映射，每项描述 timeMs 与 text 的字段名。
    pub lines: Vec<LyricLineMapping>,
    /// 统一时间字段名（可选）。
    #[serde(default)]
    pub time_field: Option<String>,
}

/// 歌词单行映射。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricLineMapping {
    /// 时间字段名（毫秒）。
    pub time_ms: String,
    /// 歌词文本字段名。
    pub text: String,
}

/// 搜索结果字段映射。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResultMapping {
    /// 总数字段名（数值）。
    #[serde(default)]
    pub total: Option<String>,
    /// 歌曲列表字段名（数组）。
    #[serde(default)]
    pub songs: Option<String>,
    /// 专辑列表字段名（数组）。
    #[serde(default)]
    pub albums: Option<String>,
    /// 艺术家列表字段名（数组）。
    #[serde(default)]
    pub artists: Option<String>,
}

/// 分页配置。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pagination {
    /// 起始页码，默认 0。
    #[serde(default = "default_page_start")]
    pub page_start: f64,
    /// 默认每页数量，默认 20。
    #[serde(default = "default_page_size_default")]
    pub page_size_default: f64,
}

impl SoundSourceConfig {
    /// 从 JSON 字符串反序列化为配置结构。
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(CoreError::from)
    }

    /// 严格校验：基于内嵌 JSON Schema 校验任意 `serde_json::Value`。
    ///
    /// 返回 `Ok(())` 表示完全符合 schema；否则返回带详细路径的 `Err`。
    pub fn validate_strict(json: &Value) -> Result<()> {
        let schema: Value = serde_json::from_str(SCHEMA_TEXT)
            .map_err(|e| CoreError::Schema(format!("加载内嵌 schema 失败: {e}")))?;
        let validator = jsonschema::validator_for(&schema)
            .map_err(|e| CoreError::Schema(format!("编译 schema 失败: {e}")))?;

        if validator.is_valid(json) {
            return Ok(());
        }

        // 先把错误格式化为拥有所有权的字符串，与 validator 的借用解耦。
        let mut entries: Vec<String> = Vec::new();
        if let Err(errors) = validator.validate(json) {
            for err in errors {
                let path = err.instance_path.to_string();
                let path = if path.is_empty() {
                    "(root)".to_string()
                } else {
                    path
                };
                entries.push(format!("路径 {path}: {err}"));
            }
        }

        let mut msg = String::from("音源配置校验失败:");
        for entry in entries {
            msg.push_str("\n  - ");
            msg.push_str(&entry);
        }
        Err(CoreError::Schema(msg))
    }
}

/// 对已反序列化的配置做额外语义校验。
///
/// 例如：`auth.type` 不为 `none` 时，`tokenParam` 必填。
pub fn validate_config(config: &SoundSourceConfig) -> Result<()> {
    if let Some(auth) = &config.auth {
        match auth.r#type {
            AuthType::None => {}
            AuthType::Query | AuthType::Header => {
                if auth.token_param.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    return Err(CoreError::Schema(format!(
                        "auth.type 为 \"{}\" 时必须提供非空 tokenParam",
                        match auth.r#type {
                            AuthType::Query => "query",
                            AuthType::Header => "header",
                            AuthType::None => "none",
                        }
                    )));
                }
            }
        }
    }
    Ok(())
}

fn default_query_param() -> String {
    "keyword".to_string()
}
fn default_page_param() -> String {
    "page".to_string()
}
fn default_page_size_param() -> String {
    "page_size".to_string()
}
fn default_id_param() -> String {
    "id".to_string()
}
fn default_page_start() -> f64 {
    0.0
}
fn default_page_size_default() -> f64 {
    20.0
}
fn default_timeout_ms() -> f64 {
    10000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_config_passes_validation() {
        let json = serde_json::json!({
            "manifest": {
                "id": "demo-source",
                "name": "Demo Source",
                "version": "1.0.0",
                "author": "tester",
                "description": "演示音源",
                "homepage": "https://example.com"
            },
            "endpoints": {
                "search": { "url": "https://example.com/api/search" },
                "metadata": { "url": "https://example.com/api/metadata" },
                "playUrl": { "url": "https://example.com/api/play" },
                "lyric": { "url": "https://example.com/api/lyric" },
                "leaderboards": {
                    "url": "https://example.com/api/leaderboards",
                    "headers": { "X-Source": "demo" }
                }
            },
            "auth": { "type": "none" },
            "fieldMapping": {
                "song": {
                    "id": "id",
                    "title": "name",
                    "artists": "artists",
                    "durationMs": "duration"
                },
                "album": { "id": "albumId", "name": "albumName" },
                "artist": { "id": "artistId", "name": "artistName" },
                "lyric": {
                    "lines": [{ "timeMs": "time", "text": "content" }]
                },
                "searchResult": { "total": "total", "songs": "songs" }
            },
            "pagination": { "pageStart": 0, "pageSizeDefault": 20 },
            "timeoutMs": 10000
        });

        // 严格 schema 校验应通过
        assert!(
            SoundSourceConfig::validate_strict(&json).is_ok(),
            "合法配置应通过 schema 校验"
        );

        // 反序列化应成功
        let config = SoundSourceConfig::from_json(&json.to_string())
            .expect("合法配置应能反序列化");
        assert_eq!(config.manifest.id, "demo-source");
        assert_eq!(config.endpoints.search.url, "https://example.com/api/search");
        // 末提供 method，应取默认 GET
        assert_eq!(config.endpoints.search.method, HttpMethod::Get);
        // 末提供 queryParam，应取默认 keyword
        assert_eq!(config.endpoints.search.query_param, "keyword");

        // 语义校验（auth.type=none，无需 tokenParam）应通过
        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn missing_manifest_returns_error() {
        let json = serde_json::json!({
            "endpoints": {
                "search": { "url": "https://example.com/api/search" },
                "metadata": { "url": "https://example.com/api/metadata" },
                "playUrl": { "url": "https://example.com/api/play" }
            },
            "fieldMapping": {
                "song": { "id": "id", "title": "name" },
                "album": { "id": "id", "name": "name" },
                "artist": { "id": "id", "name": "name" },
                "lyric": {
                    "lines": [{ "timeMs": "t", "text": "x" }]
                }
            }
        });

        let result = SoundSourceConfig::validate_strict(&json);
        assert!(
            result.is_err(),
            "缺少 manifest 的配置应校验失败"
        );
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("manifest"),
            "错误信息应指出 manifest 相关问题，实际: {msg}"
        );
    }
}
