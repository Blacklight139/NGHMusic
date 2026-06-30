//! 核心数据模型。跨端共享、与音源格式无关。
//!
//! 各音源的原始字段经 `SourceFieldMapping` 映射为本模块中的标准类型。

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 全局唯一歌曲标识：`{source_id}:{song_id}`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SongRef {
    /// 音源 ID
    pub source_id: String,
    /// 音源内歌曲 ID
    pub song_id: String,
}

impl SongRef {
    pub fn new(source_id: impl Into<String>, song_id: impl Into<String>) -> Self {
        Self {
            source_id: source_id.into(),
            song_id: song_id.into(),
        }
    }
    /// 复合键，便于缓存键与持久化
    pub fn key(&self) -> String {
        format!("{}:{}", self.source_id, self.song_id)
    }
}

/// 歌曲标准元数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Song {
    pub source_id: String,
    pub song_id: String,
    pub title: String,
    pub artist: String,
    pub album: String,
    pub cover_url: Option<String>,
    /// 秒
    pub duration: Option<f64>,
    /// 歌词地址（URL 或数据 URI）
    pub lyric_url: Option<String>,
    /// 播放数据 URL（流地址 / 文件路径）
    pub play_url: Option<String>,
    /// 是否已缓存
    pub cached: bool,
}

impl Song {
    pub fn song_ref(&self) -> SongRef {
        SongRef::new(&self.source_id, &self.song_id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Album {
    pub source_id: String,
    pub album_id: String,
    pub name: String,
    pub artist: String,
    pub cover_url: Option<String>,
    pub publish_date: Option<String>,
    pub song_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Artist {
    pub source_id: String,
    pub artist_id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub song_ids: Vec<String>,
}

/// LRC 解析后的歌词行
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LyricLine {
    /// 秒
    pub time: f64,
    pub text: String,
    /// 可选翻译
    pub translation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Lyrics {
    pub song_ref: SongRef,
    pub lines: Vec<LyricLine>,
}

/// 搜索分类
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SearchType {
    Song,
    Album,
    Artist,
}

/// 单条搜索结果（含音源来源）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub source_id: String,
    pub source_name: String,
    #[serde(flatten)]
    pub item: SearchItem,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum SearchItem {
    Song(Song),
    Album(Album),
    Artist(Artist),
}

/// 分页请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub offset: u32,
    pub limit: u32,
}

impl Default for Page {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 20,
        }
    }
}

/// 分页结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paged<T> {
    pub items: Vec<T>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

impl<T> Paged<T> {
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> Paged<U> {
        Paged {
            items: self.items.into_iter().map(f).collect(),
            total: self.total,
            offset: self.offset,
            limit: self.limit,
        }
    }
}

/// 排行榜
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Ranking {
    pub source_id: String,
    pub ranking_id: String,
    pub name: String,
    pub cover_url: Option<String>,
    pub update_time: Option<String>,
    pub songs: Vec<Song>,
}

/// 播放模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayMode {
    /// 顺序播放
    Sequence,
    /// 单曲循环
    RepeatOne,
    /// 随机
    Shuffle,
}

impl Default for PlayMode {
    fn default() -> Self {
        PlayMode::Sequence
    }
}

/// 播放器状态快照（持久化用）
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerState {
    pub current: Option<SongRef>,
    pub playlist_id: Option<String>,
    pub position_sec: f64,
    pub volume: f32,
    pub mode: PlayMode,
    pub playing: bool,
}

/// 播放列表
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub songs: Vec<SongRef>,
}

impl Playlist {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            songs: Vec::new(),
        }
    }
}

/// 收藏夹分组
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FavoriteGroup {
    pub id: String,
    pub name: String,
    pub songs: Vec<SongRef>,
}

impl FavoriteGroup {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            songs: Vec::new(),
        }
    }
}

/// 网络协议源
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "config")]
pub enum ProtocolSource {
    Smb(SmbConfig),
    WebDav(WebDavConfig),
    Ftp(FtpConfig),
    Dlna(DlnaConfig),
    Nfs(NfsConfig),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SmbConfig {
    pub host: String,
    pub port: Option<u16>,
    pub share: String,
    pub username: Option<String>,
    pub password: Option<String>,
    pub workgroup: Option<String>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WebDavConfig {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FtpConfig {
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub passive: bool,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DlnaConfig {
    pub device_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NfsConfig {
    pub host: String,
    pub export: String,
    pub path: String,
}

/// 协议浏览条目
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub url: String,
}
