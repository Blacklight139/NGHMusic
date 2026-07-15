// 职责：Android 仓库层，包装 MusicCoreBridge（JNI/UniFFI）并提供协程 suspend 函数。
// 对齐 docs/api：将核心返回的 JSON 字符串解析为强类型数据模型（Song / SearchResult / Lyric / Leaderboard 等）。
// 设计：
// - 单例 object，持有预配置 Gson（注册 SongOriginTypeAdapter 处理内部标签多态）。
// - 所有方法为 suspend，内部已通过 MusicCoreBridge 切到 Dispatchers.IO，无需调用方再切换。
// - 解析失败返回 null（由 UI 显示提示），不抛异常打断流程。
// 集成 music-core 后无需改动本文件签名：MusicCoreBridge 内部替换为 UniFFI 调用即可。

package com.musicplayer.app.repository

import com.google.gson.Gson
import com.google.gson.GsonBuilder
import com.google.gson.JsonParser
import com.google.gson.TypeAdapter
import com.google.gson.TypeAdapterFactory
import com.google.gson.reflect.TypeToken
import com.google.gson.stream.JsonReader
import com.google.gson.stream.JsonWriter
import com.musicplayer.app.bridge.MusicCoreBridge
import com.musicplayer.app.models.Leaderboard
import com.musicplayer.app.models.Lyric
import com.musicplayer.app.models.SearchResult
import com.musicplayer.app.models.Song
import com.musicplayer.app.models.SongOrigin
import com.musicplayer.app.models.SourceInfo

object MusicRepository {

    /**
     * SongOrigin 内部标签多态适配器：与 core 的 `#[serde(tag = "type")]` 对齐。
     * 读取 JSON 的 "type" 字段后委托 Gson 解析对应变体字段。
     */
    private object SongOriginTypeAdapter : TypeAdapter<SongOrigin>() {
        override fun write(out: JsonWriter, value: SongOrigin?) {
            if (value == null) {
                out.nullValue()
                return
            }
            out.beginObject()
            when (value) {
                is SongOrigin.Online -> {
                    out.name("type").value("Online")
                    out.name("source_id").value(value.sourceId)
                    out.name("play_url").value(value.playUrl)
                }
                is SongOrigin.Local -> {
                    out.name("type").value("Local")
                    out.name("path").value(value.path)
                }
                is SongOrigin.Nas -> {
                    out.name("type").value("Nas")
                    out.name("protocol").value(value.protocol)
                    out.name("url").value(value.url)
                }
            }
            out.endObject()
        }

        override fun read(reader: JsonReader): SongOrigin? {
            reader.beginObject()
            var type: String? = null
            // 先缓冲读取所有字段，遇到 type 时确定变体。
            // 为简化实现，借助 Gson 解析变体：先 peek 出 type，再按变体读取。
            // 这里采用：读到 "type" 字段记录值，其余字段累积为原始 JSON 后委托 Gson。
            // 直接顺序读取更可靠，按变体逐字段解析。
            var onlineSourceId: String? = null
            var onlinePlayUrl: String? = null
            var localPath: String? = null
            var nasProtocol: String? = null
            var nasUrl: String? = null
            while (reader.hasNext()) {
                val name = reader.nextName()
                when (name) {
                    "type" -> type = reader.nextString()
                    "source_id" -> onlineSourceId = reader.nextString()
                    "play_url" -> onlinePlayUrl = if (reader.peek() == com.google.gson.stream.JsonToken.NULL) {
                        reader.nextNull(); ""
                    } else {
                        reader.nextString()
                    }
                    "path" -> localPath = reader.nextString()
                    "protocol" -> nasProtocol = reader.nextString()
                    "url" -> nasUrl = reader.nextString()
                    else -> reader.skipValue()
                }
            }
            reader.endObject()
            return when (type) {
                "Online" -> SongOrigin.Online(onlineSourceId ?: "", onlinePlayUrl ?: "")
                "Local" -> SongOrigin.Local(localPath ?: "")
                "Nas" -> SongOrigin.Nas(nasProtocol ?: "", nasUrl ?: "")
                // 6.1 未知 type 不再返回 null（会导致非空 SongOrigin 字段 NPE），
                //      回退为 Online 空值，保证字段非空。
                else -> SongOrigin.Online("", "")
            }
        }
    }

