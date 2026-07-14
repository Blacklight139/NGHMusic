// MARK: - CoreService
// 包装 MusicCore FFI，对外暴露 async API（基于 Swift Concurrency）。
// 所有 FFI 调用走 background 优先级 DispatchQueue（MusicCore 为同步阻塞 API），
// 结果 JSON 通过 JSONDecoder 解码为 Models 中的强类型。
// 注意：FFI 调用是同步且阻塞的，故统一通过 withCheckedThrowingContinuation 包到后台线程。

import Foundation

/// CoreService 单例，封装 music-core 全部能力。
/// 通过 actor 隔离保证并发安全；FFI 仍走同步符号，由 actor 串行化访问。
public actor CoreService {

    public static let shared = CoreService()

    /// 内部统一的 JSON 解码器，使用秒级解码策略（与 Rust 默认一致）。
    private let decoder: JSONDecoder = {
        let d = JSONDecoder()
        return d
    }()

    public init() {}

    // MARK: - 版本

    /// 返回核心版本字符串。
    public func version() async throws -> String {
        try await callString { try MusicCore.version() }
    }

    // MARK: - 音源管理

    /// 导入音源 JSON，返回适配后配置与迁移报告。
    public func sourceImport(_ json: String) async throws -> SourceImportResult {
        let raw = try await callString { try MusicCore.sourceImport(json) }
        return try decode(SourceImportResult.self, from: raw)
    }

    /// 校验音源 JSON 是否符合 Schema。
    public func sourceValidate(_ json: String) async throws -> SourceValidateResult {
        let raw = try await callString { try MusicCore.sourceValidate(json) }
        return try decode(SourceValidateResult.self, from: raw)
    }

    /// 列出所有音源。
    public func sourceList() async throws -> [SourceInfo] {
        let raw = try await callString { try MusicCore.sourceList() }
        // 核心返回 {"sources":[...]} 或直接 [...]
        if let wrap = try? decode(SourceListResponse.self, from: raw) {
            return wrap.sources
        }
        return try decode([SourceInfo].self, from: raw)
    }

    /// 启用指定音源。
    public func sourceEnable(_ id: String) async throws {
        _ = try await callString { try MusicCore.sourceEnable(id) }
    }

    /// 禁用指定音源。
    public func sourceDisable(_ id: String) async throws {
        _ = try await callString { try MusicCore.sourceDisable(id) }
    }

    /// 删除指定音源。
    public func sourceDelete(_ id: String) async throws {
        _ = try await callString { try MusicCore.sourceDelete(id) }
    }

    // MARK: - 搜索与歌曲

    /// 聚合搜索。
    public func search(keyword: String, page: UInt32 = 1, pageSize: UInt32 = 20) async throws -> SearchResult {
        let raw = try await callString { try MusicCore.search(keyword, page: page, pageSize: pageSize) }
        return try decode(SearchResult.self, from: raw)
    }

    /// 获取歌曲元数据。
    public func getMetadata(sourceId: String, songId: String) async throws -> Song {
        let raw = try await callString { try MusicCore.getMetadata(sourceId: sourceId, songId: songId) }
        return try decode(Song.self, from: raw)
    }

    /// 获取播放 URL。
    public func getPlayUrl(sourceId: String, songId: String) async throws -> PlayUrlResponse {
        let raw = try await callString { try MusicCore.getPlayUrl(sourceId: sourceId, songId: songId) }
        return try decode(PlayUrlResponse.self, from: raw)
    }

    /// 获取歌词。
    public func getLyric(sourceId: String, songId: String) async throws -> Lyric {
        let raw = try await callString { try MusicCore.getLyric(sourceId: sourceId, songId: songId) }
        return try decode(Lyric.self, from: raw)
    }

    /// 获取音源排行榜。
    public func getLeaderboards(sourceId: String) async throws -> [Leaderboard] {
        let raw = try await callString { try MusicCore.getLeaderboards(sourceId: sourceId) }
        return try decode([Leaderboard].self, from: raw)
    }

    // MARK: - 飞牛 NAS

    /// 登录飞牛 NAS。
    public func feiniuLogin(baseUrl: String, username: String, password: String) async throws -> FeiniuLoginResponse {
        let raw = try await callString {
            try MusicCore.feiniuLogin(baseUrl: baseUrl, username: username, password: password)
        }
        return try decode(FeiniuLoginResponse.self, from: raw)
    }

    /// 列出飞牛 NAS 指定路径下的文件。
    public func feiniuListFiles(_ path: String) async throws -> NasFileListResponse {
        let raw = try await callString { try MusicCore.feiniuListFiles(path) }
        // 兼容 {path, files} 与裸数组
        if let wrap = try? decode(NasFileListResponse.self, from: raw) {
            return wrap
        }
        let arr = try decode([NasFile].self, from: raw)
        return NasFileListResponse(path: path, files: arr)
    }

    /// 生成飞牛 NAS 文件的可流式播放 URL。
    public func feiniuStream(_ path: String) async throws -> FeiniuStreamResponse {
        let raw = try await callString { try MusicCore.feiniuStream(path) }
        return try decode(FeiniuStreamResponse.self, from: raw)
    }

    /// 飞牛服务健康检查。
    public func feiniuHealth() async throws -> FeiniuHealthResponse {
        let raw = try await callString { try MusicCore.feiniuHealth() }
        return try decode(FeiniuHealthResponse.self, from: raw)
    }

    // MARK: - 协议源（SMB/WebDAV/FTP/DLNA/NFS）

    /// 添加一个远程协议源。
    public func protocolAdd(_ configJson: String) async throws -> ProtocolSource {
        let raw = try await callString { try MusicCore.protocolAdd(configJson) }
        return try decode(ProtocolSource.self, from: raw)
    }

    /// 列出已加载的协议源。
    public func protocolList() async throws -> [ProtocolSource] {
        let raw = try await callString { try MusicCore.protocolList() }
        if let wrap = try? decode(ProtocolListResponse.self, from: raw) {
            return wrap.sources
        }
        return try decode([ProtocolSource].self, from: raw)
    }

    /// 删除指定 id 的协议源。
    public func protocolDelete(_ id: String) async throws {
        _ = try await callString { try MusicCore.protocolDelete(id) }
    }

    /// 列出协议源下指定路径的条目名称。
    public func protocolListFiles(id: String, path: String) async throws -> ProtocolListFilesResponse {
        let raw = try await callString { try MusicCore.protocolListFiles(id: id, path: path) }
        return try decode(ProtocolListFilesResponse.self, from: raw)
    }

    /// 读取协议源下指定文件为字节（返回 base64 数据）。
    /// 注意：响应可能很大，仅对小文件使用。
    public func protocolRead(id: String, path: String) async throws -> [String: Any] {
        let raw = try await callString { try MusicCore.protocolRead(id: id, path: path) }
        guard let data = raw.data(using: .utf8),
              let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            throw MusicCoreError.ffi("protocolRead 响应非 JSON 对象")
        }
        return obj
    }

    /// 生成协议源下指定文件的可流式播放 URL。
    public func protocolStream(id: String, path: String) async throws -> ProtocolStreamResponse {
        let raw = try await callString { try MusicCore.protocolStream(id: id, path: path) }
        return try decode(ProtocolStreamResponse.self, from: raw)
    }

    // MARK: - 本地音乐

    /// 初始化本地音乐源（打开/创建 SQLite 索引库）。
    public func localInit(_ dbPath: String) async throws {
        _ = try await callString { try MusicCore.localInit(dbPath) }
    }

    /// 添加本地扫描目录并递归扫描入库。
    public func localAddDir(_ dir: String) async throws {
        _ = try await callString { try MusicCore.localAddDir(dir) }
    }

    /// 重新扫描所有已添加目录（增量更新）。
    public func localRescan() async throws {
        _ = try await callString { try MusicCore.localRescan() }
    }

    /// 返回本地扫描进度。
    public func localProgress() async throws -> LocalProgressResponse {
        let raw = try await callString { try MusicCore.localProgress() }
        return try decode(LocalProgressResponse.self, from: raw)
    }

    // MARK: - 缓存

    /// 初始化播放缓存管理器。
    public func cacheInit(_ cacheDir: String, maxBytes: UInt64) async throws {
        _ = try await callString { try MusicCore.cacheInit(cacheDir, maxBytes: maxBytes) }
    }

    /// 返回缓存统计。
    public func cacheStats() async throws -> CacheStatsResponse {
        let raw = try await callString { try MusicCore.cacheStats() }
        return try decode(CacheStatsResponse.self, from: raw)
    }

    /// 清空所有缓存文件与索引。
    public func cacheClear() async throws {
        _ = try await callString { try MusicCore.cacheClear() }
    }

    // MARK: - 内部辅助

    /// 将同步阻塞的 FFI 调用包装为 async，在后台线程执行。
    private func callString(_ work: @escaping () throws -> String) async throws -> String {
        try await withCheckedThrowingContinuation { (cont: CheckedContinuation<String, Error>) in
            DispatchQueue.global(qos: .userInitiated).async {
                do {
                    let r = try work()
                    cont.resume(returning: r)
                } catch {
                    cont.resume(throwing: error)
                }
            }
        }
    }

    /// 统一 JSON 解码，遇到错误对象优先抛出 MusicCoreError.core。
    private func decode<T: Decodable>(_ type: T.Type, from json: String) throws -> T {
        guard let data = json.data(using: .utf8) else {
            throw MusicCoreError.ffi("无法将字符串转为 UTF-8 数据")
        }
        do {
            return try decoder.decode(T.self, from: data)
        } catch {
            // 检查是否为错误对象
            if let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
               let err = obj["error"] as? [String: Any] {
                let kind = err["kind"] as? String ?? "Ffi"
                let msg = err["message"] as? String ?? ""
                throw MusicCoreError.core(kind: kind, message: msg)
            }
            throw MusicCoreError.ffi("JSON 解码失败 (\(T.self)): \(error)")
        }
    }
}
