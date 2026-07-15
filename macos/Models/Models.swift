// MARK: - Models
// 镜像 Rust core/src/models.rs 的 Swift 数据模型，用于 FFI JSON 解码。
// 所有结构均 Codable；CodingKeys 与 Rust 默认 snake_case 输出对齐。

import Foundation

/// 歌曲来源类型，使用内部标签 `type` 区分（与 Rust `#[serde(tag = "type")]` 对应）。
public enum SongOrigin: Codable, Hashable {
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

    private enum Tag: String, Codable {
        case online = "Online"
        case local = "Local"
        case nas = "Nas"
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let tag = try c.decode(Tag.self, forKey: .type)
        switch tag {
        case .online:
            let sourceId = try c.decode(String.self, forKey: .sourceId)
            let playUrl = try c.decode(String.self, forKey: .playUrl)
            self = .online(sourceId: sourceId, playUrl: playUrl)
        case .local:
            let path = try c.decode(String.self, forKey: .path)
            self = .local(path: path)
        case .nas:
            let proto = try c.decode(String.self, forKey: .protocolName)
            let url = try c.decode(String.self, forKey: .url)
            self = .nas(protocolName: proto, url: url)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case let .online(sourceId, playUrl):
            try c.encode(Tag.online, forKey: .type)
            try c.encode(sourceId, forKey: .sourceId)
            try c.encode(playUrl, forKey: .playUrl)
        case let .local(path):
            try c.encode(Tag.local, forKey: .type)
            try c.encode(path, forKey: .path)
        case let .nas(proto, url):
            try c.encode(Tag.nas, forKey: .type)
            try c.encode(proto, forKey: .protocolName)
            try c.encode(url, forKey: .url)
        }
    }
}

/// 单首歌曲
public struct Song: Codable, Identifiable, Hashable {
    public let id: String
    public let sourceId: String
    public let title: String
    public let artists: [String]
    public let album: String?
    public let coverUrl: String?
    public let durationMs: UInt64?
    public let lyricUrl: String?
    public let playUrl: String?
    public let localPath: String?
    public let origin: SongOrigin

    public init(
        id: String,
        sourceId: String,
        title: String,
        artists: [String],
        album: String? = nil,
        coverUrl: String? = nil,
        durationMs: UInt64? = nil,
        lyricUrl: String? = nil,
        playUrl: String? = nil,
        localPath: String? = nil,
        origin: SongOrigin
    ) {
        self.id = id
        self.sourceId = sourceId
        self.title = title
        self.artists = artists
        self.album = album
        self.coverUrl = coverUrl
        self.durationMs = durationMs
        self.lyricUrl = lyricUrl
        self.playUrl = playUrl
        self.localPath = localPath
        self.origin = origin
    }

    enum CodingKeys: String, CodingKey {
        case id
        case sourceId = "source_id"
        case title
        case artists
        case album
        case coverUrl = "cover_url"
        case durationMs = "duration_ms"
        case lyricUrl = "lyric_url"
        case playUrl = "play_url"
        case localPath = "local_path"
        case origin
    }

    /// 艺术家列表的可读展示（用 " / " 分隔），无艺术家时回退到 "未知艺术家"。
    public var artistsDisplay: String {
        artists.isEmpty ? "未知艺术家" : artists.joined(separator: " / ")
    }

    /// 时长文本（mm:ss），无时长返回 "--:--"。
    public var durationDisplay: String {
        guard let ms = durationMs, ms > 0 else { return "--:--" }
        let total = Int(ms / 1000)
        let m = total / 60
        let s = total % 60
        return String(format: "%02d:%02d", m, s)
    }
}

/// 专辑
public struct Album: Codable, Identifiable, Hashable {
    public let id: String
    public let sourceId: String
    public let name: String
    public let artists: [String]
    public let coverUrl: String?
    public let songIds: [String]

    public init(
        id: String,
        sourceId: String,
        name: String,
        artists: [String],
        coverUrl: String? = nil,
        songIds: [String] = []
    ) {
        self.id = id
        self.sourceId = sourceId
        self.name = name
        self.artists = artists
        self.coverUrl = coverUrl
        self.songIds = songIds
    }

    enum CodingKeys: String, CodingKey {
        case id
        case sourceId = "source_id"
        case name
        case artists
        case coverUrl = "cover_url"
        case songIds = "song_ids"
    }
}

/// 艺术家
public struct Artist: Codable, Identifiable, Hashable {
    public let id: String
    public let sourceId: String
    public let name: String
    public let avatarUrl: String?
    public let songIds: [String]

    public init(
        id: String,
        sourceId: String,
        name: String,
        avatarUrl: String? = nil,
        songIds: [String] = []
    ) {
        self.id = id
        self.sourceId = sourceId
        self.name = name
        self.avatarUrl = avatarUrl
        self.songIds = songIds
    }

    enum CodingKeys: String, CodingKey {
        case id
        case sourceId = "source_id"
        case name
        case avatarUrl = "avatar_url"
        case songIds = "song_ids"
    }
}

