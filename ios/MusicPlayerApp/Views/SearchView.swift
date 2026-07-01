// MARK: - SearchView
// 职责：搜索页，搜索栏 + 结果列表，简约风格占位数据。
// 对齐桌面端 pages/search.js：输入关键字调用 MusicCore.search，展示歌曲/专辑/艺术家。

import SwiftUI

struct SearchView: View {
    @State private var keyword: String = ""
    @State private var songs: [Song] = []
    @State private var loading = false
    @State private var errorMessage: String?

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
                SongListView(songs: songs.isEmpty ? placeholderSongs : songs)
            }
        }
    }

    private func performSearch() {
        guard !keyword.isEmpty else { return }
        loading = true
        errorMessage = nil
        Task {
            do {
                let result = try await MusicCore.search(keyword, page: 1, pageSize: 20)
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

struct SongListView: View {
    let songs: [Song]
    var body: some View {
        VStack(spacing: NghSpacing.s3) {
            ForEach(Array(songs.enumerated()), id: \.element.id) { index, song in
                Button { /* press 反馈，暂无跳转 */ } label: { SongRow(index: index + 1, song: song) }
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
    var body: some View {
        HStack(spacing: NghSpacing.s3) {
            Text("\(index)").frame(width: 24)
                .foregroundColor(Color.nghTextTertiary).font(.caption)
            VStack(alignment: .leading, spacing: 2) {
                Text(song.title).fontWeight(.medium).lineLimit(1)
                    .foregroundColor(Color.nghText)
                Text(song.artists.joined(separator: " / "))
                    .font(.caption).foregroundColor(Color.nghTextSecondary).lineLimit(1)
            }
            Spacer()
            Text("在线").font(.caption2)
                .padding(.horizontal, NghSpacing.s2).padding(.vertical, 2)
                .background(Color.nghPrimarySoft).foregroundColor(Color.nghPrimary)
                .clipShape(Capsule())
        }
        .padding(NghSpacing.s4)
        .background(RoundedRectangle(cornerRadius: NghRadius.md, style: .continuous).fill(Color.nghSurface))
        .nghCardShadow()
    }
}
