// MARK: - ContentView
// 应用主内容：NavigationSplitView（侧边栏 + 详情） + 底部 PlaybackBar。
// 侧边栏列举所有功能页；详情区根据选中页渲染。

import SwiftUI

/// 侧边栏可选页面。
enum SidebarPage: String, CaseIterable, Identifiable, Hashable {
    case search
    case playlist
    case favorites
    case lyrics
    case leaderboard
    case localMusic
    case nas
    case settings

    var id: String { rawValue }

    /// 页面显示名称。
    var title: String {
        switch self {
        case .search: return "搜索"
        case .playlist: return "播放列表"
        case .favorites: return "收藏"
        case .lyrics: return "歌词"
        case .leaderboard: return "排行榜"
        case .localMusic: return "本地音乐"
        case .nas: return "NAS"
        case .settings: return "设置"
        }
    }

    /// SF Symbol 图标。
    var symbolName: String {
        switch self {
        case .search: return "magnifyingglass"
        case .playlist: return "music.note.list"
        case .favorites: return "heart"
        case .lyrics: return "text.quote"
        case .leaderboard: return "chart.bar.xaxis"
        case .localMusic: return "music.note.house"
        case .nas: return "externaldrive.connected.to.line.below"
        case .settings: return "gearshape"
        }
    }
}

struct ContentView: View {
    @EnvironmentObject var player: PlayerService
    @State private var selection: SidebarPage? = .search
    @State private var sidebarVisibility: NavigationSplitViewVisibility = .all

    var body: some View {
        NavigationSplitView(columnVisibility: $sidebarVisibility) {
            sidebar
                .navigationSplitViewColumnWidth(min: 200, ideal: 220, max: 280)
        } detail: {
            detailView
        }
    }

    // MARK: - 侧边栏

    private var sidebar: some View {
        VStack(spacing: 0) {
            // 品牌头部：品牌名为首屏最醒目文本
            HStack(spacing: NghSpacing.s3) {
                Image(systemName: "music.note")
                    .font(.title2)
                    .foregroundColor(Color.nghPrimary)
                Text("逆光音乐")
                    .font(.title2.bold())
                    .foregroundColor(Color.nghText)
                Spacer()
            }
            .padding(.horizontal, NghSpacing.s4)
            .padding(.vertical, NghSpacing.s4)

            Divider()

            List(SidebarPage.allCases, selection: $selection) { page in
                NavigationLink(value: page) {
                    Label(page.title, systemImage: page.symbolName)
                }
            }
            .listStyle(.sidebar)

            Spacer()
            Divider()
            // 底部信息条
            HStack {
                Image(systemName: "music.mic")
                    .foregroundColor(Color.nghTextSecondary)
                Text("NGHMusic · macOS")
                    .font(.caption)
                    .foregroundColor(Color.nghTextSecondary)
                Spacer()
            }
            .padding(.horizontal, NghSpacing.s4)
            .padding(.vertical, NghSpacing.s3)
        }
        .background(Color.nghSidebarBackground)
    }

    // MARK: - 详情视图

    @ViewBuilder
    private var detailView: some View {
        VStack(spacing: 0) {
            // 详情区主体
            Group {
                switch selection {
                case .search:
                    SearchView()
                case .playlist:
                    PlaylistView(player: player)
                case .favorites:
                    FavoritesView()
                case .lyrics:
                    LyricsView(player: player)
                case .leaderboard:
                    LeaderboardView()
                case .localMusic:
                    LocalMusicView()
                case .nas:
                    NasView()
                case .settings:
                    SettingsView()
                case .none:
                    Text("请选择左侧功能页")
                        .foregroundColor(Color.nghTextSecondary)
                }
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            Divider()
            // 底部播放控制条
            PlaybackBar(player: player)
                .frame(height: 72)
        }
        .navigationTitle(selection?.title ?? "")
        .background(Color.nghBackground)
    }
}

#Preview {
    ContentView()
        .environmentObject(PlayerService())
        .frame(width: 1100, height: 720)
}
