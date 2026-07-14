// 职责：Android 数据模型，与 music-core（core/src/models.rs）对齐。
// 说明：UniFFI 生成 Kotlin 绑定时会生成等价类型（com.musicplayer.core.Song）；
//       本文件提供脚手架阶段的占位模型（纯 data class + @SerializedName），
//       字段名（snake_case）与 Rust 端 serde 序列化保持一致，便于在 music-core 未链接时
//       由 MusicRepository 直接解析核心返回的 JSON 字符串。

package com.musicplayer.app.models

import com.google.gson.annotations.SerializedName

/**
 * 播放模式，与 core/src/models.rs::PlayMode 对齐（snake_case 序列化）。
 * - SEQUENTIAL -> "sequential"
 * - SINGLE_LOOP -> "single_loop"
 * - RANDOM -> "random"
 */
enum class PlayMode {
    @SerializedName("sequential") SEQUENTIAL,
    @SerializedName("single_loop") SINGLE_LOOP,
    @SerializedName("random") RANDOM
}

data class Song(
    @SerializedName("id") val id: String,
    @SerializedName("source_id") val sourceId: String,
    @SerializedName("title") val title: String,
    @SerializedName("artists") val artists: List<String>,
    @SerializedName("album") val album: String? = null,
    @SerializedName("cover_url") val coverUrl: String? = null,
    @SerializedName("duration_ms") val durationMs: Long? = null,
    @SerializedName("lyric_url") val lyricUrl: String? = null,
    @SerializedName("play_url") val playUrl: String? = null,
    @SerializedName("local_path") val localPath: String? = null,
    @SerializedName("origin") val origin: SongOrigin
)

/**
 * 歌曲来源类型，使用内部标签 `type` 区分（与 core 的 `#[serde(tag = "type")]` 对齐）。
 * Gson 原生不支持内部标签多态，序列化/反序列化由 MusicRepository 中注册的
 * SongOriginTypeAdapter 处理：读取 JSON 的 "type" 字段后委托 Gson 解析对应变体字段。
 *
 * 注意：Nas 变体字段为 `protocol`（与 Rust 端一致），非 `protocol_name`（参见 bug-report.md HAR-003）。
 */
sealed class SongOrigin {
    data class Online(
        @SerializedName("source_id") val sourceId: String,
        @SerializedName("play_url") val playUrl: String
    ) : SongOrigin()

    data class Local(
        @SerializedName("path") val path: String
    ) : SongOrigin()

    data class Nas(
        @SerializedName("protocol") val protocol: String,
        @SerializedName("url") val url: String
    ) : SongOrigin()
}

data class SearchResult(
    @SerializedName("keyword") val keyword: String,
    @SerializedName("songs") val songs: List<Song>,
    @SerializedName("albums") val albums: List<Album>,
    @SerializedName("artists") val artists: List<Artist>,
    @SerializedName("total") val total: Long,
    @SerializedName("page") val page: Int,
    @SerializedName("page_size") val pageSize: Int
)

data class Album(
    @SerializedName("id") val id: String,
    @SerializedName("source_id") val sourceId: String,
    @SerializedName("name") val name: String,
    @SerializedName("artists") val artists: List<String>,
    @SerializedName("cover_url") val coverUrl: String? = null,
    @SerializedName("song_ids") val songIds: List<String>
)

data class Artist(
    @SerializedName("id") val id: String,
    @SerializedName("source_id") val sourceId: String,
    @SerializedName("name") val name: String,
    @SerializedName("avatar_url") val avatarUrl: String? = null,
    @SerializedName("song_ids") val songIds: List<String>
)

data class Leaderboard(
    @SerializedName("id") val id: String,
    @SerializedName("source_id") val sourceId: String,
    @SerializedName("name") val name: String,
    @SerializedName("cover_url") val coverUrl: String? = null,
    @SerializedName("songs") val songs: List<Song>
)

/** 单行歌词（LRC 时间轴）；timeMs 为 null 表示无时间戳的纯文本行。 */
data class LyricLine(
    @SerializedName("time_ms") val timeMs: Long?,
    @SerializedName("text") val text: String
)

/** 歌词，可带翻译。与 core/src/models.rs::Lyric 对齐。 */
data class Lyric(
    @SerializedName("lines") val lines: List<LyricLine>,
    @SerializedName("translation") val translation: List<LyricLine>? = null
)

/**
 * 音源信息，与 music-core（core/src/source_manager.rs）SourceInfo 对齐。
 * sourceType 取值："json"（用户导入）/ "community"（社区）/ "local"（本地）。
 * 说明：UniFFI 生成 Kotlin 绑定后会生成等价类型（com.musicplayer.core.SourceInfo），
 *       本 data class 为脚手架占位，字段与核心保持一致便于 UI 在绑定生成前编译。
 */
data class SourceInfo(
    @SerializedName("id") val id: String,
    @SerializedName("name") val name: String,
    @SerializedName("version") val version: String,
    @SerializedName("enabled") val enabled: Boolean,
    @SerializedName("source_type") val sourceType: String,
    @SerializedName("priority") val priority: Int,
    @SerializedName("description") val description: String? = null
)
