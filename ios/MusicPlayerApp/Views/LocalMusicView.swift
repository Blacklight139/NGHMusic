// MARK: - LocalMusicView
// 职责：本地音乐页，按歌曲/专辑/艺术家/文件夹浏览，简约风格占位。

import SwiftUI

struct LocalMusicView: View {
    @State private var songs: [Song] = [
        Song(id: "lo1", sourceId: "local", title: "本地示例一", artists: ["未知艺术家"],
             album: "本地专辑", coverUrl: nil, durationMs: 198000, lyricUrl: nil,
             playUrl: nil, localPath: "/music/a.mp3",
             origin: .local(path: "/music/a.mp3")),
        Song(id: "lo2", sourceId: "local", title: "本地示例二", artists: ["艺术家C"],
             album: nil, coverUrl: nil, durationMs: 165000, lyricUrl: nil,
             playUrl: nil, localPath: "/music/b.flac",
             origin: .local(path: "/music/b.flac")),
    ]
    @State private var selectedFilter = 0
    private let filters = ["歌曲", "专辑", "艺术家", "文件夹"]

    var body: some View {
        PageContainer(title: "本地音乐", subtitle: "扫描本地目录并播放") {
            VStack(alignment: .leading, spacing: AppTheme.space3) {
                Picker("", selection: $selectedFilter) {
                    ForEach(0..<filters.count, id: \.self) { Text(filters[$0]).tag($0) }
                }
                .pickerStyle(.segmented)

                if songs.isEmpty {
                    EmptyState(text: "尚未扫描本地音乐，前往设置添加目录")
                } else {
                    VStack(spacing: AppTheme.space2) {
                        ForEach(Array(songs.enumerated()), id: \.element.id) { index, song in
                            SongRow(index: index + 1, song: song)
                        }
                    }
                }
            }
        }
    }
}
