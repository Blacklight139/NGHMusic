// MARK: - LeaderboardView
// 职责：排行榜页，展示各音源排行榜卡片网格。
// 对齐 docs/sound-source-api.md：通过 MusicCoreBridge.listSources 获取已启用音源，
// 再对每个音源调用 MusicCoreBridge.getLeaderboards 聚合榜单。
// 脚手架阶段（未链接核心库）抛 unavailable 时降级到占位榜单。

import SwiftUI

struct LeaderboardView: View {
    @EnvironmentObject var player: PlayerManager
    @State private var boards: [Leaderboard] = []
    @State private var loading = false
    @State private var errorMessage: String?

    /// 占位榜单（未链接核心库 / 获取失败时展示）。
    private let placeholderBoards: [Leaderboard] = [
        Leaderboard(id: "l1", sourceId: "demo", name: "热歌榜", coverUrl: nil, songs: []),
        Leaderboard(id: "l2", sourceId: "demo", name: "新歌榜", coverUrl: nil, songs: []),
        Leaderboard(id: "l3", sourceId: "demo", name: "飙升榜", coverUrl: nil, songs: []),
    ]

    private var displayBoards: [Leaderboard] {
        boards.isEmpty ? placeholderBoards : boards
    }

    var body: some View {
        PageContainer(title: "排行榜", subtitle: "各音源热门榜单") {
            if loading { ProgressView("加载中…") }
            if let errorMessage = errorMessage {
                Text(errorMessage).foregroundColor(Color.nghDanger).font(.caption)
            }
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 160), spacing: NghSpacing.s3)],
                      spacing: NghSpacing.s3) {
                ForEach(displayBoards) { board in
                    Button {
                        playBoard(board)
                    } label: {
                        Card(title: board.name, subtitle: "\(board.songs.count) 首",
                             systemImage: boardIcon(for: board.name))
                    }
                    .nghPressableStyle()
                    .transition(.opacity.combined(with: .move(edge: .top)))
                }
            }
            // iOS 15+：卡片出现时 staggered fade-in。
            .animation(.easeOut(duration: 0.3), value: displayBoards.count)
        }
        .onAppear {
            Task { await loadLeaderboards() }
        }
    }

    /// 榜单图标映射（纯样式，不影响数据流）。
    private func boardIcon(for name: String) -> String {
        switch name {
        case let n where n.contains("热"): return "flame.fill"
        case let n where n.contains("新"): return "sparkles"
        case let n where n.contains("飙升"): return "chart.line.uptrend.xyaxis"
        default: return "trophy.fill"
        }
    }

    /// 播放整个榜单：将榜单歌曲入队播放第一首。
    private func playBoard(_ board: Leaderboard) {
        guard let first = board.songs.first else { return }
        player.play(song: first, in: board.songs)
    }

    /// 聚合各启用音源的排行榜；任一音源失败跳过。
    private func loadLeaderboards() async {
        loading = true
        defer { loading = false }
        do {
            let sources = try await MusicCoreBridge.listSources()
            var aggregated: [Leaderboard] = []
            for source in sources where source.enabled {
                if let boards = try? await MusicCoreBridge.getLeaderboards(sourceId: source.id) {
                    aggregated.append(contentsOf: boards)
                }
            }
            boards = aggregated
            errorMessage = aggregated.isEmpty ? "（占位）未获取到榜单数据" : nil
        } catch {
            boards = []
            errorMessage = "（占位）\(error.localizedDescription)"
        }
    }
}
