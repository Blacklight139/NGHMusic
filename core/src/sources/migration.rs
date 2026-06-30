//! 社区音源迁移/适配方案：将社区格式转换为标准 [SoundSourceConfig]。
//!
//! 当前提供"宽松解析"适配：兼容常见社区字段的多种命名。

use crate::sources::schema::{SoundSourceConfig, SourceManifest, SourceFieldMapping, SourceEndpoints, SourceEndpoint, HttpMethod, AuthConfig};
use crate::{CoreError, CoreResult};

/// 迁移结果
#[derive(Debug, Clone, serde::Serialize)]
pub struct MigrationResult {
    pub migrated: bool,
    pub warnings: Vec<String>,
    pub config: SoundSourceConfig,
}

/// 尝试迁移：先用标准解析，失败则尝试社区适配层
pub fn migrate(json: &str) -> CoreResult<MigrationResult> {
    // 1. 先按标准解析
    match SoundSourceConfig::from_json(json) {
        Ok(cfg) => Ok(MigrationResult {
            migrated: false,
            warnings: vec![],
            config: cfg,
        }),
        Err(_e) => {
            // 2. 社区适配
            let v: serde_json::Value = serde_json::from_str(json)
                .map_err(|e| CoreError::SchemaValidation(format!("非合法 JSON: {e}")))?;
            let cfg = convert_community(&v)?;
            Ok(MigrationResult {
                migrated: true,
                warnings: vec!["已由社区格式自动转换".into()],
                config: cfg,
            })
        }
    }
}

/// 社区格式转换：宽松提取已知字段
fn convert_community(v: &serde_json::Value) -> CoreResult<SoundSourceConfig> {
    // 常见社区字段名映射
    let manifest = SourceManifest {
        id: pick_str(v, &["id", "sourceId", "source_id", "name"])
            .unwrap_or_else(|| "community_source".into()),
        name: pick_str(v, &["name", "title", "sourceName"])
            .unwrap_or_else(|| "社区音源".into()),
        author: pick_str(v, &["author", "authorName"]),
        version: pick_str(v, &["version", "v"]),
        description: pick_str(v, &["description", "desc"]),
        homepage: pick_str(v, &["homepage", "url", "base"]),
    };

    let endpoints = SourceEndpoints {
        search: convert_endpoint(v, &["search", "searchUrl", "search_url"]),
        metadata: convert_endpoint(v, &["metadata", "song", "songUrl", "detail"]),
        play: convert_endpoint(v, &["play", "playUrl", "play_url", "stream"]),
        lyric: convert_endpoint(v, &["lyric", "lyricUrl", "lyric_url", "lrc"]),
        ranking: convert_endpoint(v, &["ranking", "toplist", "rank"]),
    };

    let auth = pick_str(v, &["token", "authToken", "auth_token"])
        .map(|t| AuthConfig {
            auth_type: "header".into(),
            name: "Authorization".into(),
            token: Some(t),
        });

    let cfg = SoundSourceConfig {
        schema_version: "1.0".into(),
        manifest,
        auth,
        field_mapping: SourceFieldMapping::default(),
        endpoints,
    };
    cfg.validate()?;
    Ok(cfg)
}

fn convert_endpoint(v: &serde_json::Value, keys: &[&str]) -> Option<SourceEndpoint> {
    // 可能是字符串 URL，也可能是对象
    for k in keys {
        if let Some(val) = v.get(*k) {
            if let Some(s) = val.as_str() {
                if let Ok(_u) = url::Url::parse(s) {
                    return Some(SourceEndpoint {
                        method: HttpMethod::Get,
                        url: s.to_string(),
                        params: vec![],
                        headers: vec![],
                        body: None,
                        data_path: None,
                    });
                }
            } else if val.is_object() {
                let url = val.get("url").and_then(|x| x.as_str())?;
                let method = val.get("method").and_then(|x| x.as_str())
                    .map(|s| s.to_uppercase())
                    .unwrap_or_else(|| "GET".into());
                let method = match method.as_str() {
                    "POST" => HttpMethod::Post,
                    _ => HttpMethod::Get,
                };
                let params = collect_pairs(val, "params");
                let headers = collect_pairs(val, "headers");
                let body = val.get("body").and_then(|x| x.as_str()).map(String::from);
                let data_path = val.get("dataPath").or_else(|| val.get("data_path"))
                    .and_then(|x| x.as_str()).map(String::from);
                return Some(SourceEndpoint {
                    method,
                    url: url.to_string(),
                    params,
                    headers,
                    body,
                    data_path,
                });
            }
        }
    }
    None
}

fn collect_pairs(v: &serde_json::Value, key: &str) -> Vec<(String, String)> {
    v.get(key)
        .and_then(|x| x.as_object())
        .map(|m| {
            m.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}

fn pick_str(v: &serde_json::Value, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(s) = v.get(*k).and_then(|x| x.as_str()) {
            return Some(s.to_string());
        }
    }
    None
}
