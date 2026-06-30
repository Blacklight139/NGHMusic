//! 音源引擎：管理已加载音源的注册表、优先级、启停，并协调元数据/搜索/播放/歌词请求。

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use crate::models::*;
use crate::sources::schema::{SoundSourceConfig, SourceFieldMapping};
use crate::sources::http_source;
use crate::{CoreError, CoreResult};

/// 已加载的音源句柄
#[derive(Clone)]
pub struct SourceHandle {
    pub config: Arc<SoundSourceConfig>,
    pub enabled: bool,
    pub priority: i32,
}

pub struct SourceEngine {
    sources: Arc<RwLock<HashMap<String, SourceHandle>>>,
    pub(crate) http: reqwest::Client,
}

impl Default for SourceEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SourceEngine {
    pub fn new() -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("reqwest client build");
        Self {
            sources: Arc::new(RwLock::new(HashMap::new())),
            http,
        }
    }

    /// 注册音源（标准配置，已校验）
    pub fn register(&self, config: SoundSourceConfig, priority: i32) -> CoreResult<String> {
        {
            let map = self.sources.read();
            if map.contains_key(&config.manifest.id) {
                return Err(CoreError::SchemaValidation(format!(
                    "音源已存在: {}",
                    config.manifest.id
                )));
            }
        }
        let id = config.manifest.id.clone();
        let handle = SourceHandle {
            config: Arc::new(config),
            enabled: true,
            priority,
        };
        self.sources.write().insert(id.clone(), handle);
        Ok(id)
    }

    /// 卸载音源
    pub fn unregister(&self, id: &str) -> CoreResult<()> {
        self.sources
            .write()
            .remove(id)
            .ok_or_else(|| CoreError::SourceNotFound(id.into()))
            .map(|_| ())
    }

    pub fn set_enabled(&self, id: &str, enabled: bool) -> CoreResult<()> {
        let mut map = self.sources.write();
        let h = map
            .get_mut(id)
            .ok_or_else(|| CoreError::SourceNotFound(id.into()))?;
        h.enabled = enabled;
        Ok(())
    }

    pub fn set_priority(&self, id: &str, priority: i32) -> CoreResult<()> {
        let mut map = self.sources.write();
        let h = map
            .get_mut(id)
            .ok_or_else(|| CoreError::SourceNotFound(id.into()))?;
        h.priority = priority;
        Ok(())
    }

    /// 列出所有音源（按优先级降序）
    pub fn list(&self) -> Vec<(String, String, bool, i32)> {
        let mut v: Vec<_> = self
            .sources
            .read()
            .iter()
            .map(|(id, h)| (id.clone(), h.config.manifest.name.clone(), h.enabled, h.priority))
            .collect();
        v.sort_by(|a, b| b.3.cmp(&a.3));
        v
    }

    /// 按优先级排序的已启用音源
    pub fn enabled_sources(&self) -> Vec<SourceHandle> {
        let mut v: Vec<_> = self
            .sources
            .read()
            .values()
            .filter(|h| h.enabled)
            .cloned()
            .collect();
        v.sort_by(|a, b| b.priority.cmp(&a.priority));
        v
    }

    pub fn get(&self, id: &str) -> CoreResult<SourceHandle> {
        self.sources
            .read()
            .get(id)
            .cloned()
            .ok_or_else(|| CoreError::SourceNotFound(id.into()))
    }

    /// 获取歌曲元数据
    pub async fn fetch_song(&self, song_ref: &SongRef) -> CoreResult<Song> {
        let handle = self.get(&song_ref.source_id)?;
        let ep = handle
            .config
            .endpoints
            .metadata
            .as_ref()
            .ok_or_else(|| CoreError::SourceApi(format!("音源 {} 无 metadata 端点", song_ref.source_id)))?;
        let subs: [(&str, &str); 1] = [(
            "songId",
            song_ref.song_id.as_str(),
        )];
        let val = http_source::request_endpoint(&self.http, &handle.config, ep, &subs).await?;
        map_song(&song_ref.source_id, &handle.config.field_mapping, &val)
    }

    /// 获取播放 URL（定位音源内播放数据）
    pub async fn resolve_play_url(&self, song_ref: &SongRef) -> CoreResult<String> {
        let handle = self.get(&song_ref.source_id)?;
        // 若已有 play_url 直接返回
        if let Some(ep) = &handle.config.endpoints.play {
            let subs: [(&str, &str); 1] = [("songId", song_ref.song_id.as_str())];
            let val = http_source::request_endpoint(&self.http, &handle.config, ep, &subs).await?;
            // 取 play_url 字段映射
            let url = val
                .get(&handle.config.field_mapping.play_url)
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    CoreError::SourceApi(format!("play 响应缺少 {}", handle.config.field_mapping.play_url))
                })?;
            Ok(url.to_string())
        } else {
            // 回退：metadata 端点取 play_url
            let song = self.fetch_song(song_ref).await?;
            song.play_url
                .ok_or_else(|| CoreError::SourceApi(format!("音源 {} 无可用 play_url", song_ref.source_id)))
        }
    }

    /// 获取歌词
    pub async fn fetch_lyrics(&self, song_ref: &SongRef) -> CoreResult<Lyrics> {
        let handle = self.get(&song_ref.source_id)?;
        let ep = handle
            .config
            .endpoints
            .lyric
            .as_ref()
            .ok_or_else(|| CoreError::NotFound(format!("音源 {} 无 lyric 端点", song_ref.source_id)))?;
        let subs: [(&str, &str); 1] = [("songId", song_ref.song_id.as_str())];
        let val = http_source::request_endpoint(&self.http, &handle.config, ep, &subs).await?;
        // 支持：字符串 LRC、或 { lyric: "..." } 对象
        let lrc_text = if let Some(s) = val.as_str() {
            s.to_string()
        } else if let Some(l) = val.get(&handle.config.field_mapping.lyric).and_then(|x| x.as_str()) {
            l.to_string()
        } else {
            return Err(CoreError::Parse("歌词响应无可解析内容".into()));
        };
        let lines = crate::lyrics::parse_lrc(&lrc_text);
        Ok(Lyrics {
            song_ref: song_ref.clone(),
            lines,
        })
    }
}

/// 将音源原始 JSON 映射为标准 Song
pub fn map_song(source_id: &str, mapping: &SourceFieldMapping, v: &serde_json::Value) -> CoreResult<Song> {
    let pick = |field: &str| -> String {
        v.get(field)
            .map(|x| match x {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Null => String::new(),
                _ => x.to_string(),
            })
            .unwrap_or_default()
    };
    let duration = v
        .get(&mapping.duration)
        .and_then(|x| {
            x.as_f64()
                .or_else(|| x.as_str().and_then(|s| s.parse::<f64>().ok()))
        });
    let play_url = v
        .get(&mapping.play_url)
        .and_then(|x| x.as_str())
        .map(String::from);
    let cover_url = v
        .get(&mapping.cover)
        .and_then(|x| x.as_str())
        .map(String::from);
    let lyric_url = v
        .get(&mapping.lyric)
        .and_then(|x| x.as_str())
        .map(String::from);
    let song_id = pick(&mapping.song_id);
    if song_id.is_empty() {
        return Err(CoreError::Parse(format!("响应缺少 {}", mapping.song_id)));
    }
    Ok(Song {
        source_id: source_id.into(),
        song_id,
        title: pick(&mapping.title),
        artist: pick(&mapping.artist),
        album: pick(&mapping.album),
        cover_url,
        duration,
        lyric_url,
        play_url,
        cached: false,
    })
}
