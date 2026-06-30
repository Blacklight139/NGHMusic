// MARK: - LeaderboardView
// 职责：排行榜页，展示排行榜卡片网格，简约风格占位。

import SwiftUI

struct LeaderboardView: View {
    @State private var boards: [Leaderboard] = [
        Leaderboard(id: "l1", sourceId: "demo", name: "热歌榜", coverUrl: nil,
                    songs: []),
        Leaderboard(id: "l2", sourceId: "demo", name: "新歌榜", coverUrl: nil,
                    songs: []),
        Leaderboard(id: "l3", sourceId: "demo", name: "飙升榜", coverUrl: nil,
                    songs: []),
    ]

    var body: some View {
        PageContainer(title: "排行榜", subtitle: "各音源热门榜单") {
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 160), spacing: AppTheme.space3)],
                      spacing: AppTheme.space3) {
                ForEach(boards) { board in
                    Card(title: board.name, subtitle: "\(board.songs.count) 首")
                }
            }
        }
    }
}
