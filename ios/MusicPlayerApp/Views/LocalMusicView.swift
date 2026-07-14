// MARK: - LocalMusicView
// 职责：本地音乐页，按歌曲/专辑/艺术家/文件夹浏览。
// 对齐 docs/sound-source-api.md 本地源：通过 MusicCoreBridge.listLocalSongs 拉取本地歌曲。
// 脚手架阶段（未链接核心库）listLocalSongs 返回占位数据；点击歌曲入队播放。

import SwiftUI

struct LocalMusicView: View {
    @EnvironmentObject var player: PlayerManager
    @State private var songs: [Song] = []
    @State private var loading = false
    @State private var selectedFilter = 0
    private let filters = ["歌曲", "专辑", "艺术家", "文件夹"]

    var body: some View {
        PageContainer(title: "本地音乐", subtitle: "扫描本地目录并播放") {
            VStack(alignment: .leading, spacing: NghSpacing.s3) {
                Picker("", selection: $selectedFilter) {
                    ForEach(0..<filters.count, id: \.self) { Text(filters[$0]).tag($0) }
                }
                .pickerStyle(.segmented)

                if loading { ProgressView("加载中…") }

                if songs.isEmpty {
                    EmptyState(text: "尚未扫描本地音乐，前往设置添加目录")
                } else {
                    SongListView(songs: songs) { song in
                        player.play(song: song, in: songs)
                    }
                }
            }
        }
        .onAppear {
            Task { await loadLocalSongs() }
        }
    }

    /// 拉取本地歌曲；脚手架阶段返回占位数据。
    private func loadLocalSongs() async {
        loading = true
        defer { loading = false }
        do {
            songs = try await MusicCoreBridge.listLocalSongs()
        } catch {
            songs = []
        }
    }
}
