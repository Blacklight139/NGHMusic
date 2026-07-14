// 职责：Android 通过 JNI/UniFFI 调用 music-core（Rust）的 Kotlin 桥接层。
//
// FFI 方案：UniFFI（生成 Kotlin 绑定 + JNI 自动加载 libmusic_core.so）。
// 启用步骤（需在 core 增加 UniFFI 依赖并生成绑定）：
// 1. 在 core/Cargo.toml 增加：
//    [dependencies] uniffi = { version = "0.27", features = ["cli"] }
//    [build-dependencies] uniffi = { version = "0.27", features = ["build"] }
// 2. 在 core/src/lib.rs 末尾追加：uniffi::include_scaffolding!("ffi");
//    并在 core/ 新建 build.rs：fn main() { uniffi::generate_scaffolding("./src/ffi.udl").unwrap(); }
// 3. UDL 模板见 ios/uniffi/music_core.udl（三端共用同一份）。
// 4. 生成 Kotlin 绑定：
//    cargo install uniffi-bindgen-cli
//    uniffi-bindgen generate core/src/ffi.udl --language kotlin \
//      --out-dir android/app/src/main/java/com/musicplayer/core
//    得到 com/musicplayer/core/musiccore.kt（包名 com.musicplayer.core）
// 5. 交叉编译各 ABI 的 libmusic_core.so 放入：
//    android/app/src/main/jniLibs/<abi>/libmusic_core.so
//    （arm64-v8a / armeabi-v7a / x86_64）
// 6. 在 app/build.gradle.kts 的 android.defaultConfig.ndk.abiFilters 中配置对应 ABI。
//
// 说明：本桥接假设生成的对象为 com.musicplayer.core.MusicCore。
//       若绑定未生成，调用降级为占位实现（返回空结果），保证脚手架可编译。

package com.musicplayer.app.bridge

import com.musicplayer.app.models.*
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

object MusicCoreBridge {

    /** 返回核心库版本号 */
    fun appVersion(): String {
        return try {
            // 实际启用后：com.musicplayer.core.MusicCore.appVersion()
            // 占位：music-core 未链接时
            "0.1.0-scaffold"
        } catch (e: Throwable) {
            "unknown"
        }
    }

