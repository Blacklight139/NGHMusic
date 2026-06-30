// MARK: - MusicCoreBridge
// 职责：iOS 通过 UniFFI 生成的 Swift 绑定调用 music-core（Rust）。
//
// FFI 方案：UniFFI。
// 启用步骤（需在 core 增加 UniFFI 依赖并生成绑定）：
// 1. 在 core/Cargo.toml 增加：
//    [dependencies]
//    uniffi = { version = "0.27", features = ["cli"] }
//    [build-dependencies]
//    uniffi = { version = "0.27", features = ["build"] }
// 2. 在 core/ 新建 build.rs：
//    fn main() { uniffi::generate_scaffolding("./src/ffi.udl").unwrap(); }
// 3. 在 core/src 新建 ffi.udl（模板见 ios/uniffi/music_core.udl），
//    并在 lib.rs 末尾加：uniffi::include_scaffolding!("ffi");
// 4. 生成 Swift 绑定：
//    cargo install uniffi-bindgen-cli
//    uniffi-bindgen generate src/ffi.udl --language swift --out-dir ./generated/swift
//    得到 MusicCoreFFI.swift + music_coreFFI.h + music_coreFFI.modulemap + libmusic_core.a
// 5. 将上述产物导入 Xcode 工程，并在 Build Settings 配置 modulemap 路径
//    （Other Swift Flags 加 -I<path>，Library Search Paths 加 .a 路径）。
//
// 说明：本文件假设 UniFFI 生成的模块名为 MusicCoreFFI（namespace music_core）。
//       若实际绑定未生成，所有调用会回退占位实现（返回空结果），保证脚手架可编译。

import Foundation

enum MusicCoreError: Error { case bridgeUnavailable(String) }

enum MusicCore {
    /// 返回核心库版本号
    static func appVersion() -> String {
        // 实际启用后：return MusicCoreFFI.appVersion()
        #if canImport(MusicCoreFFI)
        return MusicCoreFFI.appVersion()
        #else
        // 占位：music-core 未链接时
        return "0.1.0-scaffold"
        #endif
    }

    /// 导入音源 JSON，返回导入结果（成功/失败描述）
    static func importSource(_ json: String) async throws -> String {
        #if canImport(MusicCoreFFI)
        return try MusicCoreFFI.importSource(json: json)
        #else
        // 占位：模拟校验通过
        return "（占位）音源 JSON 已接收，长度 \(json.count) 字符；链接 music-core 后生效"
        #endif
    }

    /// 聚合搜索
    static func search(_ keyword: String, page: UInt32, pageSize: UInt32) async throws -> SearchResult {
        #if canImport(MusicCoreFFI)
        return try MusicCoreFFI.search(keyword: keyword, page: page, pageSize: pageSize)
        #else
        // 占位
        throw MusicCoreError.bridgeUnavailable("music-core 未链接，搜索不可用")
        #endif
    }

    /// 列出本地音乐
    static func listLocalSongs() async throws -> [Song] {
        #if canImport(MusicCoreFFI)
        return try MusicCoreFFI.listLocalSongs()
        #else
        // 占位
        throw MusicCoreError.bridgeUnavailable("music-core 未链接，本地音乐不可用")
        #endif
    }

    /// 播放（解析可播放 URL）
    static func play(songId: String) async throws -> String {
        #if canImport(MusicCoreFFI)
        return try MusicCoreFFI.play(songId: songId)
        #else
        throw MusicCoreError.bridgeUnavailable("music-core 未链接，播放不可用")
        #endif
    }
}
