//! JSON 配置驱动的在线音源引擎。
//!
//! 基于 [`SoundSourceConfig`](crate::sources::schema::SoundSourceConfig) 描述的
//! 接口/字段映射/鉴权信息，通过 reqwest 客户端发起 HTTP 请求并解析响应，
//! 将异构音源适配为统一的 [`Source`](crate::sources::Source) trait 实现。
//!
//! 核心职责：
//! - 按 endpoint 配置发起 GET/POST 请求，自动注入分页参数与鉴权 token。
//! - 通过 `fieldMapping` 把音源原始 JSON 字段映射为标准
//!   [`Song`](crate::models::Song)/[`Album`](crate::models::Album)/
//!   [`Artist`](crate::models::Artist) 等结构。
//! - 字段映射支持简单字段名与点路径（`a.b.c`），缺失字段返回 `None`/默认值，不会 panic。

use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::{CoreError, Result};
use crate::models::{Album, Artist, Leaderboard, Lyric, LyricLine, SearchResult, Song, SongOrigin};
use crate::sources::schema::{
    AlbumMapping, ArtistMapping, AuthType, Endpoint, HttpMethod, ResponseType, SongMapping,
    SoundSourceConfig,
};
use crate::sources::Source;

/// 基于 `SoundSourceConfig` 的在线音源。
pub struct JsonSource {
    /// 音源配置
    pub config: SoundSourceConfig,
    /// HTTP 客户端
    pub client: reqwest::Client,
    /// 是否启用
    pub enabled: bool,
    /// 优先级（数值越大越靠前）
    pub priority: i32,
}

impl JsonSource {
    /// 构造音源：根据 `config.timeout_ms` 构建 reqwest 客户端，默认 10s。
    ///
    /// 构造期会校验所有 endpoint URL 是否指向允许的外网地址，
    /// 阻断恶意音源 JSON 配置通过指定内网地址（如 169.254.169.254 云元数据、
    /// 127.0.0.1 本机、192.168.x.x 私网）发起 SSRF 攻击。
    pub fn new(config: SoundSourceConfig) -> Result<Self> {
        // SSRF 防御：在导入期校验所有 endpoint URL，恶意源在导入时即被拒绝。
        validate_endpoint_url(&config.endpoints.search.url)?;
        validate_endpoint_url(&config.endpoints.metadata.url)?;
        validate_endpoint_url(&config.endpoints.play_url.url)?;
        if let Some(ep) = &config.endpoints.lyric {
            validate_endpoint_url(&ep.url)?;
        }
        if let Some(ep) = &config.endpoints.leaderboards {
            validate_endpoint_url(&ep.url)?;
        }

        let timeout_ms = if config.timeout_ms > 0.0 {
            config.timeout_ms
        } else {
            10000.0
        };
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(timeout_ms as u64))
            .build()?;
        Ok(Self {
            config,
            client,
            enabled: true,
            priority: 0,
        })
    }

    /// 按 endpoint 配置发起 HTTP 请求，返回解析后的 JSON 值。
    ///
    /// `params` 为 `(参数名, 值)` 列表：GET 时进入 query，POST 时进入 JSON body。
    /// 自动注入自定义请求头与鉴权 token（query 模式注入 query；header 模式注入 header）。
    async fn request_json(
        &self,
        endpoint: &Endpoint,
        params: Vec<(&str, String)>,
    ) -> Result<Value> {
        let mut req = match endpoint.method {
            HttpMethod::Get => self.client.get(&endpoint.url),
            HttpMethod::Post => self.client.post(&endpoint.url),
        };

        // 自定义请求头
        if let Some(headers) = &endpoint.headers {
            for (k, v) in headers {
                req = req.header(k.as_str(), v.as_str());
            }
        }

        // header 鉴权
        if let Some(auth) = &self.config.auth {
            if auth.r#type == AuthType::Header {
                if let (Some(param), Some(token)) = (&auth.token_param, &auth.token) {
                    req = req.header(param.as_str(), token.as_str());
                }
            }
        }

        // 计算需要在 query/body 中携带的鉴权 token
        let auth_query: Option<(String, String)> = match &self.config.auth {
            Some(auth) if auth.r#type == AuthType::Query => {
                if let (Some(param), Some(token)) = (&auth.token_param, &auth.token) {
                    Some((param.clone(), token.clone()))
                } else {
                    None
                }
            }
            _ => None,
        };

        let req = match endpoint.method {
            HttpMethod::Get => {
                let mut q: Vec<(&str, &str)> =
                    params.iter().map(|(k, v)| (*k, v.as_str())).collect();
                if let Some((k, v)) = &auth_query {
                    q.push((k.as_str(), v.as_str()));
                }
                req.query(&q)
            }
            HttpMethod::Post => {
                let mut body = serde_json::Map::new();
                for (k, v) in &params {
                    body.insert(k.to_string(), Value::String(v.clone()));
                }
                if let Some((k, v)) = &auth_query {
                    body.insert(k.clone(), Value::String(v.clone()));
                }
                req.json(&Value::Object(body))
            }
        };

        let response = req.send().await?;
        let response = response.error_for_status()?;

        match endpoint.response_type {
            ResponseType::Json => {
                let text = response.text().await?;
                let value: Value = serde_json::from_str(&text)?;
                Ok(value)
            }
            ResponseType::Text => {
                let text = response.text().await?;
                Ok(Value::String(text))
            }
        }
    }

    /// 抓取排行榜（GET 接口，仅 url + headers）。
    async fn request_leaderboards(&self) -> Result<Value> {
        let endpoint = self
            .config
            .endpoints
            .leaderboards
            .as_ref()
            .ok_or_else(|| CoreError::Source("音源未配置 leaderboards 接口".into()))?;

        let mut req = self.client.get(&endpoint.url);
        if let Some(headers) = &endpoint.headers {
            for (k, v) in headers {
                req = req.header(k.as_str(), v.as_str());
            }
        }
        // header 鉴权
        if let Some(auth) = &self.config.auth {
            if auth.r#type == AuthType::Header {
                if let (Some(param), Some(token)) = (&auth.token_param, &auth.token) {
                    req = req.header(param.as_str(), token.as_str());
                }
            }
        }

        let response = req.send().await?;
        let response = response.error_for_status()?;
        let text = response.text().await?;
        let value: Value = serde_json::from_str(&text)?;
        Ok(value)
    }
}

