// MARK: - MusicCoreBridge
// 职责：iOS 高层桥接层，封装 music-core 的 Swift FFI（手写 C ABI 封装），
//       将同步返回 JSON 字符串的 FFI 调用包装为 async 方法并解码为 Swift 模型。
//
// FFI 参考实现：/workspace/core/ffi-bindings/swift/MusicCore.swift
//   - 该文件通过 @_silgen_name 直接引用 core/src/ffi.rs 的 #[no_mangle] C 符号；
//   - 所有返回 *mut c_char 的 ABI 经 readAndFree 释放，错误以 {"error":{...}} JSON 表达；
//   - MusicCore.swift 提供同步 throwing Swift 封装（version/source_import/search/...）。
//
// 链接启用方式（在 Xcode 中）：
//   1. 将 core/ffi-bindings/swift/MusicCore.swift 与 module.modulemap 导入工程；
//   2. 链接 libmusic_core.a（cargo build --release --target aarch64-apple-ios 产物）；
//   3. 在 Build Settings -> Other Swift Flags 增加 -DNGH_LINK_MUSIC_CORE；
//   4. 此时本桥接的所有 #if NGH_LINK_MUSIC_CORE 分支生效，调用真实核心。
//
// 未定义 NGH_LINK_MUSIC_CORE 时（脚手架默认）：所有调用回退占位实现，
//   保证 UI 在未链接核心库时仍可编译运行（listSourcesOrdered/importSourceFromJson 返回示例数据，
//   其余能力抛 MusicCoreBridgeError.unavailable，由各页面降级为空/占位态）。
//
// 说明：本桥接类型名为 MusicCoreBridge（FFI 层类型为 MusicCore），二者不冲突。

import Foundation

// MARK: - MusicCoreBridgeError
/// 桥接层错误。LocalizedError 以便视图通过 localizedDescription 展示可读信息。
enum MusicCoreBridgeError: LocalizedError {
    /// 核心 FFI 未链接 / 能力未启用。
    case unavailable(String)
    /// FFI 返回错误或 JSON 解码失败等通用消息。
    case message(String)

    var errorDescription: String? {
        switch self {
        case let .unavailable(msg): return msg
        case let .message(msg): return msg
        }
    }
}

// MARK: - 桥接层 Codable 类型（FFI JSON 响应包装）

/// 音源信息（对应 Rust SourceInfo：id/name/version/enabled/source_type/priority/description）
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

/// 音源校验结果（FFI music_core_source_validate 返回 {"valid":bool,"errors":[...]}）
struct SourceValidationResult: Codable {
    let valid: Bool
    let errors: [String]
}

/// 可播放 URL 结果（FFI music_core_get_play_url 返回 {"url":...,"cached":bool,"play_url":...}）
private struct PlayUrlResult: Codable {
    let url: String
    let cached: Bool
    let playUrl: String?

    private enum CodingKeys: String, CodingKey {
        case url, cached
        case playUrl = "play_url"
    }
}

/// 通用流式 URL 结果（feiniu_stream / protocol_stream 返回 {"url":...}）
private struct StreamUrlResult: Codable {
    let url: String
}

/// 飞牛登录结果（{"token":...,"base_url":...}）
struct FeiniuLoginResult: Codable {
    let token: String
    let baseUrl: String

    private enum CodingKeys: String, CodingKey {
        case token
        case baseUrl = "base_url"
    }
}

/// 飞牛健康检查结果（{"healthy":bool,"base_url":...}）
struct FeiniuHealthResult: Codable {
    let healthy: Bool
    let baseUrl: String

    private enum CodingKeys: String, CodingKey {
        case healthy
        case baseUrl = "base_url"
    }
}

/// NAS 文件条目（对应 Rust NasFile，is_dir 兼容 isDir）
struct NasFile: Codable, Identifiable, Equatable {
    let name: String
    let isDir: Bool
    let size: UInt64
    let modified: String?

    var id: String { name }

