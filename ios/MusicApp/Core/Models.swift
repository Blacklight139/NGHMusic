import Foundation

// MARK: - FFI 辅助类型

/// FFI 返回码，与 Rust `FfiCode` 对齐（core/src/ffi/mod.rs）。
enum FfiCode: Int32 {
    case ok = 0
    case err = 1
    case nullPtr = 2
    case utf8 = 3
    case panic = 4
}

/// FFI 错误响应。失败时 `FfiResult.data` 为 `{"error":"..."}`。
struct FFIErrorResponse: Codable {
    let error: String
}

/// 音源信息，对应 `music_core_list_sources` 返回数组中的每一项。
/// Rust 侧序列化：`{ "id": ..., "name": ..., "enabled": ..., "priority": ... }`。
struct SourceInfo: Codable, Identifiable, Hashable {
    let id: String
    let name: String
    let enabled: Bool
    let priority: Int
}

// MARK: - 核心数据模型
// 与 core/src/models.rs 逐字段对齐；JSON 字段名采用 snake_case。
// 通过显式 CodingKeys 与 Rust 端 serde 命名保持一致，确保跨端编解码兼容。

/// 全局唯一歌曲标识：`{source_id}:{song_id}`。
struct SongRef: Codable, Hashable, Identifiable {
    var sourceId: String
    var songId: String

    enum CodingKeys: String, CodingKey {
        case sourceId = "source_id"
        case songId = "song_id"
    }

    init(sourceId: String = "", songId: String = "") {
        self.sourceId = sourceId
        self.songId = songId
    }

    /// 复合键，便于缓存键与持久化。
    var id: String { "\(sourceId):\(songId)" }
    var key: String { id }
}

/// 歌曲标准元数据。
struct Song: Codable, Hashable, Identifiable {
    var sourceId: String
    var songId: String
    var title: String
    var artist: String
    var album: String
    var coverUrl: String?
    /// 秒
    var duration: Double?
    /// 歌词地址（URL 或数据 URI）
    var lyricUrl: String?
    /// 播放数据 URL（流地址 / 文件路径）
    var playUrl: String?
    /// 是否已缓存
    var cached: Bool

    enum CodingKeys: String, CodingKey {
        case sourceId = "source_id"
        case songId = "song_id"
        case title, artist, album
        case coverUrl = "cover_url"
        case duration
        case lyricUrl = "lyric_url"
        case playUrl = "play_url"
        case cached
    }

    init(
        sourceId: String = "",
        songId: String = "",
        title: String = "",
        artist: String = "",
        album: String = "",
        coverUrl: String? = nil,
        duration: Double? = nil,
        lyricUrl: String? = nil,
        playUrl: String? = nil,
        cached: Bool = false
    ) {
        self.sourceId = sourceId
        self.songId = songId
        self.title = title
        self.artist = artist
        self.album = album
        self.coverUrl = coverUrl
        self.duration = duration
        self.lyricUrl = lyricUrl
        self.playUrl = playUrl
        self.cached = cached
    }

    /// 对应的 SongRef。
    var songRef: SongRef { SongRef(sourceId: sourceId, songId: songId) }
    /// Identifiable：复合键。
    var id: String { songRef.id }

    /// 友好时长（mm:ss）。
    var formattedDuration: String {
        guard let d = duration, d > 0 else { return "--:--" }
        let total = Int(d.rounded())
        return String(format: "%d:%02d", total / 60, total % 60)
    }
}

/// 专辑。
struct Album: Codable, Hashable, Identifiable {
    var sourceId: String
    var albumId: String
    var name: String
    var artist: String
    var coverUrl: String?
    var publishDate: String?
    var songIds: [String]

    enum CodingKeys: String, CodingKey {
        case sourceId = "source_id"
        case albumId = "album_id"
        case name, artist
        case coverUrl = "cover_url"
        case publishDate = "publish_date"
        case songIds = "song_ids"
    }

