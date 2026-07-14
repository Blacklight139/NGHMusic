// MARK: - MusicCore
// 职责：手写 Swift 封装，直接调用 music-core cdylib 的 C ABI（core/src/ffi.rs）。
// 与 UniFFI 生成方案二选一；本文件不依赖 UniFFI，通过 @_silgen_name 引用链接符号。
//
// 约定：
// - 所有返回 C 字符串（*mut c_char）的 ABI 均经 CString::into_raw 分配，
//   封装层读取后立即调用 music_core_free_string 释放。
// - 失败返回 {"error":{"kind":...,"message":...}} JSON，封装层解析后抛出 MusicCoreError。
// - 成功返回核心序列化的 JSON 字符串，由调用方按需解码。

import Foundation

/// 音乐核心 FFI 错误。
public enum MusicCoreError: Error, CustomStringConvertible {
    /// 核心返回的错误（kind 与 Rust CoreError 变体名一致）。
    case core(kind: String, message: String)
    /// FFI 层错误（空指针/序列化失败等）。
    case ffi(String)

    public var description: String {
        switch self {
        case let .core(kind, message):
            return "[\(kind)] \(message)"
        case let .ffi(message):
            return "[Ffi] \(message)"
        }
    }
}

/// Rust 核心 C ABI 的 Swift 安全封装。
public enum MusicCore {

    // MARK: - C ABI 符号声明（与 core/src/ffi.rs 的 #[no_mangle] 一一对应）

    @_silgen_name("music_core_version")
    private static func _version() -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_free_string")
    private static func _free_string(_ ptr: UnsafeMutablePointer<CChar>?)

