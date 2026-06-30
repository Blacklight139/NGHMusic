// MARK: - Swift 数据模型
// 职责：与 music-core Rust 模型（core/src/models.rs）对应的 Swift 结构体。
// 说明：实际启用 UniFFI 后，UniFFI 会自动生成等价类型；此文件提供脚手架阶段的占位模型，
//       便于 UI 在绑定生成前编译。字段命名与 Rust serde 默认输出（snake_case）一致，
//       通过 CodingKeys 显式映射以保证 JSON 解码正确。

import Foundation

/// 歌曲来源（对应 Rust SongOrigin，使用 "type" 内部标签）
enum SongOrigin: Codable, Equatable {
    case online(sourceId: String, playUrl: String)
    case local(path: String)
    case nas(protocolName: String, url: String)

    private enum CodingKeys: String, CodingKey { case type }
    private enum Tag: String { case Online, Local, Nas }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        switch try c.decode(String.self, forKey: .type) {
        case Tag.Online.rawValue:
            let v = try decoder.singleValueContainer().decode(OnlinePayload.self)
            self = .online(sourceId: v.sourceId, playUrl: v.playUrl)
        case Tag.Local.rawValue:
            let v = try decoder.singleValueContainer().decode(LocalPayload.self)
            self = .local(path: v.path)
        case Tag.Nas.rawValue:
            let v = try decoder.singleValueContainer().decode(NasPayload.self)
            self = .nas(protocolName: v.protocolName, url: v.url)
        default:
            throw DecodingError.dataCorruptedError(forKey: .type, in: c, debugDescription: "未知 SongOrigin")
        }
    }
    func encode(to encoder: Encoder) throws {}

    private struct OnlinePayload: Codable { let sourceId: String; let playUrl: String }
    private struct LocalPayload: Codable { let path: String }
    private struct NasPayload: Codable { let protocolName: String; let url: String }
}

/// 单首歌曲（对应 Rust Song）
struct Song: Codable, Identifiable, Equatable {
    let id: String
    let sourceId: String
    let title: String
    let artists: [String]
    let album: String?
    let coverUrl: String?
    let durationMs: UInt64?
    let lyricUrl: String?
    let playUrl: String?
    let localPath: String?
    let origin: SongOrigin
}

/// 搜索结果聚合（对应 Rust SearchResult）
struct SearchResult: Codable, Equatable {
    let keyword: String
    let songs: [Song]
    let albums: [Album]
    let artists: [Artist]
    let total: UInt64
    let page: UInt32
    let pageSize: UInt32
}

/// 专辑（对应 Rust Album）
struct Album: Codable, Identifiable, Equatable {
    let id: String
    let sourceId: String
    let name: String
    let artists: [String]
    let coverUrl: String?
    let songIds: [String]
}

/// 艺术家（对应 Rust Artist）
struct Artist: Codable, Identifiable, Equatable {
    let id: String
    let sourceId: String
    let name: String
    let avatarUrl: String?
    let songIds: [String]
}

/// 播放模式（对应 Rust PlayMode，snake_case）
enum PlayMode: String, Codable, CaseIterable {
    case sequential, singleLoop = "single_loop", random
    var label: String {
        switch self {
        case .sequential: return "顺序"
        case .singleLoop: return "单曲循环"
        case .random: return "随机"
        }
    }
}

/// 排行榜（对应 Rust Leaderboard）
struct Leaderboard: Codable, Identifiable, Equatable {
    let id: String
    let sourceId: String
    let name: String
    let coverUrl: String?
    let songs: [Song]
}
