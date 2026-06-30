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

    /** 导入音源 JSON */
    suspend fun importSource(json: String): String = withContext(Dispatchers.IO) {
        // 实际启用后：com.musicplayer.core.MusicCore.importSource(json)
        "（占位）音源 JSON 已接收，长度 ${json.length} 字符；链接 music-core 后生效"
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