/// 单行歌词（LRC 时间轴）；timeMs 为 nil 表示无时间戳的纯文本行。
public struct LyricLine: Codable, Hashable {
    public let timeMs: UInt64?
    public let text: String

    public init(timeMs: UInt64? = nil, text: String) {
        self.timeMs = timeMs
        self.text = text
    }

    enum CodingKeys: String, CodingKey {
        case timeMs = "time_ms"
        case text
    }

    /// 行时间戳文本（mm:ss.xx），无时间戳返回空串。
    public var timestampDisplay: String {
        guard let ms = timeMs else { return "" }
        let total = Int(ms)
        let m = total / 60_000
        let s = (total % 60_000) / 1000
        let cs = (total % 1000) / 10
        return String(format: "%02d:%02d.%02d", m, s, cs)
    }
}

/// 歌词，可带翻译
public struct Lyric: Codable, Hashable {
    public let lines: [LyricLine]
    public let translation: [LyricLine]?

    public init(lines: [LyricLine], translation: [LyricLine]? = nil) {
        self.lines = lines
        self.translation = translation
    }
}

/// 搜索结果聚合
public struct SearchResult: Codable, Hashable {
    public let keyword: String
    public let songs: [Song]
    public let albums: [Album]
    public let artists: [Artist]
    public let total: UInt64
    public let page: UInt32
    public let pageSize: UInt32

    public init(
        keyword: String,
        songs: [Song],
        albums: [Album],
        artists: [Artist],
        total: UInt64,
        page: UInt32,
        pageSize: UInt32
    ) {
        self.keyword = keyword
        self.songs = songs
        self.albums = albums
        self.artists = artists
        self.total = total
        self.page = page
        self.pageSize = pageSize
    }

    enum CodingKeys: String, CodingKey {
        case keyword
        case songs
        case albums
        case artists
        case total
        case page
        case pageSize = "page_size"
    }
}

/// 排行榜
public struct Leaderboard: Codable, Identifiable, Hashable {
    public let id: String
    public let sourceId: String
    public let name: String
    public let coverUrl: String?
    public let songs: [Song]

    public init(
        id: String,
        sourceId: String,
        name: String,
        coverUrl: String? = nil,
        songs: [Song] = []
    ) {
        self.id = id
        self.sourceId = sourceId
        self.name = name
        self.coverUrl = coverUrl
        self.songs = songs
    }

    enum CodingKeys: String, CodingKey {
        case id
        case sourceId = "source_id"
        case name
        case coverUrl = "cover_url"
        case songs
    }
}

/// 播放模式（与 Rust `#[serde(rename_all = "snake_case")]` 一致）。
public enum PlayMode: String, Codable, CaseIterable, Hashable {
    case sequential = "sequential"
    case singleLoop = "single_loop"
    case random = "random"

    /// 用于 UI 显示的本地化名称。
    public var displayName: String {
        switch self {
        case .sequential: return "顺序播放"
        case .singleLoop: return "单曲循环"
        case .random: return "随机播放"
        }
    }

    /// SF Symbol 图标名。
    public var symbolName: String {
        switch self {
        case .sequential: return "list.number"
        case .singleLoop: return "repeat.1"
        case .random: return "shuffle"
        }
    }

    /// 下一个模式（用于点击切换）。
    public var next: PlayMode {
        let all = PlayMode.allCases
        let idx = all.firstIndex(of: self) ?? 0
        return all[(idx + 1) % all.count]
    }
}

/// 播放状态（用于持久化与跨界面同步）。
public struct PlayState: Codable, Hashable {
    public var currentSongId: String?
    public var playlistId: String?
    public var index: Int?
    public var positionMs: UInt64
    public var durationMs: UInt64
    public var isPlaying: Bool
    public var volume: Float
    public var mode: PlayMode

    public init(
        currentSongId: String? = nil,
        playlistId: String? = nil,
        index: Int? = nil,
        positionMs: UInt64 = 0,
        durationMs: UInt64 = 0,
        isPlaying: Bool = false,
        volume: Float = 1.0,
        mode: PlayMode = .sequential
    ) {
        self.currentSongId = currentSongId
        self.playlistId = playlistId
        self.index = index
        self.positionMs = positionMs
        self.durationMs = durationMs
        self.isPlaying = isPlaying
        self.volume = volume
        self.mode = mode
    }

    enum CodingKeys: String, CodingKey {
        case currentSongId = "current_song_id"
        case playlistId = "playlist_id"
        case index
        case positionMs = "position_ms"
        case durationMs = "duration_ms"
        case isPlaying = "is_playing"
        case volume
        case mode
    }
}

/// 音源信息（来源 /sources 列表项）。
public struct SourceInfo: Codable, Identifiable, Hashable {
    public let id: String
    public let name: String
    public let enabled: Bool
    public let priority: Int32

    public init(id: String, name: String, enabled: Bool, priority: Int32) {
        self.id = id
        self.name = name
        self.enabled = enabled
        self.priority = priority
    }
}

/// 音源列表响应（/sources）。
public struct SourceListResponse: Codable, Hashable {
    public let sources: [SourceInfo]
}

