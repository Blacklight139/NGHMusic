import Foundation

// MARK: - C ABI 声明
// 通过 @_silgen_name 直接声明 Rust 静态库导出的 C ABI 函数（core/src/ffi/mod.rs）。
// 也可改用 module.modulemap + `import MusicCore`（见 module.modulemap 说明）。
// 二者仅启用其一，避免符号重复定义；本工程默认使用 @_silgen_name。

/// FFI 通用结果，与 Rust `#[repr(C)] FfiResult` 内存布局对齐：
/// `{ i32 code; *mut c_char data; }`（aarch64 下 16 字节，code 后有 4 字节对齐填充）。
/// 注意：`data` 为 Rust 端堆分配的 C 字符串，调用方必须用 `music_core_free_string` 释放。
struct FfiResult {
    var code: Int32
    var data: UnsafeMutablePointer<CChar>?
}

@_silgen_name("music_core_import_source")
private func music_core_import_source(_ json: UnsafePointer<CChar>?) -> FfiResult

@_silgen_name("music_core_list_sources")
private func music_core_list_sources() -> FfiResult

@_silgen_name("music_core_set_enabled")
private func music_core_set_enabled(_ id: UnsafePointer<CChar>?, _ enabled: Int32) -> FfiResult

@_silgen_name("music_core_search")
private func music_core_search(
    _ keyword: UnsafePointer<CChar>?,
    _ typeJson: UnsafePointer<CChar>?,
    _ pageJson: UnsafePointer<CChar>?
) -> FfiResult

@_silgen_name("music_core_free_string")
private func music_core_free_string(_ ptr: UnsafeMutablePointer<CChar>?)

// MARK: - 错误

/// FFI 层错误。
enum FFIError: LocalizedError {
    /// Rust 核心返回的非零错误码（含可读 message）。
    case core(code: FfiCode, message: String)
    /// 指针为空。
    case nullPointer
    /// UTF-8 解码失败。
    case utf8
    /// JSON 解码失败。
    case decode(String)
    /// 期望数据但返回空（不应发生）。
    case unexpectedEmpty

    var errorDescription: String? {
        switch self {
        case let .core(code, message):
            return "[\(code)] \(message)"
        case .nullPointer: return "传入指针为空"
        case .utf8: return "UTF-8 解码失败"
        case .decode(let m): return "数据解码失败：\(m)"
        case .unexpectedEmpty: return "期望返回数据但为空"
        }
    }
}

// MARK: - FFIBridge

/// Rust 核心 FFI 桥接层。
///
/// 将 C ABI（同步、阻塞）封装为 Swift async API：
/// - 所有 `music_core_*` 调用均调度到后台线程，避免阻塞主线程与 UI。
/// - 返回的 JSON 字符串解码为 Swift `Codable` 模型。
/// - 严格处理错误码，并通过 `music_core_free_string` 释放 Rust 端分配的字符串内存。
final class FFIBridge {
    static let shared = FFIBridge()
    private init() {}

    // 后台执行队列：搜索等调用内部会 `block_on` tokio 运行时，需离开主线程。
    private let queue = DispatchQueue(label: "music.ffi.bridge", qos: .userInitiated)

    /// 将同步阻塞工作调度到后台线程并以 async 形式返回。
    private func runBlocking<T>(_ work: @escaping () throws -> T) async throws -> T {
        try await withCheckedThrowingContinuation { cont in
            queue.async {
                do {
                    cont.resume(returning: try work())
                } catch {
                    cont.resume(throwing: error)
                }
            }
        }
    }

    // MARK: 内部解码工具

    /// 将 Encodable 编码为 JSON 字符串（用于传给 C ABI）。
    private func jsonString<T: Encodable>(_ value: T) throws -> String {
        let data = try JSONEncoder().encode(value)
        guard let s = String(data: data, encoding: .utf8) else { throw FFIError.utf8 }
        return s
    }