    init(
        sourceId: String = "",
        albumId: String = "",
        name: String = "",
        artist: String = "",
        coverUrl: String? = nil,
        publishDate: String? = nil,
        songIds: [String] = []
    ) {
        self.sourceId = sourceId
        self.albumId = albumId
        self.name = name
        self.artist = artist
        self.coverUrl = coverUrl
        self.publishDate = publishDate
        self.songIds = songIds
    }

    var id: String { "\(sourceId):\(albumId)" }
}

/// 艺术家。
struct Artist: Codable, Hashable, Identifiable {
    var sourceId: String
    var artistId: String
    var name: String
    var avatarUrl: String?
    var songIds: [String]

    enum CodingKeys: String, CodingKey {
        case sourceId = "source_id"
        case artistId = "artist_id"
        case name
        case avatarUrl = "avatar_url"
        case songIds = "song_ids"
    }

    init(
        sourceId: String = "",
        artistId: String = "",
        name: String = "",
        avatarUrl: String? = nil,
        songIds: [String] = []
    ) {
        self.sourceId = sourceId
        self.artistId = artistId
        self.name = name
        self.avatarUrl = avatarUrl
        self.songIds = songIds
    }

    var id: String { "\(sourceId):\(artistId)" }
}

// MARK: - 歌词

/// LRC 解析后的歌词行（秒）。
struct LyricLine: Codable, Hashable, Identifiable {
    /// 秒
    var time: Double
    var text: String
    /// 可选翻译
    var translation: String?

    /// 用于 ForEach 稳定标识。
    var id: String { String(format: "%.3f", time) + "|" + text }
}

/// 歌词集合。
struct Lyrics: Codable, Hashable {
    var songRef: SongRef
    var lines: [LyricLine]

    enum CodingKeys: String, CodingKey {
        case songRef = "song_ref"
        case lines
    }

    init(songRef: SongRef = SongRef(), lines: [LyricLine] = []) {
        self.songRef = songRef
        self.lines = lines
    }
}

/// LRC 解析与时间轴定位，移植自 core/src/lyrics/mod.rs（parse_lrc / locate）。
enum LyricsParser {
    /// 解析 LRC 文本为带时间轴的歌词行。
    static func parse(_ text: String) -> [LyricLine] {
        var out: [LyricLine] = []
        // 匹配 [mm:ss.xx] / [mm:ss:xx] / [mm:ss]
        let pattern = #"\[(\d+):(\d+)(?:[.:](\d+))?\]"#
        guard let regex = try? NSRegularExpression(pattern: pattern) else { return [] }

        for raw in text.split(separator: "\n", omittingEmptySubsequences: false) {
            let line = raw.trimmingCharacters(in: .whitespaces)
            if line.isEmpty { continue }
            let ns = line as NSString
            let matches = regex.matches(in: line, range: NSRange(location: 0, length: ns.length))

            var times: [Double] = []
            for m in matches {
                let mm = Double(ns.substring(with: m.range(at: 1))) ?? 0
                let ss = Double(ns.substring(with: m.range(at: 2))) ?? 0
                var ms: Double = 0
                if m.range(at: 3).location != NSNotFound {
                    let frag = ns.substring(with: m.range(at: 3))
                    if frag.count <= 2 {
                        ms = (Double(frag) ?? 0) * 10          // 百分位 -> 毫秒
                    } else {
                        ms = Double(frag.prefix(3)) ?? 0       // 取前三位毫秒
                    }
                }
                times.append(mm * 60 + ss + ms / 1000)
            }
            if times.isEmpty { continue }

            // 去掉所有时间标签后的文本
            let textPart = regex
                .stringByReplacingMatches(
                    in: line,
                    range: NSRange(location: 0, length: ns.length),
                    withTemplate: ""
                )
                .trimmingCharacters(in: .whitespaces)
            for t in times {
                out.append(LyricLine(time: t, text: textPart, translation: nil))
            }
        }
        out.sort { $0.time < $1.time }
        return out
    }

