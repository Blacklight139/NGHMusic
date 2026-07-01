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

// MARK: - SourceInfo（音源信息，对应 Rust SourceInfo）
// 字段与 music_core.udl 中 SourceInfo dictionary 对齐（snake_case → camelCase 由 UniFFI 生成）。
// 脚手架阶段（MusicCoreFFI 未链接）作为占位模型；UniFFI 启用后由生成代码替换。
struct SourceInfo: Identifiable, Equatable, Codable {
    let id: String
    var name: String
    var version: String
    var enabled: Bool
    var sourceType: String        // json / community / local
    var priority: Int32
    var description: String

    private enum CodingKeys: String, CodingKey {
        case id, name, version, enabled
        case sourceType = "source_type"
        case priority, description
    }
}

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

    // MARK: - 音源管理（LXMusic 风格）
    // 对应 music_core.udl 中 list_sources_ordered / update_source_priority /
    // reorder_sources / delete_source / set_source_enabled / import_source_from_json。
    // UniFFI 生成后函数名形如 MusicCoreFFI.listSourcesOrdered()，
    // 参数名 camelCase（newPriority / orderedIds / jsonStr）。

    /// 列出所有音源（按优先级升序）
    static func listSourcesOrdered() async throws -> [SourceInfo] {
        #if canImport(MusicCoreFFI)
        return try MusicCoreFFI.listSourcesOrdered()
        #else
        // 占位：返回示例音源以便 UI 验证（链接 music-core 后替换为真实数据）
        return [
            SourceInfo(id: "demo-json", name: "示例在线音源", version: "1.0.0",
                       enabled: true, sourceType: "json", priority: 10,
                       description: "脚手架占位音源（json），链接核心库后替换为真实数据"),
            SourceInfo(id: "demo-community", name: "社区音源示例", version: "0.3.2",
                       enabled: false, sourceType: "community", priority: 20,
                       description: "脚手架占位音源（community）"),
            SourceInfo(id: "demo-local", name: "本地音源示例", version: "1.0.0",
                       enabled: true, sourceType: "local", priority: 30,
                       description: "脚手架占位音源（local）"),
        ]
        #endif
    }

    /// 更新单个音源优先级
    static func updateSourcePriority(id: String, newPriority: Int32) async throws {
        #if canImport(MusicCoreFFI)
        try MusicCoreFFI.updateSourcePriority(id: id, newPriority: newPriority)
        #else
        // 占位：no-op
        #endif
    }

    /// 按给定 id 顺序重排音源
    static func reorderSources(orderedIds: [String]) async throws {
        #if canImport(MusicCoreFFI)
        try MusicCoreFFI.reorderSources(orderedIds: orderedIds)
        #else
        // 占位：no-op
        #endif
    }

    /// 删除指定音源
    static func deleteSource(id: String) async throws {
        #if canImport(MusicCoreFFI)
        try MusicCoreFFI.deleteSource(id: id)
        #else
        // 占位：no-op
        #endif
    }

    /// 启用 / 禁用指定音源
    static func setSourceEnabled(id: String, enabled: Bool) async throws {
        #if canImport(MusicCoreFFI)
        try MusicCoreFFI.setSourceEnabled(id: id, enabled: enabled)
        #else
        // 占位：no-op
        #endif
    }

    /// 从 JSON 字符串导入音源，返回导入后的 SourceInfo
    static func importSourceFromJson(_ json: String) async throws -> SourceInfo {
        #if canImport(MusicCoreFFI)
        return try MusicCoreFFI.importSourceFromJson(jsonStr: json)
        #else
        // 占位：模拟导入成功
        return SourceInfo(id: "imported-\(Int(Date().timeIntervalSince1970))",
                          name: "已导入音源", version: "1.0.0", enabled: true,
                          sourceType: "json", priority: 999,
                          description: "脚手架占位：JSON 长度 \(json.count) 字符")
        #endif
    }
}