    private object SongOriginFactory : TypeAdapterFactory {
        override fun <T> create(gson: Gson, type: TypeToken<T>): TypeAdapter<T>? {
            if (type.rawType !== SongOrigin::class.java) return null
            @Suppress("UNCHECKED_CAST")
            return SongOriginTypeAdapter as TypeAdapter<T>
        }
    }

    private val gson: Gson = GsonBuilder()
        .registerTypeAdapterFactory(SongOriginFactory)
        .serializeNulls()
        .create()

    // ---- 基础信息 ----

    /** 核心版本号。 */
    suspend fun appVersion(): String = MusicCoreBridge.appVersion()

    // ---- 音源管理 ----

    /** 列出全部音源（按 priority 升序）。 */
    suspend fun listSourcesOrdered(): List<SourceInfo> = MusicCoreBridge.listSourcesOrdered()

    /** 更新单个音源优先级。 */
    suspend fun updateSourcePriority(id: String, newPriority: Int) =
        MusicCoreBridge.updateSourcePriority(id, newPriority)

    /** 按给定顺序重排音源。 */
    suspend fun reorderSources(orderedIds: List<String>) =
        MusicCoreBridge.reorderSources(orderedIds)

    /** 删除音源。 */
    suspend fun deleteSource(id: String) = MusicCoreBridge.deleteSource(id)

    /** 启用/禁用音源。 */
    suspend fun setSourceEnabled(id: String, enabled: Boolean) =
        MusicCoreBridge.setSourceEnabled(id, enabled)

    /** 从 JSON 字符串导入音源，返回新建音源信息。 */
    suspend fun importSourceFromJson(jsonStr: String): SourceInfo =
        MusicCoreBridge.importSourceFromJson(jsonStr)

    /** 校验音源 JSON 是否符合标准 Schema。返回原始 JSON 字符串（{"valid":...,"errors":[...]}）。 */
    suspend fun sourceValidate(json: String): String? = MusicCoreBridge.sourceValidate(json)

    // ---- 搜索与歌曲 ----

    /** 聚合搜索，返回强类型 SearchResult；解析失败返回 null。 */
    suspend fun search(keyword: String, page: Int = 1, pageSize: Int = 20): SearchResult? {
        val raw = MusicCoreBridge.search(keyword, page, pageSize) ?: return null
        return parse(raw, SearchResult::class.java)
    }

    /** 列出本地音乐，返回强类型 Song 列表；解析失败返回 null。 */
    suspend fun listLocalSongs(): List<Song>? {
        val raw = MusicCoreBridge.listLocalSongs() ?: return null
        return parseList(raw)
    }

    /** 解析并返回可播放 URL（songId 维度）。 */
    suspend fun play(songId: String): String? = MusicCoreBridge.play(songId)

    /** 获取歌曲完整元数据，返回强类型 Song；解析失败返回 null。 */
    suspend fun getMetadata(sourceId: String, songId: String): Song? {
        val raw = MusicCoreBridge.getMetadata(sourceId, songId) ?: return null
        return parse(raw, Song::class.java)
    }

    /** 获取可播放 URL，返回原始 JSON（{"url":...,"cached":...}）。 */
    suspend fun getPlayUrl(sourceId: String, songId: String): String? =
        MusicCoreBridge.getPlayUrl(sourceId, songId)

    /** 获取歌词，返回强类型 Lyric；解析失败返回 null。 */
    suspend fun getLyric(sourceId: String, songId: String): Lyric? {
        val raw = MusicCoreBridge.getLyric(sourceId, songId) ?: return null
        return parse(raw, Lyric::class.java)
    }

    /** 获取排行榜列表，返回强类型 Leaderboard 列表；解析失败返回 null。 */
    suspend fun getLeaderboards(sourceId: String): List<Leaderboard>? {
        val raw = MusicCoreBridge.getLeaderboards(sourceId) ?: return null
        return parseList(raw)
    }

