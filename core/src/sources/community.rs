//! 社区音源迁移/适配层。
//!
//! 将不同社区维护的异构音源 JSON 配置（如 "sources" 数组格式、扁平 endpoint 格式）
//! 自动适配为我方标准 `SoundSourceConfig` 对应的 JSON，便于上层统一加载与校验。
//!
//! 支持三种识别：
//! - 标准格式（同时含 manifest + endpoints + fieldMapping）：原样返回。
//! - 社区格式 A（sources 数组）：取首个 source，name/url/type 映射到标准字段。
//! - 社区格式 B（扁平 endpoint）：name/search_url/song_url 映射到标准字段。

use crate::error::{CoreError, Result};
use serde_json::{json, Value};

/// 适配结果：包含转换后的标准配置、识别到的源格式名与迁移过程中的警告列表。
#[derive(Debug, Clone)]
pub struct AdaptResult {
    /// 转换后的标准 `SoundSourceConfig` JSON（可被 `schema::validate_strict` 校验）。
    pub config: Value,
    /// 识别到的源格式名：`"standard"` / `"community-a"` / `"community-b"`。
    pub source_format: String,
    /// 迁移过程中的警告（如某些字段无法映射、某些接口缺失使用占位 URL）。
    pub warnings: Vec<String>,
}

/// 将原始社区音源 JSON 适配为标准 `SoundSourceConfig` 对应的 JSON。
///
/// 自动识别标准格式、社区格式 A 与社区格式 B 并转换为标准结构；
/// 无法识别时返回 `Err(CoreError::Schema(...))`。
pub fn adapt(raw: &Value) -> Result<Value> {
    let report = adapt_with_report(raw)?;
    Ok(report.config)
}

/// 适配并返回迁移报告（源格式名 + 警告列表）。
///
/// 失败时返回 `Err`，调用方可据此提示「导入失败：原因」。
pub fn adapt_with_report(raw: &Value) -> Result<AdaptResult> {
    // 仅识别 JSON 对象，其余类型直接判定为无法识别
    let obj = raw
        .as_object()
        .ok_or_else(|| CoreError::Schema("无法识别的社区音源格式".into()))?;

    // 1. 已是标准格式：同时含 manifest + endpoints + fieldMapping 三个顶层字段，原样返回
    if obj.contains_key("manifest")
        && obj.contains_key("endpoints")
        && obj.contains_key("fieldMapping")
    {
        return Ok(AdaptResult {
            config: raw.clone(),
            source_format: "standard".to_string(),
            warnings: Vec::new(),
        });
    }

    // 2. 社区格式 A：{ "sources": [ { "name":..., "url":..., "type":... } ] }
    if let Some(srcs) = obj.get("sources").and_then(|v| v.as_array()) {
        if !srcs.is_empty() {
            return adapt_community_a(srcs);
        }
    }

    // 3. 社区格式 B：扁平 endpoint 格式，含 search_url / song_url 等字段
    if pick_str(obj, &["search_url", "searchUrl", "search.uri"]).is_some()
        || pick_str(obj, &["song_url", "songUrl", "song.uri", "play_url", "playUrl"]).is_some()
    {
        return adapt_community_b(obj);
    }

    // 其余情况无法识别
    Err(CoreError::Schema("无法识别的社区音源格式".into()))
}