    /// 读取并校验 FfiResult，返回原始 JSON 字符串；ok_empty（data 为空）返回 nil。
    /// 无论成功与否，均会释放 Rust 端分配的 `data` 内存。
    private func extractString(_ result: FfiResult) throws -> String? {
        let code = FfiCode(rawValue: result.code) ?? .err
        if let ptr = result.data {
            // 必须释放，即使失败分支也不能泄漏
            defer { music_core_free_string(ptr) }
            let payload = String(cString: ptr)
            guard code == .ok else {
                // 失败时 data 形如 {"error":"..."}
                if let d = payload.data(using: .utf8),
                   let err = try? JSONDecoder().decode(FFIErrorResponse.self, from: d) {
                    throw FFIError.core(code: code, message: err.error)
                }
                throw FFIError.core(code: code, message: payload)
            }
            return payload
        } else {
            // data 为空：仅当 code == ok 时合法（ok_empty），否则视为错误
            guard code == .ok else {
                throw FFIError.core(code: code, message: "未知错误")
            }
            return nil
        }
    }

    /// 解码为强类型模型。
    private func decode<T: Decodable>(_ result: FfiResult, as type: T.Type) throws -> T {
        let payload = try extractString(result)
        guard let payload, let data = payload.data(using: .utf8) else {
            throw FFIError.unexpectedEmpty
        }
        do {
            return try JSONDecoder().decode(T.self, from: data)
        } catch {
            throw FFIError.decode(error.localizedDescription)
        }
    }

    /// 仅校验错误码（用于无返回值的操作，如 set_enabled）。
    private func decodeVoid(_ result: FfiResult) throws {
        _ = try extractString(result)
    }

    // MARK: - 对外 async API

    /// 导入音源 JSON（标准或社区格式），返回音源 ID。
    /// 对应 `music_core_import_source(json_ptr) -> FfiResult`（成功时 data 为 `"id"` JSON 字符串）。
    func importSource(json: String) async throws -> String {
        try await runBlocking {
            let result = json.withCString { ptr in
                music_core_import_source(ptr)
            }
            // 导入成功返回的是被引号包裹的字符串 JSON，如 "com.example.netease"
            return try self.decode(result, as: String.self)
        }
    }

    /// 列出所有已注册音源（按优先级降序）。
    /// 对应 `music_core_list_sources() -> FfiResult`（成功时 data 为 JSON 数组）。
    func listSources() async throws -> [SourceInfo] {
        try await runBlocking {
            let result = music_core_list_sources()
            return try self.decode(result, as: [SourceInfo].self)
        }
    }

    /// 设置音源启用/禁用。
    /// 对应 `music_core_set_enabled(id_ptr, enabled) -> FfiResult`（成功时为 ok_empty）。
    func setEnabled(sourceId: String, enabled: Bool) async throws {
        try await runBlocking {
            let result = sourceId.withCString { ptr in
                music_core_set_enabled(ptr, enabled ? 1 : 0)
            }
            try self.decodeVoid(result)
        }
    }

    /// 跨音源聚合搜索。
    /// 对应 `music_core_search(keyword, type_json, page_json) -> FfiResult`。
    /// - Parameters:
    ///   - keyword: 搜索关键词
    ///   - type: 搜索分类（song/album/artist）
    ///   - page: 分页（offset/limit）
    /// - Returns: 分页结果 `Paged<SearchResult>`
    func search(keyword: String, type: SearchType, page: Page) async throws -> Paged<SearchResult> {
        try await runBlocking {
            let typeJson = try self.jsonString(type)        // 序列化为 "song" 等
            let pageJson = try self.jsonString(page)        // 序列化为 {"offset":0,"limit":20}

            // 三个 C 字符串需在调用期间保持存活，使用嵌套 withCString 保证生命周期
            let result = keyword.withCString { kwPtr in
                typeJson.withCString { tyPtr in
                    pageJson.withCString { pgPtr in
                        music_core_search(kwPtr, tyPtr, pgPtr)
                    }
                }
            }
            return try self.decode(result, as: Paged<SearchResult>.self)
        }
    }
}

// MARK: - 便捷扩展

extension FFIBridge {
    /// 搜索首页第一页的便捷方法。
    func search(keyword: String, type: SearchType) async throws -> Paged<SearchResult> {
        try await search(keyword: keyword, type: type, page: Page())
    }
}
