// 职责：Android 数据模型，与 music-core（core/src/models.rs）对齐。
// 说明：UniFFI 生成 Kotlin 绑定时会生成等价类型（com.musicplayer.core.Song）；
//       本文件提供脚手架阶段的占位模型（纯 data class，不引入序列化插件），便于 UI 在绑定生成前编译。

package com.musicplayer.app.models

enum class PlayMode { SEQUENTIAL, SINGLE_LOOP, RANDOM }

data class Song(
    val id: String,
    val sourceId: String,
    val title: String,
    val artists: List<String>,
    val album: String? = null,
    val coverUrl: String? = null,
    val durationMs: Long? = null,
    val lyricUrl: String? = null,
    val playUrl: String? = null,
    val localPath: String? = null,
    val origin: SongOrigin
)

sealed class SongOrigin {
    data class Online(val sourceId: String, val playUrl: String) : SongOrigin()
    data class Local(val path: String) : SongOrigin()
    data class Nas(val protocolName: String, val url: String) : SongOrigin()
}

data class SearchResult(
    val keyword: String,
    val songs: List<Song>,
    val albums: List<Album>,
    val artists: List<Artist>,
    val total: Long,
    val page: Int,
    val pageSize: Int
)

data class Album(
    val id: String,
    val sourceId: String,
    val name: String,
    val artists: List<String>,
    val coverUrl: String? = null,
    val songIds: List<String>
)

data class Artist(
    val id: String,
    val sourceId: String,
    val name: String,
    val avatarUrl: String? = null,
    val songIds: List<String>
)

data class Leaderboard(
    val id: String,
    val sourceId: String,
    val name: String,
    val coverUrl: String? = null,
    val songs: List<Song>
)

/**
 * 音源信息，与 music-core（core/src/source_manager.rs）SourceInfo 对齐。
 * sourceType 取值："json"（用户导入）/ "community"（社区）/ "local"（本地）。
 * 说明：UniFFI 生成 Kotlin 绑定后会生成等价类型（com.musicplayer.core.SourceInfo），
 *       本 data class 为脚手架占位，字段与核心保持一致便于 UI 在绑定生成前编译。
 */
data class SourceInfo(
    val id: String,
    val name: String,
    val version: String,
    val enabled: Boolean,
    val sourceType: String,
    val priority: Int,
    val description: String? = null
)