/// 社区格式 A 适配：取 sources 数组首个元素，name/url/type 映射到标准字段。
fn adapt_community_a(srcs: &[Value]) -> Result<AdaptResult> {
    let first = srcs[0]
        .as_object()
        .ok_or_else(|| CoreError::Schema("社区格式 A 的 sources[0] 不是对象".into()))?;

    let mut warnings = Vec::new();

    // name → manifest.name（必填）
    let name = match pick_str(first, &["name", "title"]) {
        Some(n) if !n.is_empty() => n,
        _ => return Err(CoreError::Schema("社区格式 A 缺少 name 字段".into())),
    };

    // url → endpoints.search.url（必填）
    let search_url = match pick_str(first, &["url", "search_url", "searchUrl", "search.uri"]) {
        Some(u) if !u.is_empty() => u,
        _ => return Err(CoreError::Schema("社区格式 A 缺少 url 字段".into())),
    };

    // type → manifest.id（slug 化）；缺失或为空时回退到 slug(name)
    let id = {
        let raw_type = pick_str(first, &["type", "kind", "category"]);
        let base = raw_type
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or(&name);
        slugify(base)
    };

    // 社区格式 A 通常仅提供搜索接口，metadata/playUrl 缺失，使用占位 URL 并给出警告
    const PLACEHOLDER: &str = "https://example.com/unsupported";
    warnings.push("社区格式 A 未提供 metadata/playUrl 接口，已使用占位 URL".into());

    // sources 含多个元素时提示仅取首个
    if srcs.len() > 1 {
        warnings.push(format!("社区格式 A 含 {} 个 source，仅取首个", srcs.len()));
    }

    // 检测 source 对象中的未知字段
    let known_a = [
        "name",
        "title",
        "url",
        "search_url",
        "searchUrl",
        "search.uri",
        "type",
        "kind",
        "category",
    ];
    for key in first.keys() {
        if !known_a.contains(&key.as_str()) {
            warnings.push(format!("社区格式 A 存在未映射字段: {key}"));
        }
    }

    let config = build_standard_config(&id, &name, &search_url, PLACEHOLDER, PLACEHOLDER);

    Ok(AdaptResult {
        config,
        source_format: "community-a".to_string(),
        warnings,
    })
}

/// 社区格式 B 适配：扁平 endpoint 字段映射到标准结构。
fn adapt_community_b(obj: &serde_json::Map<String, Value>) -> Result<AdaptResult> {
    let mut warnings = Vec::new();
    const PLACEHOLDER: &str = "https://example.com/unsupported";

    // name → manifest.name（必填）
    let name = match pick_str(obj, &["name", "title"]) {
        Some(n) if !n.is_empty() => n,
        _ => return Err(CoreError::Schema("社区格式 B 缺少 name 字段".into())),
    };

    // search_url → endpoints.search.url
    let search_url = match pick_str(obj, &["search_url", "searchUrl", "search.uri"]) {
        Some(u) if !u.is_empty() => u,
        _ => {
            warnings.push("社区格式 B 缺少 search_url，已使用占位 URL".into());
            PLACEHOLDER.to_string()
        }
    };

    // song_url → endpoints.playUrl.url
    let play_url = match pick_str(obj, &["song_url", "songUrl", "song.uri", "play_url", "playUrl"]) {
        Some(u) if !u.is_empty() => u,
        _ => {
            warnings.push("社区格式 B 缺少 song_url，已使用占位 URL".into());
            PLACEHOLDER.to_string()
        }
    };

    // metadata 通常缺失，尝试映射；缺失则占位 + 警告
    let metadata_url = match pick_str(obj, &["metadata_url", "metadataUrl", "metadata.uri"]) {
        Some(u) if !u.is_empty() => u,
        _ => {
            warnings.push("社区格式 B 未提供 metadata 接口，已使用占位 URL".into());
            PLACEHOLDER.to_string()
        }
    };

    // manifest.id = slug(name)
    let id = slugify(&name);

    // 检测未知字段
    let known_b = [
        "name",
        "title",
        "search_url",
        "searchUrl",
        "search.uri",
        "song_url",
        "songUrl",
        "song.uri",
        "play_url",
        "playUrl",
        "metadata_url",
        "metadataUrl",
        "metadata.uri",
    ];
    for key in obj.keys() {
        if !known_b.contains(&key.as_str()) {
            warnings.push(format!("社区格式 B 存在未映射字段: {key}"));
        }
    }

    let config = build_standard_config(&id, &name, &search_url, &metadata_url, &play_url);

    Ok(AdaptResult {
        config,
        source_format: "community-b".to_string(),
        warnings,
    })
}

