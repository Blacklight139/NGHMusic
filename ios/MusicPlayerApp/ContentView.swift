// MARK: - ContentView
// 职责：根视图，品牌位「逆光音乐 / NGHMusic」+ TabView 导航（搜索/列表/收藏/排行榜/本地/设置）+ 底部 PlaybackBar。
// 豆包风格：主色 #4E6EF2，背景 #F7F8FA/#FFFFFF，图标统一 SF Symbols。
// 集成方式：由 MusicPlayerApp 直接挂载，无需额外配置。

import SwiftUI

struct ContentView: View {
    @EnvironmentObject var player: PlayerManager
    @State private var selectedTab: AppTab = .search
    @State private var showLyrics = false

    var body: some View {
        VStack(spacing: 0) {
            BrandBar()
            TabView(selection: $selectedTab) {
                SearchView()
                    .tabItem { Label(AppTab.search.title, systemImage: AppTab.search.icon) }
                    .tag(AppTab.search)
                    .transition(.opacity.combined(with: .move(edge: .trailing)))

                PlaylistView()
                    .tabItem { Label(AppTab.playlist.title, systemImage: AppTab.playlist.icon) }
                    .tag(AppTab.playlist)
                    .transition(.opacity.combined(with: .move(edge: .trailing)))

                FavoritesView()
                    .tabItem { Label(AppTab.favorites.title, systemImage: AppTab.favorites.icon) }
                    .tag(AppTab.favorites)
                    .transition(.opacity.combined(with: .move(edge: .trailing)))

                LeaderboardView()
                    .tabItem { Label(AppTab.leaderboard.title, systemImage: AppTab.leaderboard.icon) }
                    .tag(AppTab.leaderboard)
                    .transition(.opacity.combined(with: .move(edge: .trailing)))

                LocalMusicView()
                    .tabItem { Label(AppTab.local.title, systemImage: AppTab.local.icon) }
                    .tag(AppTab.local)
                    .transition(.opacity.combined(with: .move(edge: .trailing)))

                NasView()
                    .tabItem { Label(AppTab.nas.title, systemImage: AppTab.nas.icon) }
                    .tag(AppTab.nas)
                    .transition(.opacity.combined(with: .move(edge: .trailing)))

                SettingsView()
                    .tabItem { Label(AppTab.settings.title, systemImage: AppTab.settings.icon) }
                    .tag(AppTab.settings)
                    .transition(.opacity.combined(with: .move(edge: .trailing)))
            }
            .tint(Color.nghPrimary)
            // iOS 15+：selection 变化时柔和过渡（opacity + move），200ms easeInOut。
            .animation(.easeInOut(duration: 0.2), value: selectedTab)

            if player.currentSong != nil {
                PlaybackBar(showLyrics: $showLyrics)
                    .environmentObject(player)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
            }
        }
        // iOS 15+：PlaybackBar 出现/消失弹簧动画（有曲目播放时滑入，无曲目时滑出）。
        .animation(.spring(response: 0.3, dampingFraction: 0.8), value: player.currentSong != nil)
        .sheet(isPresented: $showLyrics) {
            LyricsView()
                .environmentObject(player)
        }
    }
}

// MARK: - BrandBar
// 顶部品牌位：SF Symbols 图标 + 「逆光音乐」/ NGHMusic。
struct BrandBar: View {
    var body: some View {
        HStack(spacing: NghSpacing.s2) {
            Image(systemName: "music.note")
                .font(.system(size: 16, weight: .semibold))
                .foregroundColor(.white)
                .frame(width: 28, height: 28)
                .background(Color.nghPrimary)
                .clipShape(RoundedRectangle(cornerRadius: NghRadius.sm, style: .continuous))
            VStack(alignment: .leading, spacing: 0) {
                Text("逆光音乐")
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundColor(Color.nghText)
                Text("NGHMusic")
                    .font(.system(size: 10))
                    .foregroundColor(Color.nghTextTertiary)
            }
            Spacer()
        }
        .padding(.horizontal, NghSpacing.s4)
        .padding(.vertical, NghSpacing.s2)
        .background(Color.nghSurface)
        .overlay(Rectangle().frame(height: 1).foregroundColor(Color.nghBorderSoft), alignment: .bottom)
    }
}

// MARK: - AppTab
enum AppTab: Hashable {
    case search, playlist, favorites, leaderboard, local, nas, settings
    var title: String {
        switch self {
        case .search: return "搜索"
        case .playlist: return "播放列表"
        case .favorites: return "收藏"
        case .leaderboard: return "排行榜"
        case .local: return "本地音乐"
        case .nas: return "NAS"
        case .settings: return "设置"
        }
    }
    var icon: String {
        switch self {
        case .search: return "magnifyingglass"
        case .playlist: return "music.note.list"
        case .favorites: return "heart"
        case .leaderboard: return "trophy"
        case .local: return "folder"
        case .nas: return "externaldrive.connected.to.line.below"
        case .settings: return "gearshape"
        }
    }
}

#Preview {
    ContentView().environmentObject(PlayerManager())
}
