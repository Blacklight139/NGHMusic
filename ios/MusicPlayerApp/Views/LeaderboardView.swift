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
            LazyVGrid(columns: [GridItem(.adaptive(minimum: 160), spacing: NghSpacing.s3)],
                      spacing: NghSpacing.s3) {
                ForEach(boards) { board in
                    Button { /* press 反馈，暂无跳转 */ } label: {
                        Card(title: board.name, subtitle: "\(board.songs.count) 首",
                             systemImage: boardIcon(for: board.name))
                    }
                    .nghPressableStyle()
                    .transition(.opacity.combined(with: .move(edge: .top)))
                }
            }
            // iOS 15+：卡片出现时 staggered fade-in。
            .animation(.easeOut(duration: 0.3), value: boards.count)
        }
    }

    /// 榜单图标映射（纯样式，不影响数据流）
    private func boardIcon(for name: String) -> String {
        switch name {
        case let n where n.contains("热"): return "flame.fill"
        case let n where n.contains("新"): return "sparkles"
        case let n where n.contains("飙升"): return "chart.line.uptrend.xyaxis"
        default: return "trophy.fill"
        }
    }
}