/// 用核心字段拼装一个可通过 schema 严格校验的标准 `SoundSourceConfig` JSON。
///
/// 未提供的接口（metadata/playUrl 等）由调用方决定是否使用占位 URL。
fn build_standard_config(
    id: &str,
    name: &str,
    search_url: &str,
    metadata_url: &str,
    play_url: &str,
) -> Value {
    json!({
        "manifest": {
            "id": id,
            "name": name,
            "version": "0.0.0",
            "author": "community"
        },
        "endpoints": {
            "search": { "url": search_url },
            "metadata": { "url": metadata_url },
            "playUrl": { "url": play_url }
        },
        "auth": { "type": "none" },
        "fieldMapping": {
            "song": { "id": "id", "title": "name" },
            "album": { "id": "albumId", "name": "albumName" },
            "artist": { "id": "artistId", "name": "artistName" },
            "lyric": { "lines": [{ "timeMs": "time", "text": "content" }] }
        },
        "pagination": { "pageStart": 0, "pageSizeDefault": 20 },
        "timeoutMs": 10000
    })
}

/// 在对象中按候选键名列表依次查找，返回首个命中的字符串值。
///
/// 兼容 snake_case / camelCase / 点号分隔（如 `search.uri`）等多种命名风格，
/// 用于健壮处理社区格式多变的字段名。
fn pick_str(obj: &serde_json::Map<String, Value>, keys: &[&str]) -> Option<String> {
    for k in keys {
        if let Some(v) = obj.get(*k) {
            if let Some(s) = v.as_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

/// 将任意名称 slug 化：转小写、非字母数字替换为 "-"、去重 "-"、截断到 64 字符。
///
/// 注意：为满足标准 schema 的 `id` 约束（`^[a-z0-9-]{1,64}$`），
/// 此处按 ASCII 字母数字处理，非 ASCII 字符（如中文）视为分隔符；
/// 若结果为空（如纯符号/纯中文名称），回退为 `"source"` 保证非空合法。
fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    // 起始视为"前一个已是分隔符"，避免开头出现 "-"
    let mut prev_dash = true;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            out.push('-');
            prev_dash = true;
        }
    }
    // 去掉尾部可能的 "-"
    while out.ends_with('-') {
        out.pop();
    }
    // 截断到 64 字符（按 char 边界安全截断）
    if out.chars().count() > 64 {
        out = out.chars().take(64).collect();
        while out.ends_with('-') {
            out.pop();
        }
    }
    // 空字符串兜底，保证符合 schema pattern
    if out.is_empty() {
        out.push_str("source");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 标准格式原样返回，source_format="standard"，无警告。
    #[test]
    fn standard_format_returns_as_is() {
        let raw = json!({
            "manifest": {
                "id": "demo-source",
                "name": "Demo Source",
                "version": "1.0.0",
                "author": "tester"
            },
            "endpoints": {
                "search": { "url": "https://example.com/api/search" },
                "metadata": { "url": "https://example.com/api/metadata" },
                "playUrl": { "url": "https://example.com/api/play" }
            },
            "fieldMapping": {
                "song": { "id": "id", "title": "name" },
                "album": { "id": "id", "name": "name" },
                "artist": { "id": "id", "name": "name" },
                "lyric": { "lines": [{ "timeMs": "t", "text": "x" }] }
            }
        });

        let report = adapt_with_report(&raw).expect("标准格式应原样返回");
        assert_eq!(report.source_format, "standard");
        assert!(report.warnings.is_empty(), "标准格式不应有警告");
        // 原样返回，内容应等价
        assert_eq!(report.config, raw);

        // adapt 应与 adapt_with_report 的 config 一致
        let adapted = adapt(&raw).expect("adapt 标准格式应成功");
        assert_eq!(adapted, raw);
    }

    /// 社区格式 A 转换：断言 endpoints.search.url 正确、manifest.id 为 slug(type)。
    #[test]
    fn community_format_a_converts() {
        let raw = json!({
            "sources": [
                { "name": "My Music", "url": "https://music.example.com/search", "type": "music" }
            ]
        });

        let report = adapt_with_report(&raw).expect("社区格式 A 应转换成功");
        assert_eq!(report.source_format, "community-a");

        let manifest = report.config.get("manifest").expect("应含 manifest");
        assert_eq!(manifest["name"], json!("My Music"));
        // type="music" slug 化后为 "music"
        assert_eq!(manifest["id"], json!("music"));

        let endpoints = report.config.get("endpoints").expect("应含 endpoints");
        assert_eq!(
            endpoints["search"]["url"],
            json!("https://music.example.com/search")
        );

        // 转换结果应能通过 schema 严格校验
        assert!(
            crate::sources::schema::SoundSourceConfig::validate_strict(&report.config).is_ok(),
            "社区格式 A 转换结果应通过 schema 校验"
        );
    }

    /// 社区格式 B 转换：断言 endpoints.playUrl.url 正确。
    #[test]
    fn community_format_b_converts() {
        let raw = json!({
            "name": "网易云",
            "search_url": "https://music.example.com/search",
            "song_url": "https://music.example.com/song"
        });

        let report = adapt_with_report(&raw).expect("社区格式 B 应转换成功");
        assert_eq!(report.source_format, "community-b");

        let manifest = report.config.get("manifest").expect("应含 manifest");
        assert_eq!(manifest["name"], json!("网易云"));
        // manifest.id = slug(name)；中文名回退为 "source" 以满足 schema pattern
        assert_eq!(manifest["id"], json!("source"));

        let endpoints = report.config.get("endpoints").expect("应含 endpoints");
        assert_eq!(
            endpoints["search"]["url"],
            json!("https://music.example.com/search")
        );
        assert_eq!(
            endpoints["playUrl"]["url"],
            json!("https://music.example.com/song")
        );

        // 转换结果应能通过 schema 严格校验
        assert!(
            crate::sources::schema::SoundSourceConfig::validate_strict(&report.config).is_ok(),
            "社区格式 B 转换结果应通过 schema 校验"
        );
    }

    /// 无法识别的格式返回 Err。
    #[test]
    fn unknown_format_returns_error() {
        let raw = json!({ "foo": "bar", "baz": 123 });
        let result = adapt_with_report(&raw);
        assert!(result.is_err(), "无法识别的格式应返回 Err");
        let msg = format!("{}", result.unwrap_err());
        assert!(
            msg.contains("无法识别"),
            "错误信息应包含「无法识别」，实际: {msg}"
        );

        // adapt 同样返回 Err
        let result2 = adapt(&json!({ "hello": "world" }));
        assert!(result2.is_err());
    }

    /// adapt_with_report 返回 warnings（未映射字段）。
    #[test]
    fn adapt_with_report_returns_warnings() {
        let raw = json!({
            "name": "测试音源",
            "search_url": "https://music.example.com/search",
            "song_url": "https://music.example.com/song",
            "extra_field": "ignored",
            "another_extra": 42
        });

        let report = adapt_with_report(&raw).expect("应转换成功");
        assert!(!report.warnings.is_empty(), "应至少有一条警告");
        let joined = report.warnings.join("\n");
        assert!(
            joined.contains("extra_field") || joined.contains("another_extra"),
            "警告应包含未映射字段名，实际: {joined}"
        );
    }

    /// 额外：社区格式 A 中 type 缺失时，manifest.id 回退到 slug(name)。
    #[test]
    fn community_format_a_falls_back_to_name_slug() {
        let raw = json!({
            "sources": [
                { "name": "My Source", "url": "https://music.example.com/search" }
            ]
        });

        let report = adapt_with_report(&raw).expect("社区格式 A 应转换成功");
        // type 缺失，回退到 slug(name) = "my-source"
        assert_eq!(report.config["manifest"]["id"], json!("my-source"));
    }

    /// 额外：camelCase 字段名兼容（社区格式 B 用 searchUrl/songUrl）。
    #[test]
    fn community_format_b_camelcase_keys() {
        let raw = json!({
            "name": "CamelCase Source",
            "searchUrl": "https://music.example.com/search",
            "songUrl": "https://music.example.com/song"
        });

        let report = adapt_with_report(&raw).expect("camelCase 字段应被识别");
        assert_eq!(report.source_format, "community-b");
        assert_eq!(
            report.config["endpoints"]["search"]["url"],
            json!("https://music.example.com/search")
        );
        assert_eq!(
            report.config["endpoints"]["playUrl"]["url"],
            json!("https://music.example.com/song")
        );
    }
}