    /// 在时间轴上定位当前应高亮的行索引（返回 <= positionSec 的最后一行）。
    static func locate(lines: [LyricLine], positionSec: Double) -> Int? {
        var idx: Int? = nil
        for (i, line) in lines.enumerated() {
            if line.time <= positionSec { idx = i } else { break }
        }
        return idx
    }
}

// MARK: - 搜索

/// 搜索分类，与 Rust `SearchType`（serde rename_all = "lowercase"）对齐。
enum SearchType: String, Codable, CaseIterable, Identifiable {
    case song = "song"
    case album = "album"
    case artist = "artist"

    var id: String { rawValue }
    var displayName: String {
        switch self {
        case .song: return "歌曲"
        case .album: return "专辑"
        case .artist: return "艺术家"
        }
    }
}

/// 搜索结果项（内部标签枚举，对应 Rust `#[serde(tag = "kind", content = "data")]`）。
enum SearchItem: Codable, Hashable {
    case song(Song)
    case album(Album)
    case artist(Artist)

    enum CodingKeys: String, CodingKey { case kind, data }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let kind = try c.decode(String.self, forKey: .kind)
        switch kind {
        case "song":     self = .song(try c.decode(Song.self, forKey: .data))
        case "album":    self = .album(try c.decode(Album.self, forKey: .data))
        case "artist":   self = .artist(try c.decode(Artist.self, forKey: .data))
        default:
            throw DecodingError.dataCorruptedError(
                forKey: .kind, in: c,
                debugDescription: "未知 search item kind: \(kind)"
            )
        }
    }

    func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .song(let v):   try c.encode("song", forKey: .kind);   try c.encode(v, forKey: .data)
        case .album(let v):  try c.encode("album", forKey: .kind);  try c.encode(v, forKey: .data)
        case .artist(let v): try c.encode("artist", forKey: .kind); try c.encode(v, forKey: .data)
        }
    }

    /// 展示标题。
    var title: String {
        switch self {
        case .song(let s): return s.title
        case .album(let a): return a.name
        case .artist(let a): return a.name
        }
    }

    /// 展示副标题。
    var subtitle: String {
        switch self {
        case .song(let s): return s.artist
        case .album(let a): return a.artist
        case .artist(let a): return "艺术家"
        }
    }

    /// 若为歌曲则返回，否则 nil。
    var song: Song? {
        if case .song(let s) = self { return s } else { return nil }
    }
}

/// 单条搜索结果（含音源来源）。`item` 在 Rust 端为 `#[serde(flatten)]`，
/// 即 kind/data 字段平铺到本对象，故此处自定义编解码实现等价语义。
struct SearchResult: Codable, Hashable, Identifiable {
    let sourceId: String
    let sourceName: String
    let item: SearchItem

    enum CodingKeys: String, CodingKey {
        case sourceId = "source_id"
        case sourceName = "source_name"
        case kind
        case data
    }

    init(sourceId: String, sourceName: String, item: SearchItem) {
        self.sourceId = sourceId
        self.sourceName = sourceName
        self.item = item
    }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        sourceId = try c.decode(String.self, forKey: .sourceId)
        sourceName = try c.decode(String.self, forKey: .sourceName)
        // flatten：kind/data 与外层同级，直接复用同一 decoder 构造 SearchItem
        item = try SearchItem(from: decoder)
    }

    func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        try c.encode(sourceId, forKey: .sourceId)
        try c.encode(sourceName, forKey: .sourceName)
        try item.encode(to: encoder)
    }

    /// Identifiable：音源来源 + 内容复合键。
    var id: String { "\(sourceId)|\(item.title)" }
}

// MARK: - 分页

/// 分页请求（Rust `Page`，默认 offset=0, limit=20）。
struct Page: Codable, Hashable {
    var offset: UInt32
    var limit: UInt32

    init(offset: UInt32 = 0, limit: UInt32 = 20) {
        self.offset = offset
        self.limit = limit
    }