    @_silgen_name("music_core_source_import")
    private static func _source_import(_ json: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_source_validate")
    private static func _source_validate(_ json: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_source_list")
    private static func _source_list() -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_source_enable")
    private static func _source_enable(_ id: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_source_disable")
    private static func _source_disable(_ id: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_source_delete")
    private static func _source_delete(_ id: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_search")
    private static func _search(_ keyword: UnsafePointer<CChar>?, _ page: UInt32, _ pageSize: UInt32) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_get_metadata")
    private static func _get_metadata(_ sourceId: UnsafePointer<CChar>?, _ songId: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_get_play_url")
    private static func _get_play_url(_ sourceId: UnsafePointer<CChar>?, _ songId: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_get_lyric")
    private static func _get_lyric(_ sourceId: UnsafePointer<CChar>?, _ songId: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_get_leaderboards")
    private static func _get_leaderboards(_ sourceId: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_feiniu_login")
    private static func _feiniu_login(_ baseUrl: UnsafePointer<CChar>?, _ username: UnsafePointer<CChar>?, _ password: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_feiniu_list_files")
    private static func _feiniu_list_files(_ path: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_feiniu_stream")
    private static func _feiniu_stream(_ path: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_feiniu_health")
    private static func _feiniu_health() -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_protocol_add")
    private static func _protocol_add(_ configJson: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_protocol_list")
    private static func _protocol_list() -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_protocol_delete")
    private static func _protocol_delete(_ id: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_protocol_list_files")
    private static func _protocol_list_files(_ id: UnsafePointer<CChar>?, _ path: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_protocol_read")
    private static func _protocol_read(_ id: UnsafePointer<CChar>?, _ path: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_protocol_stream")
    private static func _protocol_stream(_ id: UnsafePointer<CChar>?, _ path: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_local_init")
    private static func _local_init(_ dbPath: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_local_add_dir")
    private static func _local_add_dir(_ dir: UnsafePointer<CChar>?) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_local_rescan")
    private static func _local_rescan() -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_local_progress")
    private static func _local_progress() -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_cache_init")
    private static func _cache_init(_ cacheDir: UnsafePointer<CChar>?, _ maxBytes: UInt64) -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_cache_stats")
    private static func _cache_stats() -> UnsafeMutablePointer<CChar>?

    @_silgen_name("music_core_cache_clear")
    private static func _cache_clear() -> UnsafeMutablePointer<CChar>?

    // MARK: - 私有辅助

    /// 读取核心返回的 C 字符串并立即释放其内存；空指针抛出 .ffi 错误。
    private static func readAndFree(_ ptr: UnsafeMutablePointer<CChar>?) throws -> String {
        guard let p = ptr else {
            throw MusicCoreError.ffi("核心返回空指针（序列化失败）")
        }
        let s = String(cString: p)
        _free_string(p)
        return s
    }

    /// 校验 JSON 是否为错误对象，是则抛出 MusicCoreError.core；否则原样返回。
    private static func checkError(_ json: String) throws {
        guard let data = json.data(using: .utf8),
              let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let err = obj["error"] as? [String: Any] else {
            return
        }
        let kind = err["kind"] as? String ?? "Ffi"
        let msg = err["message"] as? String ?? ""
        throw MusicCoreError.core(kind: kind, message: msg)
    }

    /// 读取 + 释放 + 错误检查的组合。
    @discardableResult
    private static func readAndCheck(_ ptr: UnsafeMutablePointer<CChar>?) throws -> String {
        let json = try readAndFree(ptr)
        try checkError(json)
        return json
    }

    // MARK: - 版本

    /// 返回核心库版本字符串。
    @discardableResult
    public static func version() throws -> String {
        try readAndCheck(_version())
    }

    // MARK: - 音源管理

    /// 导入音源 JSON，返回导入后的 SourceInfo JSON。
    @discardableResult
    public static func sourceImport(_ json: String) throws -> String {
        try json.withCString { try readAndCheck(_source_import($0)) }
    }

    /// 校验音源 JSON 是否符合标准 Schema，返回 {"valid":bool,"errors":[...]}。
    @discardableResult
    public static func sourceValidate(_ json: String) throws -> String {
        try json.withCString { try readAndCheck(_source_validate($0)) }
    }

    /// 列出所有音源（按 priority 降序），返回 SourceInfo 数组 JSON。
    @discardableResult
    public static func sourceList() throws -> String {
        try readAndCheck(_source_list())
    }

    /// 启用指定 id 的音源，返回 {"id":...,"enabled":true}。
    @discardableResult
    public static func sourceEnable(_ id: String) throws -> String {
        try id.withCString { try readAndCheck(_source_enable($0)) }
    }

    /// 禁用指定 id 的音源，返回 {"id":...,"enabled":false}。
    @discardableResult
    public static func sourceDisable(_ id: String) throws -> String {
        try id.withCString { try readAndCheck(_source_disable($0)) }
    }

    /// 删除指定 id 的音源，返回 {"id":...,"deleted":true}。
    @discardableResult
    public static func sourceDelete(_ id: String) throws -> String {
        try id.withCString { try readAndCheck(_source_delete($0)) }
    }

    // MARK: - 搜索与歌曲

    /// 聚合搜索，返回 SearchResult JSON。
    @discardableResult
    public static func search(_ keyword: String, page: UInt32, pageSize: UInt32) throws -> String {
        try keyword.withCString { try readAndCheck(_search($0, page, pageSize)) }
    }

    /// 获取指定音源下歌曲的完整元数据，返回 Song JSON。
    @discardableResult
    public static func getMetadata(sourceId: String, songId: String) throws -> String {
        try sourceId.withCString { s1 in
            try songId.withCString { s2 in
                try readAndCheck(_get_metadata(s1, s2))
            }
        }
    }

    /// 获取指定音源下歌曲的可播放 URL，返回 {"url":...,"cached":false}。
    @discardableResult
    public static func getPlayUrl(sourceId: String, songId: String) throws -> String {
        try sourceId.withCString { s1 in
            try songId.withCString { s2 in
                try readAndCheck(_get_play_url(s1, s2))
            }
        }
    }

    /// 获取指定音源下歌曲的歌词，返回 Lyric JSON。
    @discardableResult
    public static func getLyric(sourceId: String, songId: String) throws -> String {
        try sourceId.withCString { s1 in
            try songId.withCString { s2 in
                try readAndCheck(_get_lyric(s1, s2))
            }
        }
    }

    /// 获取指定音源的排行榜列表，返回 Leaderboard 数组 JSON。
    @discardableResult
    public static func getLeaderboards(sourceId: String) throws -> String {
        try sourceId.withCString { try readAndCheck(_get_leaderboards($0)) }
    }

    // MARK: - 飞牛 NAS

    /// 登录飞牛 NAS，返回 {"token":...,"base_url":...}。
    @discardableResult
    public static func feiniuLogin(baseUrl: String, username: String, password: String) throws -> String {
        try baseUrl.withCString { a in
            try username.withCString { b in
                try password.withCString { c in
                    try readAndCheck(_feiniu_login(a, b, c))
                }
            }
        }
    }

    /// 列出飞牛 NAS 指定路径下的文件，返回 {"path":...,"files":[...]}。
    @discardableResult
    public static func feiniuListFiles(_ path: String) throws -> String {
        try path.withCString { try readAndCheck(_feiniu_list_files($0)) }
    }

    /// 生成飞牛 NAS 文件的可流式播放 URL，返回 {"url":...}。
    @discardableResult
    public static func feiniuStream(_ path: String) throws -> String {
        try path.withCString { try readAndCheck(_feiniu_stream($0)) }
    }

    /// 飞牛服务健康检查，返回 {"healthy":bool,"base_url":...}。
    @discardableResult
    public static func feiniuHealth() throws -> String {
        try readAndCheck(_feiniu_health())
    }

    // MARK: - 协议源（SMB/WebDAV/FTP/DLNA/NFS）

    /// 添加一个远程协议源，返回 {"id":...,"protocol":...,"root":...,"enabled":true,"placeholder":bool}。
    @discardableResult
    public static func protocolAdd(_ configJson: String) throws -> String {
        try configJson.withCString { try readAndCheck(_protocol_add($0)) }
    }

    /// 列出已加载的协议源，返回 {"sources":[...]}。
    @discardableResult
    public static func protocolList() throws -> String {
        try readAndCheck(_protocol_list())
    }

    /// 删除指定 id 的协议源，返回 {"id":...,"deleted":bool}。
    @discardableResult
    public static func protocolDelete(_ id: String) throws -> String {
        try id.withCString { try readAndCheck(_protocol_delete($0)) }
    }

    /// 列出协议源下指定路径的条目名称，返回 {"path":...,"entries":[...]}。
    @discardableResult
    public static func protocolListFiles(id: String, path: String) throws -> String {
        try id.withCString { a in
            try path.withCString { b in
                try readAndCheck(_protocol_list_files(a, b))
            }
        }
    }

    /// 读取协议源下指定文件为字节，返回 {"size":N,"data_base64":"..."}。
    @discardableResult
    public static func protocolRead(id: String, path: String) throws -> String {
        try id.withCString { a in
            try path.withCString { b in
                try readAndCheck(_protocol_read(a, b))
            }
        }
    }

    /// 生成协议源下指定文件的可流式播放 URL，返回 {"url":...}。
    @discardableResult
    public static func protocolStream(id: String, path: String) throws -> String {
        try id.withCString { a in
            try path.withCString { b in
                try readAndCheck(_protocol_stream(a, b))
            }
        }
    }

    // MARK: - 本地音乐

    /// 初始化本地音乐源（打开/创建 SQLite 索引库），返回 {"ok":true}。
    @discardableResult
    public static func localInit(_ dbPath: String) throws -> String {
        try dbPath.withCString { try readAndCheck(_local_init($0)) }
    }

    /// 添加本地扫描目录并递归扫描入库，返回 {"ok":true}。
    @discardableResult
    public static func localAddDir(_ dir: String) throws -> String {
        try dir.withCString { try readAndCheck(_local_add_dir($0)) }
    }

    /// 重新扫描所有已添加目录（增量更新），返回 {"ok":true}。
    @discardableResult
    public static func localRescan() throws -> String {
        try readAndCheck(_local_rescan())
    }

    /// 返回本地扫描进度，返回 {"current_count":N,"scanning":bool}。
    @discardableResult
    public static func localProgress() throws -> String {
        try readAndCheck(_local_progress())
    }

    // MARK: - 缓存

    /// 初始化播放缓存管理器，返回 {"ok":true}。
    @discardableResult
    public static func cacheInit(_ cacheDir: String, maxBytes: UInt64) throws -> String {
        try cacheDir.withCString { try readAndCheck(_cache_init($0, maxBytes)) }
    }

    /// 返回缓存统计，返回 {"entries":N,"total_bytes":N,"max_bytes":N}。
    @discardableResult
    public static func cacheStats() throws -> String {
        try readAndCheck(_cache_stats())
    }

    /// 清空所有缓存文件与索引，返回 {"ok":true}。
    @discardableResult
    public static func cacheClear() throws -> String {
        try readAndCheck(_cache_clear())
    }
}
