// MARK: - SearchView
// 职责：搜索页，搜索栏 + 结果列表。
// 对齐 docs/sound-source-api.md 聚合搜索：输入关键字调用 MusicCoreBridge.search，
// 展示歌曲/专辑/艺术家；点击歌曲调用 PlayerManager.play(song:in:) 入队播放。
// 脚手架阶段（未链接核心库）搜索抛 unavailable，自动降级到占位数据。

import SwiftUI

struct SearchView: View {
    @EnvironmentObject var player: PlayerManager
    @State private var keyword: String = ""
    @State private var songs: [Song] = []
    @State private var loading = false
    @State private var errorMessage: String?
    @State private var hasSearched = false

    /// 占位歌曲（未链接核心库 / 搜索失败时展示，便于 UI 验证）。
    /// playUrl 为 nil → IOS-001 保护下点击不会进入播放状态。
    private let placeholderSongs: [Song] = [
        Song(id: "s1", sourceId: "demo", title: "示例歌曲一", artists: ["艺术家A"],
             album: "专辑X", coverUrl: nil, durationMs: 210000, lyricUrl: nil,
             playUrl: nil, localPath: nil, origin: .online(sourceId: "demo", playUrl: "")),
        Song(id: "s2", sourceId: "demo", title: "示例歌曲二", artists: ["艺术家B"],
             album: nil, coverUrl: nil, durationMs: 184000, lyricUrl: nil,
             playUrl: nil, localPath: nil, origin: .online(sourceId: "demo", playUrl: "")),
    ]

    var body: some View {
        PageContainer(title: "搜索", subtitle: "跨音源聚合检索") {
            VStack(spacing: NghSpacing.s3) {
                HStack(spacing: NghSpacing.s2) {
                    TextField("输入歌曲 / 艺术家 / 专辑", text: $keyword)
                        .textFieldStyle(.roundedBorder)
                        .submitLabel(.search)
                        .onSubmit { performSearch() }
                    Button(action: performSearch) {
                        Text("搜索").foregroundColor(.white)
                            .padding(.horizontal, NghSpacing.s4).padding(.vertical, NghSpacing.s2)
                            .background(Color.nghPrimary).cornerRadius(NghRadius.md)
                    }
                }
                if let errorMessage = errorMessage {
                    Text(errorMessage).foregroundColor(Color.nghDanger).font(.caption)
                }
                if loading { ProgressView("搜索中…") }
                if !hasSearched && songs.isEmpty {
                    EmptyState(text: "输入关键字后点击搜索，跨音源聚合检索")
                } else if songs.isEmpty && !loading {
                    EmptyState(text: "未找到匹配结果")
                } else {
                    SongListView(songs: songs, currentSongId: player.currentSong?.id) { song in
                        player.play(song: song, in: songs)
                    }
                }
            }
        }
    }

    private func performSearch() {
        guard !keyword.isEmpty else { return }
        loading = true
        errorMessage = nil
        hasSearched = true
        Task {
            do {
                let result = try await MusicCoreBridge.search(keyword, page: 1, pageSize: 20)
                songs = result.songs
            } catch {
                // 脚手架阶段 music-core 未链接时降级到占位数据
                errorMessage = "（占位）\(error.localizedDescription)"
                songs = placeholderSongs
            }
            loading = false
        }
    }
}

/// 通用歌曲列表，onPlay 回调点击播放。
struct SongListView: View {
    let songs: [Song]
    /// 当前正在播放曲目 id（用于主色高亮）。
    var currentSongId: String? = nil
    var onPlay: ((Song) -> Void)? = nil

    var body: some View {
        // Linear 风格：spacing 0，每行自带底部分割线，避免卡片间距造成的视觉碎片。
        VStack(spacing: 0) {
            ForEach(Array(songs.enumerated()), id: \.element.id) { index, song in
                Button { onPlay?(song) } label: {
                    SongRow(index: index + 1, song: song, isCurrent: song.id == currentSongId)
                }
                .nghPressableStyle()
                .transition(.opacity.combined(with: .move(edge: .top)))
            }
        }
        // iOS 15+：列表项出现时 staggered fade-in。
        .animation(.easeOut(duration: 0.3), value: songs.count)
    }
}

struct SongRow: View {
    let index: Int
    let song: Song
    /// 是否为当前正在播放的曲目（播放列表用于高亮）。
    var isCurrent: Bool = false

    var body: some View {
        // Linear 风格：行即交互时才用卡片；这里改为无背景的纯行 + 底部分割线。
        VStack(spacing: 0) {
            HStack(spacing: NghSpacing.s3) {
                // 当前播放曲目用主色 play 指示替代序号，强化高亮。
                if isCurrent {
                    Image(systemName: "play.circle.fill")
                        .frame(width: 20)
                        .foregroundColor(Color.nghPrimary)
                        .font(.caption)
                } else {
                    Text("\(index)")
                        .frame(width: 20)
                        .foregroundColor(Color.nghTextTertiary)
                        .font(.caption)
                }
                // 封面占位：nghPrimarySoft 背景 + NghRadius.sm 圆角，与 Card 图标一致。
                Image(systemName: isCurrent ? "play.fill" : "music.note")
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundColor(Color.nghPrimary)
                    .frame(width: 36, height: 36)
                    .background(Color.nghPrimarySoft)
                    .clipShape(RoundedRectangle(cornerRadius: NghRadius.sm, style: .continuous))
                VStack(alignment: .leading, spacing: NghSpacing.s1) {
                    Text(song.title)
                        .fontWeight(isCurrent ? .semibold : .medium)
                        .lineLimit(1)
                        .foregroundColor(isCurrent ? Color.nghPrimary : Color.nghText)
                    Text(song.artists.joined(separator: " / "))
                        .font(.caption)
                        .foregroundColor(Color.nghTextSecondary)
                        .lineLimit(1)
                }
                Spacer(minLength: 0)
                if let durationText = durationText {
                    Text(durationText)
                        .font(.caption)
                        .foregroundColor(isCurrent ? Color.nghPrimary : Color.nghTextTertiary)
                }
                // 来源标签：更小更克制。
                Text(originTag)
                    .font(.caption2)
                    .padding(.horizontal, NghSpacing.s1)
                    .padding(.vertical, NghSpacing.s1)
                    .background(originColor.opacity(0.1))
                    .foregroundColor(originColor)
                    .clipShape(Capsule())
            }
            .padding(.horizontal, NghSpacing.s4)
            .padding(.vertical, NghSpacing.s3)
            Divider()
        }
    }

    /// 时长文本「m:ss」，durationMs 为空时不展示。
    private var durationText: String? {
        guard let ms = song.durationMs, ms > 0 else { return nil }
        let totalSeconds = Int(ms / 1000)
        let minutes = totalSeconds / 60
        let seconds = totalSeconds % 60
        return String(format: "%d:%02d", minutes, seconds)
    }

    /// 来源标签：根据 SongOrigin 显示「在线/本地/NAS」。
    private var originTag: String {
        switch song.origin {
        case .online: return "在线"
        case .local: return "本地"
        case .nas: return "NAS"
        }
    }

    private var originColor: Color {
        switch song.origin {
        case .online: return Color.nghPrimary
        case .local: return Color.nghSuccess
        case .nas: return Color.nghWarning
        }
    }
}
