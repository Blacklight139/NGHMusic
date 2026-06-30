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
    pub fn new(config: SoundSourceConfig) -> Result<Self> {
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
}