    /** 导入音源 JSON（旧入口，保留向后兼容；UI 已改用 importSourceFromJson） */
    suspend fun importSource(json: String): String = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.importSource(json)
        "（占位）音源 JSON 已接收，长度 ${json.length} 字符；链接 music-core 后生效"
    }

    // ---- 音源管理（SourceManager）----
    // 实际启用后，下列方法均替换为 com.musicplayer.core.MusicCore.* 调用。
    // 占位实现：在内存中维护一份音源列表，使 LXMusic 风格 UI 在 music-core 未链接时即可演示；
    // 链接 music-core 后删除 placeholderSources 及相关逻辑，改为直接调用 UniFFI 生成函数。
    private val placeholderSources = mutableListOf(
        SourceInfo("src_json_1", "网易云 JSON 音源", "1.0.0", true, "json", 0,
            "由用户导入的标准 JSON Schema 音源"),
        SourceInfo("src_comm_1", "社区音源 A", "2.3.1", true, "community", 1,
            "来自社区仓库的共享音源"),
        SourceInfo("src_local_1", "本地音乐音源", "0.9.0", false, "local", 2,
            "扫描本机存储的音源")
    )
    private var placeholderSeq = 100

    /** 列出全部音源（按 priority 升序）。实际启用后：MusicCore.listSourcesOrdered() */
    suspend fun listSourcesOrdered(): List<SourceInfo> = withContext(Dispatchers.IO) {
        placeholderSources.sortedBy { it.priority }.toList()
    }

    /** 更新单个音源优先级。实际启用后：MusicCore.updateSourcePriority(id, newPriority) */
    suspend fun updateSourcePriority(id: String, newPriority: Int) = withContext(Dispatchers.IO) {
        val idx = placeholderSources.indexOfFirst { it.id == id }
        if (idx >= 0) placeholderSources[idx] = placeholderSources[idx].copy(priority = newPriority)
    }

    /** 按给定顺序重排音源。实际启用后：MusicCore.reorderSources(orderedIds) */
    suspend fun reorderSources(orderedIds: List<String>) = withContext(Dispatchers.IO) {
        orderedIds.forEachIndexed { i, id ->
            val idx = placeholderSources.indexOfFirst { it.id == id }
            if (idx >= 0) placeholderSources[idx] = placeholderSources[idx].copy(priority = i)
        }
    }

    /** 删除音源。实际启用后：MusicCore.deleteSource(id) */
    suspend fun deleteSource(id: String) = withContext(Dispatchers.IO) {
        placeholderSources.removeAll { it.id == id }
        Unit
    }

    /** 启用/禁用音源。实际启用后：MusicCore.setSourceEnabled(id, enabled) */
    suspend fun setSourceEnabled(id: String, enabled: Boolean) = withContext(Dispatchers.IO) {
        val idx = placeholderSources.indexOfFirst { it.id == id }
        if (idx >= 0) placeholderSources[idx] = placeholderSources[idx].copy(enabled = enabled)
    }

    /** 从 JSON 字符串导入音源，返回新建音源信息。实际启用后：MusicCore.importSourceFromJson(jsonStr) */
    suspend fun importSourceFromJson(jsonStr: String): SourceInfo = withContext(Dispatchers.IO) {
        // 占位：基于 JSON 内容长度构造一个新音源并加入列表尾部
        val n = placeholderSeq++
        val info = SourceInfo(
            id = "src_json_$n",
            name = "导入音源 ${n - 100}",
            version = "1.0.0",
            enabled = true,
            sourceType = "json",
            priority = placeholderSources.size,
            description = "由文件导入（${jsonStr.length} 字符）"
        )
        placeholderSources.add(info)
        info
    }

    /** 聚合搜索 */
    suspend fun search(keyword: String, page: Int, pageSize: Int): SearchResult? =
        withContext(Dispatchers.IO) {
            // 实际启用后：com.musicplayer.core.MusicCore.search(keyword, page.toUInt(), pageSize.toUInt())
            // 占位：返回 null，由 UI 显示提示
            null
        }

    /** 列出本地音乐 */
    suspend fun listLocalSongs(): List<Song>? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.listLocalSongs()
        null
    }

    /** 解析并返回可播放 URL */
    suspend fun play(songId: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.play(songId)
        null
    }

    // ---- 音源校验（SourceManager）----

    /** 校验音源 JSON 是否符合标准 Schema，返回 {"valid":bool,"errors":[...]} JSON。 */
    suspend fun sourceValidate(json: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.sourceValidate(json)
        null
    }

    // ---- 歌曲元数据/歌词/排行榜 ----
    // 实际启用后均替换为 com.musicplayer.core.MusicCore.* 调用，返回核心序列化的 JSON 字符串。

    /** 获取指定音源下歌曲的完整元数据，返回 Song JSON。 */
    suspend fun getMetadata(sourceId: String, songId: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.getMetadata(sourceId, songId)
        null
    }

    /** 获取指定音源下歌曲的可播放 URL，返回 {"url":...,"cached":false}。 */
    suspend fun getPlayUrl(sourceId: String, songId: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.getPlayUrl(sourceId, songId)
        null
    }

    /** 获取指定音源下歌曲的歌词，返回 Lyric JSON。 */
    suspend fun getLyric(sourceId: String, songId: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.getLyric(sourceId, songId)
        null
    }

    /** 获取指定音源的排行榜列表，返回 Leaderboard 数组 JSON。 */
    suspend fun getLeaderboards(sourceId: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.getLeaderboards(sourceId)
        null
    }

    // ---- 飞牛 NAS ----
    // 实际启用后替换为 com.musicplayer.core.MusicCore.feiniu* 调用，返回核心 JSON。

    /** 登录飞牛 NAS，返回 {"token":...,"base_url":...}。 */
    suspend fun feiniuLogin(baseUrl: String, username: String, password: String): String? =
        withContext(Dispatchers.IO) {
            // 实际启用后：com.musicplayer.core.MusicCore.feiniuLogin(baseUrl, username, password)
            null
        }

    /** 列出飞牛 NAS 指定路径下的文件，返回 {"path":...,"files":[...]}。 */
    suspend fun feiniuListFiles(path: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.feiniuListFiles(path)
        null
    }

    /** 生成飞牛 NAS 文件的可流式播放 URL，返回 {"url":...}。 */
    suspend fun feiniuStream(path: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.feiniuStream(path)
        null
    }

    /** 飞牛服务健康检查，返回 {"healthy":bool,"base_url":...}。 */
    suspend fun feiniuHealth(): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.feiniuHealth()
        null
    }

    // ---- 协议源（SMB/WebDAV/FTP/DLNA/NFS）----
    // 实际启用后替换为 com.musicplayer.core.MusicCore.protocol* 调用，返回核心 JSON。
    // WebDAV/FTP 完整实现，SMB/DLNA/NFS 为占位实现（调用返回 Protocol 错误）。

    /** 添加一个远程协议源，返回 {"id":...,"protocol":...,"root":...,"enabled":true,"placeholder":bool}。 */
    suspend fun protocolAdd(configJson: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.protocolAdd(configJson)
        null
    }

    /** 列出已加载的协议源，返回 {"sources":[...]}。 */
    suspend fun protocolList(): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.protocolList()
        null
    }

    /** 删除指定 id 的协议源，返回 {"id":...,"deleted":bool}。 */
    suspend fun protocolDelete(id: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.protocolDelete(id)
        null
    }

    /** 列出协议源下指定路径的条目名称，返回 {"path":...,"entries":[...]}。 */
    suspend fun protocolListFiles(id: String, path: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.protocolListFiles(id, path)
        null
    }

    /** 读取协议源下指定文件为字节，返回 {"size":N,"data_base64":"..."}。 */
    suspend fun protocolRead(id: String, path: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.protocolRead(id, path)
        null
    }

    /** 生成协议源下指定文件的可流式播放 URL，返回 {"url":...}。 */
    suspend fun protocolStream(id: String, path: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.protocolStream(id, path)
        null
    }

    // ---- 本地音乐管理 ----
    // 实际启用后替换为 com.musicplayer.core.MusicCore.local* 调用，返回核心 JSON。

    /** 初始化本地音乐源（打开/创建 SQLite 索引库），返回 {"ok":true}。 */
    suspend fun localInit(dbPath: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.localInit(dbPath)
        null
    }

    /** 添加本地扫描目录并递归扫描入库，返回 {"ok":true}。 */
    suspend fun localAddDir(dir: String): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.localAddDir(dir)
        null
    }

    /** 重新扫描所有已添加目录（增量更新），返回 {"ok":true}。 */
    suspend fun localRescan(): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.localRescan()
        null
    }

    /** 返回本地扫描进度，返回 {"current_count":N,"scanning":bool}。 */
    suspend fun localProgress(): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.localProgress()
        null
    }

    // ---- 缓存 ----
    // 实际启用后替换为 com.musicplayer.core.MusicCore.cache* 调用，返回核心 JSON。

    /** 初始化播放缓存管理器，返回 {"ok":true}。 */
    suspend fun cacheInit(cacheDir: String, maxBytes: Long): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.cacheInit(cacheDir, maxBytes)
        null
    }

    /** 返回缓存统计，返回 {"entries":N,"total_bytes":N,"max_bytes":N}。 */
    suspend fun cacheStats(): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.cacheStats()
        null
    }

    /** 清空所有缓存文件与索引，返回 {"ok":true}。 */
    suspend fun cacheClear(): String? = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.cacheClear()
        null
    }
}
