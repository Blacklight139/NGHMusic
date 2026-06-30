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
            VStack(spacing: AppTheme.space3) {
                HStack(spacing: AppTheme.space2) {
                    TextField("输入歌曲 / 艺术家 / 专辑", text: $keyword)
                        .textFieldStyle(.roundedBorder)
                    Button(action: performSearch) {
                        Text("搜索").foregroundColor(.white)
                            .padding(.horizontal, AppTheme.space4).padding(.vertical, AppTheme.space3)
                            .background(AppTheme.primary).cornerRadius(AppTheme.radius)
                    }
                }
                if let errorMessage = errorMessage {
                    Text(errorMessage).foregroundColor(.red).font(.caption)
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
        VStack(spacing: AppTheme.space2) {
            ForEach(Array(songs.enumerated()), id: \.element.id) { index, song in
                SongRow(index: index + 1, song: song)
            }
        }
    }
}

struct SongRow: View {
    let index: Int
    let song: Song
    var body: some View {
        HStack(spacing: AppTheme.space3) {
            Text("\(index)").frame(width: 24)
                .foregroundColor(AppTheme.textMuted).font(.caption)
            VStack(alignment: .leading, spacing: 2) {
                Text(song.title).fontWeight(.medium).lineLimit(1)
                Text(song.artists.joined(separator: " / "))
                    .font(.caption).foregroundColor(AppTheme.textMuted).lineLimit(1)
            }
            Spacer()
            Text("在线").font(.caption2)
                .padding(.horizontal, AppTheme.space2).padding(.vertical, 2)
                .background(AppTheme.bgAlt).cornerRadius(999)
                .overlay(Capsule().stroke(AppTheme.border, lineWidth: 1))
                .foregroundColor(AppTheme.textMuted)
        }
        .padding(AppTheme.space3)
        .background(AppTheme.bg)
        .cornerRadius(AppTheme.radius)
        .overlay(RoundedRectangle(cornerRadius: AppTheme.radius).stroke(AppTheme.border, lineWidth: 1))
    }
}