#[async_trait]
impl Source for JsonSource {
    fn id(&self) -> &str {
        &self.config.manifest.id
    }

    fn name(&self) -> &str {
        &self.config.manifest.name
    }

    async fn search(&self, keyword: &str, page: u32, page_size: u32) -> Result<SearchResult> {
        let endpoint = &self.config.endpoints.search;
        let params = vec![
            (endpoint.query_param.as_str(), keyword.to_string()),
            (endpoint.page_param.as_str(), page.to_string()),
            (endpoint.page_size_param.as_str(), page_size.to_string()),
        ];
        let response = self.request_json(endpoint, params).await?;

        let sr_mapping = self.config.field_mapping.search_result.as_ref();
        let song_mapping = &self.config.field_mapping.song;
        let album_mapping = &self.config.field_mapping.album;
        let artist_mapping = &self.config.field_mapping.artist;
        let source_id = &self.config.manifest.id;

        let total = sr_mapping
            .and_then(|m| m.total.as_deref())
            .and_then(|field| extract_field(&response, field))
            .and_then(value_to_u64)
            .unwrap_or(0);

        let songs: Vec<Song> = sr_mapping
            .and_then(|m| m.songs.as_deref())
            .and_then(|field| extract_field(&response, field))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| map_song(item, song_mapping, source_id))
                    .collect()
            })
            .unwrap_or_default();

        let albums: Vec<Album> = sr_mapping
            .and_then(|m| m.albums.as_deref())
            .and_then(|field| extract_field(&response, field))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| map_album(item, album_mapping, source_id))
                    .collect()
            })
            .unwrap_or_default();

        let artists: Vec<Artist> = sr_mapping
            .and_then(|m| m.artists.as_deref())
            .and_then(|field| extract_field(&response, field))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| map_artist(item, artist_mapping, source_id))
                    .collect()
            })
            .unwrap_or_default();

        Ok(SearchResult {
            keyword: keyword.to_string(),
            songs,
            albums,
            artists,
            total,
            page,
            page_size,
        })
    }

    async fn get_metadata(&self, song_id: &str) -> Result<Song> {
        let endpoint = &self.config.endpoints.metadata;
        let params = vec![(endpoint.id_param.as_str(), song_id.to_string())];
        let response = self.request_json(endpoint, params).await?;
        map_song(&response, &self.config.field_mapping.song, &self.config.manifest.id)
            .ok_or_else(|| CoreError::Source(format!("无法从元数据响应映射歌曲: {song_id}")))
    }

    async fn get_play_url(&self, song_id: &str) -> Result<String> {
        let endpoint = &self.config.endpoints.play_url;
        let params = vec![(endpoint.id_param.as_str(), song_id.to_string())];
        let response = self.request_json(endpoint, params).await?;

        // 优先使用 song.playUrl 字段映射提取
        if let Some(field) = self.config.field_mapping.song.play_url.as_deref() {
            if let Some(v) = extract_field(&response, field) {
                if let Some(s) = v.as_str() {
                    return Ok(s.to_string());
                }
            }
        }
        // 退化为整体字符串
        response
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| CoreError::Source("无法从响应中提取播放 URL".into()))
    }

    async fn get_lyric(&self, song_id: &str) -> Result<Lyric> {
        let endpoint = match &self.config.endpoints.lyric {
            Some(e) => e,
            None => {
                return Err(CoreError::NotFound(format!(
                    "音源 {} 未配置歌词接口",
                    self.config.manifest.id
                )))
            }
        };
        let params = vec![(endpoint.id_param.as_str(), song_id.to_string())];
        let response = self.request_json(endpoint, params).await?;

        let lyric_mapping = &self.config.field_mapping.lyric;
        // 找到行数组：响应本身是数组，或响应对象含 lines 字段
        let lines_arr: &Vec<Value> = match response.as_array() {
            Some(arr) => arr,
            None => match extract_field(&response, "lines").and_then(|v| v.as_array()) {
                Some(arr) => arr,
                None => {
                    return Err(CoreError::Source(
                        "歌词响应格式错误：应为数组或含 lines 字段".into(),
                    ))
                }
            },
        };

        let line_mapping = lyric_mapping.lines.first();
        let mut lines = Vec::with_capacity(lines_arr.len());
        for line_value in lines_arr {
            let time_ms = line_mapping
                .and_then(|m| extract_field(line_value, &m.time_ms))
                .and_then(value_to_u64);
            let text = line_mapping
                .and_then(|m| extract_field(line_value, &m.text))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            lines.push(LyricLine { time_ms, text });
        }

        Ok(Lyric {
            lines,
            translation: None,
        })
    }

    async fn get_leaderboards(&self) -> Result<Vec<Leaderboard>> {
        if self.config.endpoints.leaderboards.is_none() {
            return Ok(Vec::new());
        }
        let response = self.request_leaderboards().await?;

        let arr = match response.as_array() {
            Some(a) => a,
            None => return Ok(Vec::new()),
        };

        let song_mapping = &self.config.field_mapping.song;
        let source_id = &self.config.manifest.id;
        let mut result = Vec::with_capacity(arr.len());
        for item in arr {
            let id = item
                .get("id")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            if id.is_empty() {
                continue;
            }
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_default();
            let cover_url = item
                .get("coverUrl")
                .or_else(|| item.get("cover_url"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let songs: Vec<Song> = item
                .get("songs")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| map_song(s, song_mapping, source_id))
                        .collect()
                })
                .unwrap_or_default();

            result.push(Leaderboard {
                id,
                source_id: source_id.clone(),
                name,
                cover_url,
                songs,
            });
        }
        Ok(result)
    }
}