    /// 下一页。
    var next: Page { Page(offset: offset + limit, limit: limit) }
    /// 是否还有更多（基于已知 total）。
    func hasMore(total: UInt32) -> Bool { Int(offset) + Int(limit) < Int(total) }
}

/// 分页结果（Rust `Paged<T>`）。
struct Paged<T: Codable>: Codable {
    let items: [T]
    let total: UInt32
    let offset: UInt32
    let limit: UInt32
}

// MARK: - 排行榜

/// 排行榜。
struct Ranking: Codable, Hashable, Identifiable {
    var sourceId: String
    var rankingId: String
    var name: String
    var coverUrl: String?
    var updateTime: String?
    var songs: [Song]

    enum CodingKeys: String, CodingKey {
        case sourceId = "source_id"
        case rankingId = "ranking_id"
        case name
        case coverUrl = "cover_url"
        case updateTime = "update_time"
        case songs
    }

    init(
        sourceId: String = "",
        rankingId: String = "",
        name: String = "",
        coverUrl: String? = nil,
        updateTime: String? = nil,
        songs: [Song] = []
    ) {
        self.sourceId = sourceId
        self.rankingId = rankingId
        self.name = name
        self.coverUrl = coverUrl
        self.updateTime = updateTime
        self.songs = songs
    }

    var id: String { "\(sourceId):\(rankingId)" }
}

// MARK: - 播放器

/// 播放模式（Rust `PlayMode`，serde rename_all = "snake_case"）。
enum PlayMode: String, Codable, CaseIterable, Identifiable {
    case sequence = "sequence"
    case repeatOne = "repeat_one"
    case shuffle = "shuffle"

    var id: String { rawValue }
    var displayName: String {
        switch self {
        case .sequence: return "顺序播放"
        case .repeatOne: return "单曲循环"
        case .shuffle: return "随机播放"
        }
    }
    /// 系统图标名。
    var iconName: String {
        switch self {
        case .sequence: return "list.number"
        case .repeatOne: return "repeat.1"
        case .shuffle: return "shuffle"
        }
    }
}

/// 播放器状态快照（持久化用）。
struct PlayerState: Codable {
    var current: SongRef?
    var playlistId: String?
    var positionSec: Double
    var volume: Float
    var mode: PlayMode
    var playing: Bool

    enum CodingKeys: String, CodingKey {
        case current
        case playlistId = "playlist_id"
        case positionSec = "position_sec"
        case volume
        case mode
        case playing
    }

    init(
        current: SongRef? = nil,
        playlistId: String? = nil,
        positionSec: Double = 0,
        volume: Float = 0.8,
        mode: PlayMode = .sequence,
        playing: Bool = false
    ) {
        self.current = current
        self.playlistId = playlistId
        self.positionSec = positionSec
        self.volume = volume
        self.mode = mode
        self.playing = playing
    }
}

// MARK: - 播放列表 / 收藏夹

/// 播放列表。
struct Playlist: Codable, Hashable, Identifiable {
    var id: String
    var name: String
    var songs: [SongRef]

    init(id: String = UUID().uuidString, name: String, songs: [SongRef] = []) {
        self.id = id
        self.name = name
        self.songs = songs
    }
}

/// 收藏夹分组。
struct FavoriteGroup: Codable, Hashable, Identifiable {
    var id: String
    var name: String
    var songs: [SongRef]

    init(id: String = UUID().uuidString, name: String, songs: [SongRef] = []) {
        self.id = id
        self.name = name
        self.songs = songs
    }
}

// MARK: - 网络协议源（与 Rust `ProtocolSource` 对齐）

/// 网络协议源（内部标签枚举，对应 `#[serde(tag = "kind", content = "config")]`）。
enum ProtocolSource: Codable, Hashable, Identifiable {
    case smb(SmbConfig)
    case webdav(WebDavConfig)
    case ftp(FtpConfig)
    case dlna(DlnaConfig)
    case nfs(NfsConfig)

    enum CodingKeys: String, CodingKey { case kind, config }