/// 校验结果（/sources/validate）。
public struct SourceValidateResult: Codable, Hashable {
    public let valid: Bool
    public let errors: [String]
}

/// 导入音源响应（/sources/import）。
public struct SourceImportResult: Codable, Hashable {
    public let sourceFormat: String
    public let warnings: [String]
    public let config: SourceImportConfig?

    enum CodingKeys: String, CodingKey {
        case sourceFormat = "source_format"
        case warnings
        case config
    }
}

/// 导入后的标准音源配置（保留为泛型 JSON 以避免与 core schema 强耦合）。
public struct SourceImportConfig: Codable, Hashable {
    public let manifest: [String: String]?
    public let endpoints: [String: [String: String]]?
}

/// 播放 URL 响应（/sources/{id}/songs/{songId}/play-url）。
public struct PlayUrlResponse: Codable, Hashable {
    public let url: String
    public let cached: Bool
    public let playUrl: String?

    enum CodingKeys: String, CodingKey {
        case url
        case cached
        case playUrl = "play_url"
    }
}

/// 飞牛 NAS 文件条目（兼容 isDir 命名）。
public struct NasFile: Codable, Identifiable, Hashable {
    public let name: String
    public let isDir: Bool
    public let size: UInt64
    public let modified: String?

    public var id: String { name }

    enum CodingKeys: String, CodingKey {
        case name
        case isDir = "is_dir"
        case size
        case modified
    }

    public init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        name = try c.decode(String.self, forKey: .name)
        // 兼容 isDir / is_dir
        if let v = try? c.decode(Bool.self, forKey: .isDir) {
            isDir = v
        } else {
            let nested = try decoder.container(keyedBy: AnyKey.self)
            isDir = (try? nested.decode(Bool.self, forKey: AnyKey(stringValue: "is_dir"))) ?? false
        }
        size = (try? c.decode(UInt64.self, forKey: .size)) ?? 0
        modified = try? c.decode(String.self, forKey: .modified)
    }

    public init(name: String, isDir: Bool, size: UInt64 = 0, modified: String? = nil) {
        self.name = name
        self.isDir = isDir
        self.size = size
        self.modified = modified
    }

    /// 动态 key 容器（用于兼容 is_dir 字段名）。
    private struct AnyKey: CodingKey {
        var stringValue: String
        init?(stringValue: String) { self.stringValue = stringValue }
        var intValue: Int? { nil }
        init?(intValue: Int) { return nil }
    }
}

/// 飞牛列表文件响应。
public struct NasFileListResponse: Codable, Hashable {
    public let path: String
    public let files: [NasFile]
}

/// 飞牛登录响应。
public struct FeiniuLoginResponse: Codable, Hashable {
    public let token: String
    public let baseUrl: String

    enum CodingKeys: String, CodingKey {
        case token
        case baseUrl = "base_url"
    }
}

/// 飞牛健康检查响应。
public struct FeiniuHealthResponse: Codable, Hashable {
    public let healthy: Bool
    public let baseUrl: String

    enum CodingKeys: String, CodingKey {
        case healthy
        case baseUrl = "base_url"
    }
}

/// 协议源条目（/protocols/sources）。
public struct ProtocolSource: Codable, Identifiable, Hashable {
    public let id: String
    public let protocolName: String
    public let root: String
    public let enabled: Bool
    public let placeholder: Bool?

    enum CodingKeys: String, CodingKey {
        case id
        case protocolName = "protocol"
        case root
        case enabled
        case placeholder
    }

    public init(id: String, protocolName: String, root: String, enabled: Bool, placeholder: Bool? = nil) {
        self.id = id
        self.protocolName = protocolName
        self.root = root
        self.enabled = enabled
        self.placeholder = placeholder
    }
}

/// 协议源列表响应。
public struct ProtocolListResponse: Codable, Hashable {
    public let sources: [ProtocolSource]
}

/// 协议源 list 响应。
public struct ProtocolListFilesResponse: Codable, Hashable {
    public let path: String
    public let entries: [String]
}

/// 协议源 stream 响应。
public struct ProtocolStreamResponse: Codable, Hashable {
    public let url: String
}

/// 协议源 stream 响应别名（feiniu stream 同结构）。
public typealias FeiniuStreamResponse = ProtocolStreamResponse

/// 本地扫描进度响应。
public struct LocalProgressResponse: Codable, Hashable {
    public let currentCount: UInt32
    public let scanning: Bool

    enum CodingKeys: String, CodingKey {
        case currentCount = "current_count"
        case scanning
    }
}

/// 通用 ok 响应（如 local_init / local_add_dir / local_rescan / cache_init / cache_clear）。
public struct OkResponse: Codable, Hashable {
    public let ok: Bool
}

/// 缓存统计响应。
public struct CacheStatsResponse: Codable, Hashable {
    public let entries: UInt64
    public let totalBytes: UInt64
    public let maxBytes: UInt64

    enum CodingKeys: String, CodingKey {
        case entries
        case totalBytes = "total_bytes"
        case maxBytes = "max_bytes"
    }
}

/// 核心版本响应（直接字符串，无需结构体；保留以备扩展）。
public typealias VersionResponse = String