/// 校验 endpoint URL 是否指向允许的外网地址。
///
/// 防御 SSRF：恶意音源 JSON 配置可能指定内网地址（如 `169.254.169.254` 云元数据、
/// `127.0.0.1` 本机、`192.168.x.x` 私网、`fe80::` 链路本地）使播放器代为访问内网服务。
/// 本函数在音源导入期对所有 endpoint URL 做静态校验，命中黑名单的源在导入时即被拒绝。
///
/// 规则：
/// - 仅允许 `http`/`https` scheme；
/// - 拒绝字面量主机名 `localhost`；
/// - 拒绝解析为回环/未指定/链路本地/多播/私网/CGNAT/保留段的 IPv4/IPv6 字面量；
/// - 域名（非字面量 IP）默认放行：运行时由本机网络栈承担 DNS 解析后的访问控制。
fn validate_endpoint_url(url: &str) -> Result<()> {
    let parsed = url::Url::parse(url)
        .map_err(|e| CoreError::Source(format!("音源 endpoint URL 非法: {e}")))?;
    match parsed.scheme() {
        "http" | "https" => {}
        s => {
            return Err(CoreError::Source(format!(
                "音源 endpoint URL 仅允许 http/https，实际为 {s}: {url}"
            )));
        }
    }
    // 使用 host() 返回类型化 Host 枚举，避免 host_str() 对 IPv6 含括号导致 parse 失败
    match parsed.host() {
        None => {
            return Err(CoreError::Source(format!(
                "音源 endpoint URL 缺少 host: {url}"
            )));
        }
        Some(url::Host::Domain(d)) if d.eq_ignore_ascii_case("localhost") => {
            return Err(CoreError::Source(format!(
                "音源 endpoint URL 禁止指向 localhost: {url}"
            )));
        }
        Some(url::Host::Ipv4(ip4)) if is_disallowed_ipv4(&ip4) => {
            return Err(CoreError::Source(format!(
                "音源 endpoint URL 禁止指向内网/保留地址 {ip4}: {url}"
            )));
        }
        Some(url::Host::Ipv6(ip6)) if is_disallowed_ipv6(&ip6) => {
            return Err(CoreError::Source(format!(
                "音源 endpoint URL 禁止指向内网/保留地址 {ip6}: {url}"
            )));
        }
        _ => {} // 公网域名 / 公网 IP 字面量放行
    }
    Ok(())
}