    // ---- 飞牛 NAS（返回原始 JSON，UI 按需解析）----

    suspend fun feiniuLogin(baseUrl: String, username: String, password: String): String? =
        MusicCoreBridge.feiniuLogin(baseUrl, username, password)

    suspend fun feiniuListFiles(path: String): String? = MusicCoreBridge.feiniuListFiles(path)

    suspend fun feiniuStream(path: String): String? = MusicCoreBridge.feiniuStream(path)

    suspend fun feiniuHealth(): String? = MusicCoreBridge.feiniuHealth()

    // ---- 协议源（返回原始 JSON，UI 按需解析）----

    suspend fun protocolAdd(configJson: String): String? = MusicCoreBridge.protocolAdd(configJson)

    suspend fun protocolList(): String? = MusicCoreBridge.protocolList()

    suspend fun protocolDelete(id: String): String? = MusicCoreBridge.protocolDelete(id)

    suspend fun protocolListFiles(id: String, path: String): String? =
        MusicCoreBridge.protocolListFiles(id, path)

    suspend fun protocolRead(id: String, path: String): String? =
        MusicCoreBridge.protocolRead(id, path)

    suspend fun protocolStream(id: String, path: String): String? =
        MusicCoreBridge.protocolStream(id, path)

    // ---- 本地音乐管理 ----

    suspend fun localInit(dbPath: String): String? = MusicCoreBridge.localInit(dbPath)

    suspend fun localAddDir(dir: String): String? = MusicCoreBridge.localAddDir(dir)

    suspend fun localRescan(): String? = MusicCoreBridge.localRescan()

    suspend fun localProgress(): String? = MusicCoreBridge.localProgress()

    // ---- 缓存 ----

    suspend fun cacheInit(cacheDir: String, maxBytes: Long): String? =
        MusicCoreBridge.cacheInit(cacheDir, maxBytes)

    suspend fun cacheStats(): String? = MusicCoreBridge.cacheStats()

    suspend fun cacheClear(): String? = MusicCoreBridge.cacheClear()

    // ---- 解析工具 ----

    /** 解析单个对象；非法 JSON 或类型不匹配返回 null。 */
    private fun <T> parse(json: String, clazz: Class<T>): T? = try {
        gson.fromJson(json, clazz)
    } catch (e: Exception) {
        // 6.2 捕获所有解析异常（含 JsonSyntaxException / IllegalStateException / JsonParseException 等），
        //     不再仅捕获 JsonSyntaxException。
        null
    }

    /** 解析数组；非法 JSON 或类型不匹配返回 null。 */
    private fun <T> parseList(json: String): List<T>? {
        val type = TypeToken.getParameterized(List::class.java, inferElementType(json)).type
        return try {
            @Suppress("UNCHECKED_CAST")
            gson.fromJson<List<T>>(json, type)
        } catch (e: Exception) {
            // 6.2 捕获所有解析异常，统一返回 null。
            null
        }
    }

    /**
     * 简单推断数组元素类型：根据首个对象的字段猜测 Song / Leaderboard。
     * 这里为搜索/本地歌曲/排行榜三类数组服务：含 "songs" 字段视为 Leaderboard，
     * 其余回退为 Song。
     *
     * 6.5 修复：原实现使用 String.contains 全文匹配，会把含 "songs" 子串的非对象元素误判；
     *           改用 JsonParser 解析首元素并检查 "songs" 字段。
     */
    private fun inferElementType(json: String): Class<*> {
        return try {
            val element = JsonParser.parseString(json.trim())
            if (!element.isJsonArray) return Song::class.java
            val array = element.asJsonArray
            if (array.size() == 0) return Song::class.java
            val first = array[0]
            if (first.isJsonObject && first.asJsonObject.has("songs")) {
                Leaderboard::class.java
            } else {
                Song::class.java
            }
        } catch (e: Exception) {
            Song::class.java
        }
    }
}
