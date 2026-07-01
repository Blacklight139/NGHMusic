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
}
