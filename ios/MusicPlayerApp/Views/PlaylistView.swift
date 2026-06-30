// MARK: - PlaylistView
// 职责：播放列表页，展示当前播放列表的歌曲，简约风格占位。

import SwiftUI

struct PlaylistView: View {
    @State private var songs: [Song] = [
        Song(id: "p1", sourceId: "demo", title: "播放列表曲目一", artists: ["艺术家A"],
             album: nil, coverUrl: nil, durationMs: 200000, lyricUrl: nil,
             playUrl: nil, localPath: nil, origin: .online(sourceId: "demo", playUrl: "")),
        Song(id: "p2", sourceId: "demo", title: "播放列表曲目二", artists: ["艺术家B"],
             album: nil, coverUrl: nil, durationMs: 175000, lyricUrl: nil,
             playUrl: nil, localPath: nil, origin: .online(sourceId: "demo", playUrl: "")),
    ]

    var body: some View {
        PageContainer(title: "播放列表", subtitle: "当前队列共 \(songs.count) 首") {
            if songs.isEmpty {
                EmptyState(text: "播放列表为空，去搜索添加歌曲吧")
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
