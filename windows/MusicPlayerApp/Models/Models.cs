using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace MusicPlayerApp.Models;

// =============================================================
// 播放模式枚举（与 Rust core PlayMode 一致）
// =============================================================

/// <summary>播放模式：顺序播放 / 单曲循环 / 随机播放。</summary>
public enum PlayMode
{
    /// <summary>顺序播放（列表播完即止）。</summary>
    Sequential = 0,

    /// <summary>单曲循环。</summary>
    SingleLoop = 1,

    /// <summary>随机播放。</summary>
    Random = 2,
}

// =============================================================
// SongOrigin：内部标签 type 区分 Online / Local / Nas
// 与 core/src/models.rs::SongOrigin 一一对应
// =============================================================

/// <summary>歌曲来源基类，JSON 内部标签字段为 <c>type</c>。</summary>
[JsonPolymorphic(TypeDiscriminatorPropertyName = "type", IgnoreUnrecognized = true)]
[JsonDerivedType(typeof(OnlineOrigin), "Online")]
[JsonDerivedType(typeof(LocalOrigin), "Local")]
[JsonDerivedType(typeof(NasOrigin), "Nas")]
public abstract class SongOrigin
{
}

/// <summary>在线音源来源。</summary>
public sealed class OnlineOrigin : SongOrigin
{
    [JsonPropertyName("source_id")]
    public string? SourceId { get; set; }

    [JsonPropertyName("play_url")]
    public string? PlayUrl { get; set; }
}

/// <summary>本地文件来源。</summary>
public sealed class LocalOrigin : SongOrigin
{
    [JsonPropertyName("path")]
    public string? Path { get; set; }
}

/// <summary>NAS（飞牛 / 协议源）来源。</summary>
public sealed class NasOrigin : SongOrigin
{
    [JsonPropertyName("protocol")]
    public string? Protocol { get; set; }

    [JsonPropertyName("url")]
    public string? Url { get; set; }
}

// =============================================================
// 核心业务数据模型
// =============================================================

/// <summary>歌曲，与 core::models::Song 对齐。</summary>
public sealed class Song
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = string.Empty;

    [JsonPropertyName("source_id")]
    public string? SourceId { get; set; }

    [JsonPropertyName("title")]
    public string Title { get; set; } = string.Empty;

    [JsonPropertyName("artists")]
    public List<string> Artists { get; set; } = new();

    [JsonPropertyName("album")]
    public string? Album { get; set; }

    [JsonPropertyName("cover_url")]
    public string? CoverUrl { get; set; }

    [JsonPropertyName("duration_ms")]
    public ulong? DurationMs { get; set; }

    [JsonPropertyName("lyric_url")]
    public string? LyricUrl { get; set; }

    [JsonPropertyName("play_url")]
    public string? PlayUrl { get; set; }

    [JsonPropertyName("local_path")]
    public string? LocalPath { get; set; }

    [JsonPropertyName("origin")]
    public SongOrigin? Origin { get; set; }
}

/// <summary>专辑，与 core::models::Album 对齐。</summary>
public sealed class Album
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = string.Empty;

    [JsonPropertyName("source_id")]
    public string? SourceId { get; set; }

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("artists")]
    public List<string> Artists { get; set; } = new();

    [JsonPropertyName("cover_url")]
    public string? CoverUrl { get; set; }

    [JsonPropertyName("song_ids")]
    public List<string> SongIds { get; set; } = new();
}

/// <summary>艺术家，与 core::models::Artist 对齐。</summary>
public sealed class Artist
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = string.Empty;

    [JsonPropertyName("source_id")]
    public string? SourceId { get; set; }

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("avatar_url")]
    public string? AvatarUrl { get; set; }

    [JsonPropertyName("song_ids")]
    public List<string> SongIds { get; set; } = new();
}

/// <summary>歌词单行，与 core::models::LyricLine 对齐。</summary>
public sealed class LyricLine
{
    /// <summary>毫秒时间戳；为 null 表示无时间戳的纯文本行。</summary>
    [JsonPropertyName("time_ms")]
    public ulong? TimeMs { get; set; }

    [JsonPropertyName("text")]
    public string Text { get; set; } = string.Empty;
}

/// <summary>歌词，与 core::models::Lyric 对齐。</summary>
public sealed class Lyric
{
    [JsonPropertyName("lines")]
    public List<LyricLine> Lines { get; set; } = new();

    [JsonPropertyName("translation")]
    public List<LyricLine>? Translation { get; set; }
}

/// <summary>聚合搜索结果，与 core::models::SearchResult 对齐。</summary>
public sealed class SearchResult
{
    [JsonPropertyName("keyword")]
    public string Keyword { get; set; } = string.Empty;

    [JsonPropertyName("songs")]
    public List<Song> Songs { get; set; } = new();

    [JsonPropertyName("albums")]
    public List<Album> Albums { get; set; } = new();

    [JsonPropertyName("artists")]
    public List<Artist> Artists { get; set; } = new();

    [JsonPropertyName("total")]
    public ulong Total { get; set; }

    [JsonPropertyName("page")]
    public uint Page { get; set; }

    [JsonPropertyName("page_size")]
    public uint PageSize { get; set; }
}

/// <summary>排行榜，与 core::models::Leaderboard 对齐。</summary>
public sealed class Leaderboard
{
    [JsonPropertyName("id")]
    public string Id { get; set; } = string.Empty;

    [JsonPropertyName("source_id")]
    public string? SourceId { get; set; }

    [JsonPropertyName("name")]
    public string Name { get; set; } = string.Empty;

    [JsonPropertyName("cover_url")]
    public string? CoverUrl { get; set; }

    [JsonPropertyName("songs")]
    public List<Song> Songs { get; set; } = new();
}