    init(from decoder: Decoder) throws {
        let c = try decoder.container(keyedBy: CodingKeys.self)
        let kind = try c.decode(String.self, forKey: .kind)
        switch kind {
        case "smb":    self = .smb(try c.decode(SmbConfig.self, forKey: .config))
        case "webdav": self = .webdav(try c.decode(WebDavConfig.self, forKey: .config))
        case "ftp":    self = .ftp(try c.decode(FtpConfig.self, forKey: .config))
        case "dlna":   self = .dlna(try c.decode(DlnaConfig.self, forKey: .config))
        case "nfs":    self = .nfs(try c.decode(NfsConfig.self, forKey: .config))
        default:
            throw DecodingError.dataCorruptedError(
                forKey: .kind, in: c,
                debugDescription: "未知 protocol kind: \(kind)"
            )
        }
    }

    func encode(to encoder: Encoder) throws {
        var c = encoder.container(keyedBy: CodingKeys.self)
        switch self {
        case .smb(let v):    try c.encode("smb", forKey: .kind);    try c.encode(v, forKey: .config)
        case .webdav(let v): try c.encode("webdav", forKey: .kind); try c.encode(v, forKey: .config)
        case .ftp(let v):   try c.encode("ftp", forKey: .kind);    try c.encode(v, forKey: .config)
        case .dlna(let v):  try c.encode("dlna", forKey: .kind);  try c.encode(v, forKey: .config)
        case .nfs(let v):   try c.encode("nfs", forKey: .kind);   try c.encode(v, forKey: .config)
        }
    }

    var kindName: String {
        switch self {
        case .smb: return "SMB"
        case .webdav: return "WebDAV"
        case .ftp: return "FTP"
        case .dlna: return "DLNA"
        case .nfs: return "NFS"
        }
    }

    var displayName: String {
        switch self {
        case .smb(let c): return c.share.isEmpty ? "SMB" : "SMB · \(c.share)"
        case .webdav(let c): return c.url.isEmpty ? "WebDAV" : "WebDAV · \(c.url)"
        case .ftp(let c): return c.host.isEmpty ? "FTP" : "FTP · \(c.host)"
        case .dlna(let c): return c.deviceUrl.isEmpty ? "DLNA" : "DLNA · \(c.deviceUrl)"
        case .nfs(let c): return c.host.isEmpty ? "NFS" : "NFS · \(c.host)"
        }
    }

    /// 稳定标识。
    var id: String { displayName + "|" + kindName }
}

struct SmbConfig: Codable, Hashable {
    var host: String
    var port: UInt16?
    var share: String
    var username: String?
    var password: String?
    var workgroup: String?
    var path: String
}

struct WebDavConfig: Codable, Hashable {
    var url: String
    var username: String?
    var password: String?
}

struct FtpConfig: Codable, Hashable {
    var host: String
    var port: UInt16?
    var username: String?
    var password: String?
    var passive: Bool
    var path: String
}

struct DlnaConfig: Codable, Hashable {
    var deviceUrl: String

    enum CodingKeys: String, CodingKey { case deviceUrl = "device_url" }
}

struct NfsConfig: Codable, Hashable {
    var host: String
    var `export`: String
    var path: String
}

/// 协议浏览条目。
struct ProtocolEntry: Codable, Hashable, Identifiable {
    let name: String
    let isDir: Bool
    let size: UInt64?
    let url: String

    enum CodingKeys: String, CodingKey {
        case name
        case isDir = "is_dir"
        case size
        case url
    }

    var id: String { url + "|" + name }
}

// MARK: - 飞牛 NAS 配置（与 core/src/feiniu/mod.rs::FeiniuConfig 对齐）

struct FeiniuConfig: Codable, Hashable {
    var baseUrl: String
    var username: String
    var password: String

    enum CodingKeys: String, CodingKey {
        case baseUrl = "base_url"
        case username
        case password
    }

    init(baseUrl: String = "", username: String = "", password: String = "") {
        self.baseUrl = baseUrl
        self.username = username
        self.password = password
    }
}