    private enum CodingKeys: String, CodingKey {
        case name
        case isDir = "is_dir"
        case size, modified
    }
}

/// 飞牛列目录结果（{"path":...,"files":[NasFile...]}）
private struct FeiniuListFilesResult: Codable {
    let path: String
    let files: [NasFile]
}

/// 协议源条目（{"id":...,"protocol":...,"root":...,"enabled":bool,"placeholder":bool}）
struct ProtocolSource: Codable, Identifiable, Equatable {
    let id: String
    let protocolName: String
    let root: String
    let enabled: Bool
    let placeholder: Bool

    private enum CodingKeys: String, CodingKey {
        case id
        case protocolName = "protocol"
        case root, enabled, placeholder
    }
}

/// 协议源列表结果（{"sources":[...]}）
private struct ProtocolListResult: Codable {
    let sources: [ProtocolSource]
}

/// 协议列目录结果（{"path":...,"entries":[...]}）
private struct ProtocolListFilesResult: Codable {
    let path: String
    let entries: [String]
}

/// 协议读取结果（{"size":N,"data_base64":"..."}）
struct ProtocolReadResult: Codable {
    let size: Int
    let dataBase64: String

    private enum CodingKeys: String, CodingKey {
        case size
        case dataBase64 = "data_base64"
    }
}

/// 缓存统计（{"entries":N,"total_bytes":N,"max_bytes":N}）
struct CacheStats: Codable, Equatable {
    let entries: Int
    let totalBytes: UInt64
    let maxBytes: UInt64

    private enum CodingKeys: String, CodingKey {
        case entries
        case totalBytes = "total_bytes"
        case maxBytes = "max_bytes"
    }
}

/// 本地扫描进度（{"current_count":N,"scanning":bool}）
struct LocalProgress: Codable, Equatable {
    let currentCount: UInt64
    let scanning: Bool

    private enum CodingKeys: String, CodingKey {
        case currentCount = "current_count"
        case scanning
    }
}

// MARK: - MusicCoreBridge
enum MusicCoreBridge {

    // MARK: - 私有辅助

    /// 将同步 throwing FFI 调用派发到后台线程执行（核心内部 block_on 会阻塞），
    /// 并统一把任何抛出包装为 MusicCoreBridgeError，便于视图 localizedDescription 展示。
    private static func run<T>(_ block: @Sendable @escaping () throws -> T) async throws -> T {
        do {
            return try await Task.detached(priority: .userInitiated) { try block() }.value
        } catch {
            throw MusicCoreBridgeError.message("\(error)")
        }
    }

    /// 将 FFI 返回的 JSON 字符串解码为指定 Decodable 类型。
    private static func decode<T: Decodable>(_ type: T.Type, _ json: String) throws -> T {
        guard let data = json.data(using: .utf8) else {
            throw MusicCoreBridgeError.message("FFI 返回非 UTF-8 字符串")
        }
        do {
            return try JSONDecoder().decode(type, from: data)
        } catch {
            throw MusicCoreBridgeError.message("JSON 解码失败：\(error)")
        }
    }

    // MARK: - 版本

    /// 返回核心库版本号（同步，不涉及网络 IO）。
    static func appVersion() -> String {
        #if NGH_LINK_MUSIC_CORE
        return (try? MusicCore.version()) ?? "unknown"
        #else
        return "0.1.0-scaffold"
        #endif
    }

    // MARK: - 音源管理（docs/sound-source-api.md）