/// IPv4 黑名单：本机网络/私网/CGNAT/回环/链路本地/保留/多播/文档用途网段。
fn is_disallowed_ipv4(ip: &std::net::Ipv4Addr) -> bool {
    let o = ip.octets();
    o[0] == 0 // 0.0.0.0/8 本机网络
        || o[0] == 10 // 10.0.0.0/8 RFC1918
        || (o[0] == 100 && (o[1] & 0xc0) == 64) // 100.64.0.0/10 RFC6598 CGNAT
        || o[0] == 127 // 127.0.0.0/8 回环
        || (o[0] == 169 && o[1] == 254) // 169.254.0.0/16 链路本地（含云元数据）
        || (o[0] == 172 && (o[1] & 0xf0) == 16) // 172.16.0.0/12 RFC1918
        || (o[0] == 192 && o[1] == 0 && (o[2] == 0 || o[2] == 2)) // 192.0.0.0/24, 192.0.2.0/24
        || (o[0] == 192 && o[1] == 88 && o[2] == 99) // 192.88.99.0/24
        || (o[0] == 192 && o[1] == 168) // 192.168.0.0/16 RFC1918
        || (o[0] == 198 && (o[1] == 18 || o[1] == 19)) // 198.18.0.0/15 基准测试
        || (o[0] == 198 && o[1] == 51 && o[2] == 100) // 198.51.100.0/24 TEST-NET-2
        || (o[0] == 203 && o[1] == 0 && o[2] == 113) // 203.0.113.0/24 TEST-NET-3
        || (o[0] & 0xf0) == 0xe0 // 224.0.0.0/4 多播
        || (o[0] & 0xf0) == 0xf0 // 240.0.0.0/4 保留
}

/// IPv6 黑名单：回环/未指定/链路本地/唯一本地/多播；IPv4-mapped 按对应 IPv4 检查。
fn is_disallowed_ipv6(ip: &std::net::Ipv6Addr) -> bool {
    let o = ip.octets();
    ip.is_loopback()
        || ip.is_unspecified()
        || ip.is_multicast()
        || (o[0] == 0xfe && (o[1] & 0xc0) == 0x80) // fe80::/10 链路本地
        || (o[0] & 0xfe) == 0xfc // fc00::/7 唯一本地
        // IPv4-mapped ::ffff:a.b.c.d：按对应 IPv4 规则检查
        || (o[0..10] == [0u8; 10]
            && o[10] == 0xff
            && o[11] == 0xff
            && is_disallowed_ipv4(&std::net::Ipv4Addr::new(o[12], o[13], o[14], o[15])))
}

