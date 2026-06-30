//! 核心数据模型。
//!
//! 定义歌曲、专辑、艺术家、歌词、搜索结果、播放列表、收藏分组、
//! 播放模式、播放状态、排行榜等跨平台共享数据结构。
//! 所有结构均实现 Serialize/Deserialize/Clone/Debug，部分实现 PartialEq/Eq。

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 单首歌曲
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Song {
    /// 歌曲唯一标识（在单个音源内唯一）
    pub id: String,
    /// 所属音源标识
    pub source_id: String,
    /// 标题
    pub title: String,
    /// 艺术家列表
    pub artists: Vec<String>,
    /// 专辑名
    pub album: Option<String>,
    /// 封面 URL
    pub cover_url: Option<String>,
    /// 时长（毫秒）
    pub duration_ms: Option<u64>,
    /// 歌词资源 URL
    pub lyric_url: Option<String>,
    /// 可播放 URL
    pub play_url: Option<String>,
    /// 本地文件路径（若已缓存到本地）
    pub local_path: Option<PathBuf>,
    /// 歌曲来源（在线/本地/NAS）
    pub origin: SongOrigin,
}

/// 歌曲来源类型，使用内部标签 `type` 区分
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum SongOrigin {
    /// 在线音源
    Online { source_id: String, play_url: String },
    /// 本地文件
    Local { path: PathBuf },
    /// NAS 等远程协议
    Nas { protocol: String, url: String },
}

/// 专辑
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Album {
    pub id: String,
    pub source_id: String,
    pub name: String,
    pub artists: Vec<String>,
    pub cover_url: Option<String>,
    pub song_ids: Vec<String>,
}

/// 艺术家
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Artist {
    pub id: String,
    pub source_id: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub song_ids: Vec<String>,
}

/// 单行歌词（LRC 时间轴）；time_ms 为 None 表示无时间戳的歌词行
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LyricLine {
    pub time_ms: Option<u64>,
    pub text: String,
}

/// 歌词，可带翻译
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Lyric {
    pub lines: Vec<LyricLine>,
    pub translation: Option<Vec<LyricLine>>,
}

/// 搜索结果聚合
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub keyword: String,
    pub songs: Vec<Song>,
    pub albums: Vec<Album>,
    pub artists: Vec<Artist>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

/// 播放列表
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Playlist {
    pub id: String,
    pub name: String,
    pub song_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
}

/// 收藏分组
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct FavoriteGroup {
    pub id: String,
    pub name: String,
    pub song_ids: Vec<String>,
}

/// 播放模式（snake_case 序列化）
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlayMode {
    Sequential,
    SingleLoop,
    Random,
}

/// 播放状态（含 f32 音量，仅实现 PartialEq）
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlayState {
    pub current_song_id: Option<String>,
    pub playlist_id: Option<String>,
    pub index: Option<usize>,
    pub position_ms: u64,
    pub duration_ms: u64,
    pub is_playing: bool,
    pub volume: f32,
    pub mode: PlayMode,
}

/// 排行榜
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Leaderboard {
    pub id: String,
    pub source_id: String,
    pub name: String,
    pub cover_url: Option<String>,
    pub songs: Vec<Song>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 验证 Song 可被序列化并完整反序列化往返
    #[test]
    fn test_song_roundtrip() {
        let song = Song {
            id: "song-1".into(),
            source_id: "local".into(),
            title: "测试歌曲".into(),
            artists: vec!["艺术家A".into(), "艺术家B".into()],
            album: Some("专辑X".into()),
            cover_url: Some("http://example.com/cover.jpg".into()),
            duration_ms: Some(180_000),
            lyric_url: None,
            play_url: Some("http://example.com/play.mp3".into()),
            local_path: None,
            origin: SongOrigin::Online {
                source_id: "local".into(),
                play_url: "http://example.com/play.mp3".into(),
            },
        };
        let json = serde_json::to_string(&song).expect("序列化失败");
        let decoded: Song = serde_json::from_str(&json).expect("反序列化失败");
        assert_eq!(song, decoded);
    }

    /// 验证 SongOrigin 使用 "type" 内部标签
    #[test]
    fn test_song_origin_tag() {
        let origin = SongOrigin::Local {
            path: PathBuf::from("/music/a.mp3"),
        };
        let json = serde_json::to_string(&origin).expect("序列化失败");
        assert!(
            json.contains("\"type\":\"Local\""),
            "未生成 type=Local 标签: {}",
            json
        );

        let nas = SongOrigin::Nas {
            protocol: "smb".into(),
            url: "smb://host/share".into(),
        };
        let json = serde_json::to_string(&nas).expect("序列化失败");
        assert!(
            json.contains("\"type\":\"Nas\""),
            "未生成 type=Nas 标签: {}",
            json
        );
    }

    /// 验证 PlayMode 使用 snake_case 序列化
    #[test]
    fn test_play_mode_snake_case() {
        assert_eq!(
            serde_json::to_string(&PlayMode::Sequential).unwrap(),
            "\"sequential\""
        );
        assert_eq!(
            serde_json::to_string(&PlayMode::SingleLoop).unwrap(),
            "\"single_loop\""
        );
        assert_eq!(
            serde_json::to_string(&PlayMode::Random).unwrap(),
            "\"random\""
        );
    }

    /// 验证歌词结构序列化往返，且支持无时间戳行
    #[test]
    fn test_lyric_serialization() {
        let lyric = Lyric {
            lines: vec![
                LyricLine {
                    time_ms: Some(0),
                    text: "第一行".into(),
                },
                LyricLine {
                    time_ms: None,
                    text: "无时间戳行".into(),
                },
            ],
            translation: None,
        };
        let json = serde_json::to_string(&lyric).expect("序列化失败");
        let decoded: Lyric = serde_json::from_str(&json).expect("反序列化失败");
        assert_eq!(lyric, decoded);
        assert_eq!(decoded.lines.len(), 2);
    }
}
