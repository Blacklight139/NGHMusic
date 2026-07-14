// MARK: - Swift 数据模型
// 职责：与 music-core Rust 模型（core/src/models.rs）对应的 Swift 结构体。
//
// 说明：所有类型均为 Codable，并通过 CodingKeys 显式映射到 Rust serde 默认
//       输出的 snake_case JSON，保证 JSON 解码正确（不依赖 keyDecodingStrategy）。
//       实际启用 UniFFI 后，UniFFI 会自动生成等价类型；此文件提供脚手架阶段的占位模型，
//       便于 UI 在绑定生成前编译。字段命名与 core/src/models.rs 完全对齐。

import Foundation

// MARK: - SongOrigin
/// 歌曲来源（对应 Rust SongOrigin，使用 "type" 内部标签，serde `#[serde(tag = "type")]`）。
/// 关键：Nas 变体的协议字段在 JSON 中为 `protocol`（无 `_name` 后缀），
/// 与 core/src/models.rs::SongOrigin::Nas { protocol } 一致（修复 HAR-003 类问题）。
enum SongOrigin: Codable, Equatable {
    case online(sourceId: String, playUrl: String)
    case local(path: String)
    case nas(protocolName: String, url: String)

    private enum CodingKeys: String, CodingKey {
        case type
        case sourceId = "source_id"
        case playUrl = "play_url"
        case path
        case protocolName = "protocol"
        case url
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        switch try c.decode(String.self, forKey: .type) {
        case "Online":
            self = .online(sourceId: try c.decode(String.self, forKey: .sourceId),
                           playUrl: try c.decode(String.self, forKey: .playUrl))
        case "Local":
            self = .local(path: try c.decode(String.self, forKey: .path))
        case "Nas":
            self = .nas(protocolName: try c.decode(String.self, forKey: .protocolName),
                        url: try c.decode(String.self, forKey: .url))
        default:
            throw DecodingError.dataCorruptedError(forKey: .type, in: c,
                                                   debugDescription: "未知 SongOrigin 类型")
        }
    }

    func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case let .online(sourceId, playUrl):
            try c.encode("Online", forKey: .type)
            try c.encode(sourceId, forKey: .sourceId)
            try c.encode(playUrl, forKey: .playUrl)
        case let .local(path):
            try c.encode("Local", forKey: .type)
            try c.encode(path, forKey: .path)
        case let .nas(protocolName, url):
            try c.encode("Nas", forKey: .type)
            try c.encode(protocolName, forKey: .protocolName)
            try c.encode(url, forKey: .url)
        }
    }
}

// MARK: - Song
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

    private enum CodingKeys: String, CodingKey {
        case id
        case sourceId = "source_id"
        case title, artists, album
        case coverUrl = "cover_url"
        case durationMs = "duration_ms"
        case lyricUrl = "lyric_url"
        case playUrl = "play_url"
        case localPath = "local_path"
        case origin
    }
}

// MARK: - Album
/// 专辑（对应 Rust Album）
struct Album: Codable, Identifiable, Equatable {
    let id: String
    let sourceId: String
    let name: String
    let artists: [String]
    let coverUrl: String?
    let songIds: [String]

    private enum CodingKeys: String, CodingKey {
        case id
        case sourceId = "source_id"
        case name, artists
        case coverUrl = "cover_url"
        case songIds = "song_ids"
    }
}

// MARK: - Artist
/// 艺术家（对应 Rust Artist）
struct Artist: Codable, Identifiable, Equatable {
    let id: String
    let sourceId: String
    let name: String
    let avatarUrl: String?
    let songIds: [String]

    private enum CodingKeys: String, CodingKey {
        case id
        case sourceId = "source_id"
        case name
        case avatarUrl = "avatar_url"
        case songIds = "song_ids"
    }
}

// MARK: - LyricLine / Lyric
/// 单行歌词（对应 Rust LyricLine）；timeMs 为 nil 表示无时间戳的纯文本行。
struct LyricLine: Codable, Equatable, Identifiable {
    let timeMs: UInt64?
    let text: String

    /// Identifiable：用文本自身作 id（同一歌词内文本通常唯一，足够列表渲染）。
    var id: String { text }

    private enum CodingKeys: String, CodingKey {
        case timeMs = "time_ms"
        case text
    }
}

/// 歌词，可带翻译（对应 Rust Lyric）
struct Lyric: Codable, Equatable {
    let lines: [LyricLine]
    let translation: [LyricLine]?
}

// MARK: - SearchResult
/// 搜索结果聚合（对应 Rust SearchResult）
struct SearchResult: Codable, Equatable {
    let keyword: String
    let songs: [Song]
    let albums: [Album]
    let artists: [Artist]
    let total: UInt64
    let page: UInt32
    let pageSize: UInt32

    private enum CodingKeys: String, CodingKey {
        case keyword, songs, albums, artists, total, page
        case pageSize = "page_size"
    }
}

// MARK: - Leaderboard
/// 排行榜（对应 Rust Leaderboard）
struct Leaderboard: Codable, Identifiable, Equatable {
    let id: String
    let sourceId: String
    let name: String
    let coverUrl: String?
    let songs: [Song]

    private enum CodingKeys: String, CodingKey {
        case id
        case sourceId = "source_id"
        case name
        case coverUrl = "cover_url"
        case songs
    }
}

// MARK: - PlayMode
/// 播放模式（对应 Rust PlayMode，snake_case 序列化：sequential / single_loop / random）。
/// CaseIterable：供 PlayerManager.toggleMode() 遍历（修复 IOS-002）。
enum PlayMode: String, Codable, CaseIterable {
    case sequential
    case singleLoop = "single_loop"
    case random

    var label: String {
        switch self {
        case .sequential: return "顺序"
        case .singleLoop: return "单曲循环"
        case .random: return "随机"
        }
    }
}