/// 按 mapping 描述的字段名/点路径从 JSON 值中取子值。
///
/// 支持两种写法：
/// - 简单字段名：`"title"` 等价于 `value["title"]`
/// - 点路径：`"a.b.c"` 等价于 `value["a"]["b"]["c"]`
///
/// 任何一段不存在或 mapping 为空都返回 `None`。
fn extract_field<'a>(value: &'a Value, mapping: &str) -> Option<&'a Value> {
    if mapping.is_empty() {
        return None;
    }
    let mut current = value;
    for part in mapping.split('.') {
        current = current.get(part)?;
    }
    Some(current)
}

/// 将 JSON 值转为 u64：兼容整数与浮点（截断）。
fn value_to_u64(v: &Value) -> Option<u64> {
    v.as_u64().or_else(|| v.as_f64().map(|f| f as u64))
}

/// 将字符串/数组形态的 artists 字段统一为 `Vec<String>`。
fn collect_artists(v: &Value) -> Vec<String> {
    match v {
        Value::String(s) => vec![s.clone()],
        Value::Array(arr) => arr
            .iter()
            .filter_map(|x| x.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

/// 将字符串/数组形态的 song_ids 字段统一为 `Vec<String>`。
fn collect_string_array(v: &Value) -> Vec<String> {
    v.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

/// 将一个 JSON 对象按 `SongMapping` 映射为标准 `Song`。
///
/// 缺失字段返回 `None`/默认值，不会 panic。
fn map_song(value: &Value, mapping: &SongMapping, source_id: &str) -> Option<Song> {
    let id = extract_field(value, &mapping.id)?.as_str()?.to_string();
    let title = extract_field(value, &mapping.title)?.as_str()?.to_string();

    let artists = mapping
        .artists
        .as_deref()
        .and_then(|m| extract_field(value, m))
        .map(collect_artists)
        .unwrap_or_default();

    let album = mapping
        .album
        .as_deref()
        .and_then(|m| extract_field(value, m))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let cover_url = mapping
        .cover_url
        .as_deref()
        .and_then(|m| extract_field(value, m))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let duration_ms = mapping
        .duration_ms
        .as_deref()
        .and_then(|m| extract_field(value, m))
        .and_then(value_to_u64);

    let lyric_url = mapping
        .lyric_url
        .as_deref()
        .and_then(|m| extract_field(value, m))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let play_url = mapping
        .play_url
        .as_deref()
        .and_then(|m| extract_field(value, m))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(Song {
        id,
        source_id: source_id.to_string(),
        title,
        artists,
        album,
        cover_url,
        duration_ms,
        lyric_url,
        play_url: play_url.clone(),
        local_path: None,
        origin: SongOrigin::Online {
            source_id: source_id.to_string(),
            play_url: play_url.unwrap_or_default(),
        },
    })
}

/// 将一个 JSON 对象按 `AlbumMapping` 映射为 `Album`。
fn map_album(value: &Value, mapping: &AlbumMapping, source_id: &str) -> Option<Album> {
    let id = extract_field(value, &mapping.id)?.as_str()?.to_string();
    let name = extract_field(value, &mapping.name)?.as_str()?.to_string();
    let artists = mapping
        .artists
        .as_deref()
        .and_then(|m| extract_field(value, m))
        .map(collect_artists)
        .unwrap_or_default();
    let cover_url = mapping
        .cover_url
        .as_deref()
        .and_then(|m| extract_field(value, m))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let song_ids = mapping
        .song_ids
        .as_deref()
        .and_then(|m| extract_field(value, m))
        .map(collect_string_array)
        .unwrap_or_default();
    Some(Album {
        id,
        source_id: source_id.to_string(),
        name,
        artists,
        cover_url,
        song_ids,
    })
}

/// 将一个 JSON 对象按 `ArtistMapping` 映射为 `Artist`。
fn map_artist(value: &Value, mapping: &ArtistMapping, source_id: &str) -> Option<Artist> {
    let id = extract_field(value, &mapping.id)?.as_str()?.to_string();
    let name = extract_field(value, &mapping.name)?.as_str()?.to_string();
    let avatar_url = mapping
        .avatar_url
        .as_deref()
        .and_then(|m| extract_field(value, m))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let song_ids = mapping
        .song_ids
        .as_deref()
        .and_then(|m| extract_field(value, m))
        .map(collect_string_array)
        .unwrap_or_default();
    Some(Artist {
        id,
        source_id: source_id.to_string(),
        name,
        avatar_url,
        song_ids,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn extract_field_simple_name() {
        let v = json!({ "id": "s1", "name": "title" });
        assert_eq!(extract_field(&v, "id").and_then(|x| x.as_str()), Some("s1"));
        assert_eq!(
            extract_field(&v, "name").and_then(|x| x.as_str()),
            Some("title")
        );
        assert!(extract_field(&v, "missing").is_none());
        assert!(extract_field(&v, "").is_none());
    }

    #[test]
    fn extract_field_dotted_path() {
        let v = json!({ "data": { "song": { "id": "s2" } } });
        assert_eq!(
            extract_field(&v, "data.song.id").and_then(|x| x.as_str()),
            Some("s2")
        );
        assert!(extract_field(&v, "data.missing.id").is_none());
        assert!(extract_field(&v, "data.song.id.extra").is_none());
    }

    #[test]
    fn map_song_full_fields_array_artists() {
        let mapping = SongMapping {
            id: "id".into(),
            title: "name".into(),
            artists: Some("artists".into()),
            album: Some("album".into()),
            cover_url: Some("cover".into()),
            duration_ms: Some("durationMs".into()),
            lyric_url: Some("lyricUrl".into()),
            play_url: Some("playUrl".into()),
        };
        let v = json!({
            "id": "s1",
            "name": "My Song",
            "artists": ["A", "B"],
            "album": "Album X",
            "cover": "http://example.com/c.jpg",
            "durationMs": 180000,
            "lyricUrl": "http://example.com/l.lrc",
            "playUrl": "http://example.com/p.mp3"
        });
        let song = map_song(&v, &mapping, "demo").unwrap();
        assert_eq!(song.id, "s1");
        assert_eq!(song.source_id, "demo");
        assert_eq!(song.title, "My Song");
        assert_eq!(song.artists, vec!["A".to_string(), "B".to_string()]);
        assert_eq!(song.album.as_deref(), Some("Album X"));
        assert_eq!(song.cover_url.as_deref(), Some("http://example.com/c.jpg"));
        assert_eq!(song.duration_ms, Some(180_000));
        assert_eq!(song.lyric_url.as_deref(), Some("http://example.com/l.lrc"));
        assert_eq!(song.play_url.as_deref(), Some("http://example.com/p.mp3"));
        match song.origin {
            SongOrigin::Online {
                source_id,
                play_url,
            } => {
                assert_eq!(source_id, "demo");
                assert_eq!(play_url, "http://example.com/p.mp3");
            }
            _ => panic!("origin 应为 Online"),
        }
    }

    #[test]
    fn map_song_artists_as_string() {
        let mapping = SongMapping {
            id: "id".into(),
            title: "title".into(),
            artists: Some("artist".into()),
            album: None,
            cover_url: None,
            duration_ms: None,
            lyric_url: None,
            play_url: None,
        };
        let v = json!({ "id": "s1", "title": "T", "artist": "Solo" });
        let song = map_song(&v, &mapping, "demo").unwrap();
        assert_eq!(song.artists, vec!["Solo".to_string()]);
    }

    #[test]
    fn map_song_duration_supports_float() {
        let mapping = SongMapping {
            id: "id".into(),
            title: "title".into(),
            artists: None,
            album: None,
            cover_url: None,
            duration_ms: Some("durationMs".into()),
            lyric_url: None,
            play_url: None,
        };
        let v = json!({ "id": "s1", "title": "T", "durationMs": 180000.0 });
        let song = map_song(&v, &mapping, "demo").unwrap();
        assert_eq!(song.duration_ms, Some(180_000));
    }

    #[test]
    fn map_song_missing_required_returns_none() {
        let mapping = SongMapping {
            id: "id".into(),
            title: "title".into(),
            artists: None,
            album: None,
            cover_url: None,
            duration_ms: None,
            lyric_url: None,
            play_url: None,
        };
        // 缺少 id
        let v = json!({ "title": "T" });
        assert!(map_song(&v, &mapping, "demo").is_none());
        // 缺少 title
        let v = json!({ "id": "s1" });
        assert!(map_song(&v, &mapping, "demo").is_none());
    }

    #[test]
    fn map_song_missing_optional_uses_defaults() {
        let mapping = SongMapping {
            id: "id".into(),
            title: "title".into(),
            artists: None,
            album: None,
            cover_url: None,
            duration_ms: None,
            lyric_url: None,
            play_url: None,
        };
        let v = json!({ "id": "s1", "title": "T" });
        let song = map_song(&v, &mapping, "demo").unwrap();
        assert!(song.artists.is_empty());
        assert!(song.album.is_none());
        assert!(song.cover_url.is_none());
        assert!(song.duration_ms.is_none());
        assert!(song.lyric_url.is_none());
        assert!(song.play_url.is_none());
        match song.origin {
            SongOrigin::Online {
                source_id,
                play_url,
            } => {
                assert_eq!(source_id, "demo");
                assert!(play_url.is_empty());
            }
            _ => panic!("origin 应为 Online"),
        }
    }

    #[test]
    fn map_song_dotted_mapping() {
        let mapping = SongMapping {
            id: "data.song.id".into(),
            title: "data.song.title".into(),
            artists: None,
            album: None,
            cover_url: None,
            duration_ms: None,
            lyric_url: None,
            play_url: None,
        };
        let v = json!({ "data": { "song": { "id": "s9", "title": "Nested" } } });
        let song = map_song(&v, &mapping, "demo").unwrap();
        assert_eq!(song.id, "s9");
        assert_eq!(song.title, "Nested");
    }

    #[test]
    fn map_album_and_artist_work() {
        let album_mapping = AlbumMapping {
            id: "id".into(),
            name: "name".into(),
            artists: Some("artists".into()),
            cover_url: Some("cover".into()),
            song_ids: Some("songs".into()),
        };
        let v = json!({
            "id": "a1", "name": "Album", "artists": ["X"], "cover": "c", "songs": ["s1", "s2"]
        });
        let album = map_album(&v, &album_mapping, "demo").unwrap();
        assert_eq!(album.id, "a1");
        assert_eq!(album.source_id, "demo");
        assert_eq!(album.name, "Album");
        assert_eq!(album.artists, vec!["X".to_string()]);
        assert_eq!(album.cover_url.as_deref(), Some("c"));
        assert_eq!(album.song_ids, vec!["s1".to_string(), "s2".to_string()]);

        let artist_mapping = ArtistMapping {
            id: "id".into(),
            name: "name".into(),
            avatar_url: Some("avatar".into()),
            song_ids: Some("songs".into()),
        };
        let v = json!({
            "id": "ar1", "name": "Artist", "avatar": "av", "songs": ["s1"]
        });
        let artist = map_artist(&v, &artist_mapping, "demo").unwrap();
        assert_eq!(artist.id, "ar1");
        assert_eq!(artist.source_id, "demo");
        assert_eq!(artist.name, "Artist");
        assert_eq!(artist.avatar_url.as_deref(), Some("av"));
        assert_eq!(artist.song_ids, vec!["s1".to_string()]);
    }

    #[test]
    fn json_source_new_builds_client_with_defaults() {
        let config_json = serde_json::json!({
            "manifest": { "id": "demo", "name": "Demo", "version": "1.0.0", "author": "t" },
            "endpoints": {
                "search": { "url": "https://example.com/s" },
                "metadata": { "url": "https://example.com/m" },
                "playUrl": { "url": "https://example.com/p" }
            },
            "fieldMapping": {
                "song": { "id": "id", "title": "title" },
                "album": { "id": "id", "name": "name" },
                "artist": { "id": "id", "name": "name" },
                "lyric": { "lines": [{ "timeMs": "t", "text": "x" }] }
            }
        });
        let config = SoundSourceConfig::from_json(&config_json.to_string()).unwrap();
        let source = JsonSource::new(config).unwrap();
        assert_eq!(source.id(), "demo");
        assert_eq!(source.name(), "Demo");
        assert!(source.enabled);
        assert_eq!(source.priority, 0);
    }

    /// SSRF 防御：合法公网 endpoint URL（example.com）应通过校验。
    #[test]
    fn validate_endpoint_url_allows_public_domain() {
        assert!(validate_endpoint_url("https://example.com/api").is_ok());
        assert!(validate_endpoint_url("http://music.example.com/search?q=1").is_ok());
        // 公网 IP 字面量允许（8.8.8.8）
        assert!(validate_endpoint_url("https://8.8.8.8/api").is_ok());
    }

    /// SSRF 防御：非 http/https scheme 应被拒绝。
    #[test]
    fn validate_endpoint_url_rejects_non_http_scheme() {
        assert!(validate_endpoint_url("file:///etc/passwd").is_err());
        assert!(validate_endpoint_url("ftp://example.com/file").is_err());
        assert!(validate_endpoint_url("gopher://example.com/").is_err());
    }

    /// SSRF 防御：localhost / 回环 / 链路本地 / 私网 IPv4 字面量应被拒绝。
    #[test]
    fn validate_endpoint_url_rejects_internal_ipv4() {
        // localhost 字面量
        assert!(validate_endpoint_url("http://localhost/api").is_err());
        assert!(validate_endpoint_url("http://LocalHost:8080/api").is_err());
        // 回环
        assert!(validate_endpoint_url("http://127.0.0.1/api").is_err());
        assert!(validate_endpoint_url("http://127.1.2.3/api").is_err());
        // 链路本地（含云元数据 169.254.169.254）
        assert!(validate_endpoint_url("http://169.254.169.254/latest/meta-data/").is_err());
        // 私网
        assert!(validate_endpoint_url("http://10.0.0.1/api").is_err());
        assert!(validate_endpoint_url("http://192.168.1.1/api").is_err());
        assert!(validate_endpoint_url("http://172.16.0.1/api").is_err());
        // 未指定 / 本机网络
        assert!(validate_endpoint_url("http://0.0.0.0/api").is_err());
        // CGNAT
        assert!(validate_endpoint_url("http://100.64.0.1/api").is_err());
        // 多播 / 保留
        assert!(validate_endpoint_url("http://224.0.0.1/api").is_err());
        assert!(validate_endpoint_url("http://240.0.0.1/api").is_err());
    }

    /// SSRF 防御：IPv6 回环 / 链路本地 / 唯一本地 / IPv4-mapped 内网应被拒绝。
    #[test]
    fn validate_endpoint_url_rejects_internal_ipv6() {
        assert!(validate_endpoint_url("http://[::1]/api").is_err());
        assert!(validate_endpoint_url("http://[::]/api").is_err());
        assert!(validate_endpoint_url("http://[fe80::1]/api").is_err());
        assert!(validate_endpoint_url("http://[fc00::1]/api").is_err());
        assert!(validate_endpoint_url("http://[ff02::1]/api").is_err());
        // IPv4-mapped ::ffff:127.0.0.1 应被拒绝
        assert!(validate_endpoint_url("http://[::ffff:127.0.0.1]/api").is_err());
        assert!(validate_endpoint_url("http://[::ffff:169.254.169.254]/api").is_err());
    }

    /// SSRF 防御：音源导入时若任一 endpoint 命中黑名单应在 JsonSource::new 即失败。
    #[test]
    fn json_source_new_rejects_ssrf_endpoint() {
        let config_json = serde_json::json!({
            "manifest": { "id": "evil", "name": "Evil", "version": "1.0.0", "author": "x" },
            "endpoints": {
                "search": { "url": "http://169.254.169.254/latest/meta-data/" },
                "metadata": { "url": "https://example.com/m" },
                "playUrl": { "url": "https://example.com/p" }
            },
            "fieldMapping": {
                "song": { "id": "id", "title": "title" },
                "album": { "id": "id", "name": "name" },
                "artist": { "id": "id", "name": "name" },
                "lyric": { "lines": [{ "timeMs": "t", "text": "x" }] }
            }
        });
        let config = SoundSourceConfig::from_json(&config_json.to_string()).unwrap();
        let err = JsonSource::new(config).err().expect("SSRF endpoint 应在导入期被拒绝");
        assert!(err.to_string().contains("169.254.169.254"));
    }
}