    /// 导入音源 JSON，返回导入后的 SourceInfo（FFI music_core_source_import 返回 SourceInfo JSON）。
    static func importSource(_ json: String) async throws -> SourceInfo {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.sourceImport(json)
            return try Self.decode(SourceInfo.self, result)
        }
        #else
        return Self.placeholderImported(json)
        #endif
    }

    /// LXMusic 风格入口：从 JSON 字符串导入音源（等价 importSource，保留旧 API 名以兼容 SettingsView）。
    static func importSourceFromJson(_ json: String) async throws -> SourceInfo {
        try await importSource(json)
    }

    /// 校验音源 JSON 是否符合标准 Schema（不加载、不持久化）。
    static func validateSource(_ json: String) async throws -> SourceValidationResult {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.sourceValidate(json)
            return try Self.decode(SourceValidationResult.self, result)
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，音源校验不可用")
        #endif
    }

    /// 列出所有音源（FFI music_core_source_list 返回 SourceInfo 数组，按 priority 降序）。
    static func listSources() async throws -> [SourceInfo] {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.sourceList()
            return try Self.decode([SourceInfo].self, result)
        }
        #else
        return Self.placeholderSources
        #endif
    }

    /// 列出所有音源（LXMusic 风格命名，等价 listSources）。
    static func listSourcesOrdered() async throws -> [SourceInfo] {
        try await listSources()
    }

    /// 启用 / 禁用指定音源（FFI music_core_source_enable / music_core_source_disable）。
    static func setSourceEnabled(id: String, enabled: Bool) async throws {
        #if NGH_LINK_MUSIC_CORE
        _ = try await run {
            enabled ? try MusicCore.sourceEnable(id) : try MusicCore.sourceDisable(id)
        }
        #else
        // 占位：no-op
        #endif
    }

    /// 删除指定音源（FFI music_core_source_delete）。
    static func deleteSource(id: String) async throws {
        #if NGH_LINK_MUSIC_CORE
        _ = try await run { try MusicCore.sourceDelete(id) }
        #else
        // 占位：no-op
        #endif
    }

    /// 按给定 id 顺序重排音源。
    /// 核心 FFI 当前未暴露优先级变更 ABI（docs 仅提供 enable/disable/delete/import），
    /// 故脚手架阶段为 no-op，保留接口以对齐 LXMusic 风格 UI 与 UDL 声明。
    static func reorderSources(orderedIds: [String]) async throws {
        _ = orderedIds
        // NGH_LINK_MUSIC_CORE：music-core 暂未提供 reorder_sources，本地顺序不持久化。
    }

    /// 更新单个音源优先级（同 reorderSources，核心 FFI 暂未暴露）。
    static func updateSourcePriority(id: String, newPriority: Int32) async throws {
        _ = id
        _ = newPriority
    }

    // MARK: - 搜索与歌曲（docs/sound-source-api.md）

    /// 聚合搜索（FFI music_core_search 返回 SearchResult JSON）。
    static func search(_ keyword: String, page: UInt32, pageSize: UInt32) async throws -> SearchResult {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.search(keyword, page: page, pageSize: pageSize)
            return try Self.decode(SearchResult.self, result)
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，搜索不可用")
        #endif
    }

    /// 获取歌曲完整元数据（FFI music_core_get_metadata 返回 Song JSON）。
    static func getMetadata(sourceId: String, songId: String) async throws -> Song {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.getMetadata(sourceId: sourceId, songId: songId)
            return try Self.decode(Song.self, result)
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，元数据不可用")
        #endif
    }

    /// 获取可播放 URL（FFI music_core_get_play_url 返回 {url,cached,play_url}，提取 url）。
    static func getPlayUrl(sourceId: String, songId: String) async throws -> String {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.getPlayUrl(sourceId: sourceId, songId: songId)
            return try Self.decode(PlayUrlResult.self, result).url
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，播放地址不可用")
        #endif
    }

    /// 获取歌词（FFI music_core_get_lyric 返回 Lyric JSON）。
    static func getLyric(sourceId: String, songId: String) async throws -> Lyric {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.getLyric(sourceId: sourceId, songId: songId)
            return try Self.decode(Lyric.self, result)
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，歌词不可用")
        #endif
    }

    /// 获取音源排行榜（FFI music_core_get_leaderboards 返回 Leaderboard 数组）。
    static func getLeaderboards(sourceId: String) async throws -> [Leaderboard] {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.getLeaderboards(sourceId: sourceId)
            return try Self.decode([Leaderboard].self, result)
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，排行榜不可用")
        #endif
    }

    // MARK: - 飞牛 NAS（docs/feiniu-api.md）

    /// 登录飞牛 NAS（FFI music_core_feiniu_login 返回 {token,base_url}）。
    static func feiniuLogin(baseUrl: String, username: String, password: String) async throws -> FeiniuLoginResult {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.feiniuLogin(baseUrl: baseUrl, username: username, password: password)
            return try Self.decode(FeiniuLoginResult.self, result)
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，飞牛登录不可用")
        #endif
    }

    /// 列出飞牛 NAS 指定路径文件（FFI music_core_feiniu_list_files 返回 {path,files}，提取 files）。
    static func feiniuListFiles(path: String) async throws -> [NasFile] {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.feiniuListFiles(path)
            return try Self.decode(FeiniuListFilesResult.self, result).files
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，飞牛列目录不可用")
        #endif
    }

    /// 生成飞牛文件流式播放 URL（FFI music_core_feiniu_stream 返回 {url}，提取 url）。
    static func feiniuStream(path: String) async throws -> String {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.feiniuStream(path)
            return try Self.decode(StreamUrlResult.self, result).url
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，飞牛流式播放不可用")
        #endif
    }

    /// 飞牛服务健康检查（FFI music_core_feiniu_health 返回 {healthy,base_url}）。
    static func feiniuHealth() async throws -> FeiniuHealthResult {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.feiniuHealth()
            return try Self.decode(FeiniuHealthResult.self, result)
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，飞牛健康检查不可用")
        #endif
    }

    // MARK: - 协议源（docs/protocol-api.md：SMB/WebDAV/FTP/DLNA/NFS）

    /// 添加协议源（FFI music_core_protocol_add 返回协议源对象）。
    static func protocolAdd(configJson: String) async throws -> ProtocolSource {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.protocolAdd(configJson)
            return try Self.decode(ProtocolSource.self, result)
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，协议源添加不可用")
        #endif
    }

    /// 列出已加载协议源（FFI music_core_protocol_list 返回 {sources:[...]}，提取 sources）。
    static func protocolList() async throws -> [ProtocolSource] {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.protocolList()
            return try Self.decode(ProtocolListResult.self, result).sources
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，协议源列表不可用")
        #endif
    }

    /// 删除协议源（FFI music_core_protocol_delete）。
    static func protocolDelete(id: String) async throws {
        #if NGH_LINK_MUSIC_CORE
        _ = try await run { try MusicCore.protocolDelete(id) }
        #else
        // 占位：no-op
        #endif
    }

    /// 列出协议源下指定路径条目（FFI music_core_protocol_list_files 返回 {path,entries}，提取 entries）。
    static func protocolListFiles(id: String, path: String) async throws -> [String] {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.protocolListFiles(id: id, path: path)
            return try Self.decode(ProtocolListFilesResult.self, result).entries
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，协议列目录不可用")
        #endif
    }

    /// 读取协议源文件字节（FFI music_core_protocol_read 返回 {size,data_base64}）。
    static func protocolRead(id: String, path: String) async throws -> ProtocolReadResult {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.protocolRead(id: id, path: path)
            return try Self.decode(ProtocolReadResult.self, result)
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，协议读取不可用")
        #endif
    }

    /// 生成协议源文件流式播放 URL（FFI music_core_protocol_stream 返回 {url}，提取 url）。
    static func protocolStream(id: String, path: String) async throws -> String {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.protocolStream(id: id, path: path)
            return try Self.decode(StreamUrlResult.self, result).url
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，协议流式播放不可用")
        #endif
    }

    // MARK: - 本地音乐（docs/sound-source-api.md 本地源）

    /// 初始化本地音乐源（FFI music_core_local_init）。
    static func localInit(dbPath: String) async throws {
        #if NGH_LINK_MUSIC_CORE
        _ = try await run { try MusicCore.localInit(dbPath) }
        #else
        // 占位：no-op
        #endif
    }

    /// 添加本地扫描目录并递归入库（FFI music_core_local_add_dir）。
    static func localAddDir(dir: String) async throws {
        #if NGH_LINK_MUSIC_CORE
        _ = try await run { try MusicCore.localAddDir(dir) }
        #else
        // 占位：no-op
        #endif
    }

    /// 重新扫描所有已添加目录（FFI music_core_local_rescan）。
    static func localRescan() async throws {
        #if NGH_LINK_MUSIC_CORE
        _ = try await run { try MusicCore.localRescan() }
        #else
        // 占位：no-op
        #endif
    }

    /// 返回本地扫描进度（FFI music_core_local_progress 返回 {current_count,scanning}）。
    static func localProgress() async throws -> LocalProgress {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.localProgress()
            return try Self.decode(LocalProgress.self, result)
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，扫描进度不可用")
        #endif
    }

    /// 列出本地音乐库歌曲。
    /// 注意：music-core FFI 当前未暴露 list_local_songs ABI（仅 init/add_dir/rescan/progress），
    /// 链接后仍抛 unavailable；脚手架默认（未链接）返回示例数据以便 UI 验证。
    static func listLocalSongs() async throws -> [Song] {
        #if NGH_LINK_MUSIC_CORE
        throw MusicCoreBridgeError.unavailable("核心 FFI 暂未提供本地歌曲列表接口")
        #else
        return Self.placeholderLocalSongs
        #endif
    }

    // MARK: - 缓存（docs/sound-source-api.md 缓存层）

    /// 初始化播放缓存（FFI music_core_cache_init）。
    static func cacheInit(cacheDir: String, maxBytes: UInt64) async throws {
        #if NGH_LINK_MUSIC_CORE
        _ = try await run { try MusicCore.cacheInit(cacheDir, maxBytes: maxBytes) }
        #else
        // 占位：no-op
        #endif
    }

    /// 返回缓存统计（FFI music_core_cache_stats 返回 {entries,total_bytes,max_bytes}）。
    static func cacheStats() async throws -> CacheStats {
        #if NGH_LINK_MUSIC_CORE
        return try await run {
            let result = try MusicCore.cacheStats()
            return try Self.decode(CacheStats.self, result)
        }
        #else
        throw MusicCoreBridgeError.unavailable("music-core 未链接，缓存统计不可用")
        #endif
    }

    /// 清空缓存（FFI music_core_cache_clear）。
    static func cacheClear() async throws {
        #if NGH_LINK_MUSIC_CORE
        _ = try await run { try MusicCore.cacheClear() }
        #else
        // 占位：no-op
        #endif
    }

    // MARK: - 脚手架占位数据（未链接核心库时供 UI 验证）

    private static func placeholderImported(_ json: String) -> SourceInfo {
        SourceInfo(id: "imported-\(Int(Date().timeIntervalSince1970))",
                   name: "已导入音源", version: "1.0.0", enabled: true,
                   sourceType: "json", priority: 999,
                   description: "脚手架占位：JSON 长度 \(json.count) 字符")
    }

    private static let placeholderSources: [SourceInfo] = [
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

    private static let placeholderLocalSongs: [Song] = [
        Song(id: "lo1", sourceId: "local", title: "本地示例一", artists: ["未知艺术家"],
             album: "本地专辑", coverUrl: nil, durationMs: 198000, lyricUrl: nil,
             playUrl: nil, localPath: "/music/a.mp3",
             origin: .local(path: "/music/a.mp3")),
        Song(id: "lo2", sourceId: "local", title: "本地示例二", artists: ["艺术家C"],
             album: nil, coverUrl: nil, durationMs: 165000, lyricUrl: nil,
             playUrl: nil, localPath: "/music/b.flac",
             origin: .local(path: "/music/b.flac")),
    ]
}
