// 职责：HarmonyOS ArkTS 数据模型，1:1 镜像 /workspace/core/src/models.rs 的核心数据结构。
// 规范：
//   - 字段名严格使用 Rust snake_case（与核心序列化 JSON 对齐），跨端反序列化不错位。
//   - SongOrigin 使用内部标签 `type`（与 Rust `#[serde(tag = "type")]` 一致），
//     Nas 变体字段名为 `protocol`（NOT `protocol_name`）—— 修复 docs/bug-report.md HAR-003。
//   - PlayMode 使用 snake_case 序列化（`sequential` / `single_loop` / `random`），
//     与 Rust `#[serde(rename_all = "snake_case")]` 对齐。
//   - 数字字段保留 number 类型；可选字段使用 `?` 表示（与 Rust Option<T> 对齐）。

/**
 * 歌曲来源类型（内部标签 `type`）。
 * 修复 HAR-003：Nas 变体使用 `protocol` 字段，而非 `protocol_name`。
 */
export type SongOrigin =
  | { type: 'Online'; source_id: string; play_url: string }
  | { type: 'Local'; path: string }
  | { type: 'Nas'; protocol: string; url: string };

/**
 * 单首歌曲。镜像 core/src/models.rs::Song。
 */
export interface Song {
  id: string;
  source_id: string;
  title: string;
  artists: string[];
  album?: string;
  cover_url?: string;
  duration_ms?: number;
  lyric_url?: string;
  play_url?: string;
  local_path?: string;
  origin: SongOrigin;
}

/**
 * 专辑。镜像 core/src/models.rs::Album。
 */
export interface Album {
  id: string;
  source_id: string;
  name: string;
  artists: string[];
  cover_url?: string;
  song_ids: string[];
}

/**
 * 艺术家。镜像 core/src/models.rs::Artist。
 */
export interface Artist {
  id: string;
  source_id: string;
  name: string;
  avatar_url?: string;
  song_ids: string[];
}

/**
 * 单行歌词。镜像 core/src/models.rs::LyricLine。
 * time_ms 为 undefined 表示无时间戳的纯文本行。
 */
export interface LyricLine {
  time_ms?: number;
  text: string;
}

/**
 * 歌词，可带翻译。镜像 core/src/models.rs::Lyric。
 */
export interface Lyric {
  lines: LyricLine[];
  translation?: LyricLine[];
}

/**
 * 搜索结果聚合。镜像 core/src/models.rs::SearchResult。
 */
export interface SearchResult {
  keyword: string;
  songs: Song[];
  albums: Album[];
  artists: Artist[];
  total: number;
  page: number;
  page_size: number;
}

/**
 * 排行榜。镜像 core/src/models.rs::Leaderboard。
 */
export interface Leaderboard {
  id: string;
  source_id: string;
  name: string;
  cover_url?: string;
  songs: Song[];
}

/**
 * 播放列表。镜像 core/src/models.rs::Playlist。
 */
export interface Playlist {
  id: string;
  name: string;
  song_ids: string[];
  created_at: string;
}

/**
 * 收藏分组。镜像 core/src/models.rs::FavoriteGroup。
 */
export interface FavoriteGroup {
  id: string;
  name: string;
  song_ids: string[];
}

/**
 * 播放模式枚举。镜像 core/src/models.rs::PlayMode（snake_case 序列化）。
 * 同时提供数值形式（与 PlayerManager 内部使用的 0/1/2 等价）以便 ArkTS 状态切换。
 */
export enum PlayMode {
  /** 顺序播放（snake_case: sequential） */
  SEQUENTIAL = 0,
  /** 单曲循环（snake_case: single_loop） */
  SINGLE_LOOP = 1,
  /** 随机播放（snake_case: random） */
  RANDOM = 2,
}

/**
 * PlayMode 序列化为 Rust snake_case 字符串。与 core `#[serde(rename_all = "snake_case")]` 对齐。
 */
export function playModeToSnakeString(mode: PlayMode): string {
  switch (mode) {
    case PlayMode.SEQUENTIAL: return 'sequential';
    case PlayMode.SINGLE_LOOP: return 'single_loop';
    case PlayMode.RANDOM: return 'random';
    default: return 'sequential';
  }
}

/**
 * 由 Rust snake_case 字符串解析为 PlayMode。
 */
export function playModeFromSnakeString(s: string): PlayMode {
  switch (s) {
    case 'single_loop': return PlayMode.SINGLE_LOOP;
    case 'random': return PlayMode.RANDOM;
    case 'sequential':
    default: return PlayMode.SEQUENTIAL;
  }
}

/**
 * 播放状态。镜像 core/src/models.rs::PlayState。
 */
export interface PlayState {
  current_song_id?: string;
  playlist_id?: string;
  index?: number;
  position_ms: number;
  duration_ms: number;
  is_playing: boolean;
  volume: number;
  mode: PlayMode;
}

/**
 * 飞牛 NAS 文件条目。镜像 core/src/feiniu.rs::NasFile。
 */
export interface NasFile {
  name: string;
  is_dir: boolean;
  size: number;
  modified?: string;
}
