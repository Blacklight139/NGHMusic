// MARK: - ContentView
// 职责：根视图，TabView 导航（搜索/列表/收藏/排行榜/本地/设置）+ 底部迷你播放器。
// 简约风格：主色 #1db954，背景 #fff/#f5f5f5，与桌面端布局对齐。
// 集成方式：由 MusicPlayerApp 直接挂载，无需额外配置。

import SwiftUI

struct ContentView: View {
    @EnvironmentObject var player: PlayerManager
    @State private var selectedTab: AppTab = .search
    @State private var showLyrics = false

    var body: some View {
        VStack(spacing: 0) {
            TabView(selection: $selectedTab) {
                SearchView()
                    .tabItem { Label(AppTab.search.title, systemImage: AppTab.search.icon) }
                    .tag(AppTab.search)

                PlaylistView()
                    .tabItem { Label(AppTab.playlist.title, systemImage: AppTab.playlist.icon) }
                    .tag(AppTab.playlist)

                FavoritesView()
                    .tabItem { Label(AppTab.favorites.title, systemImage: AppTab.favorites.icon) }
                    .tag(AppTab.favorites)

                LeaderboardView()
                    .tabItem { Label(AppTab.leaderboard.title, systemImage: AppTab.leaderboard.icon) }
                    .tag(AppTab.leaderboard)

                LocalMusicView()
                    .tabItem { Label(AppTab.local.title, systemImage: AppTab.local.icon) }
                    .tag(AppTab.local)

                SettingsView()
                    .tabItem { Label(AppTab.settings.title, systemImage: AppTab.settings.icon) }
                    .tag(AppTab.settings)
            }
            .tint(AppTheme.primary)

            MiniPlayerBar(showLyrics: $showLyrics)
                .environmentObject(player)
        }
        .sheet(isPresented: $showLyrics) {
            LyricsView()
                .environmentObject(player)
        }
    }
}

// MARK: - MiniPlayerBar
// 底部迷你播放组件：封面/标题/艺术家 + 上一首/播放/下一首 + 进度条 + 音量。
struct MiniPlayerBar: View {
    @EnvironmentObject var player: PlayerManager
    @Binding var showLyrics: Bool

    var body: some View {
        VStack(spacing: 4) {
            ProgressView(value: Double(player.position), total: max(Double(player.duration), 1))
                .progressViewStyle(.linear)
                .tint(AppTheme.primary)

            HStack(spacing: AppTheme.space4) {
                // 左侧：曲目信息
                RoundedRectangle(cornerRadius: AppTheme.radius)
                    .fill(LinearGradient(colors: [AppTheme.primary, AppTheme.primaryDark],
                                         startPoint: .topLeading, endPoint: .bottomTrailing))
                    .frame(width: 40, height: 40)
                VStack(alignment: .leading, spacing: 2) {
                    Text(player.currentSong?.title ?? "未在播放")
                        .font(.subheadline).fontWeight(.medium)
                        .lineLimit(1)
                    Text(player.currentSong?.artists.joined(separator: " / ") ?? "—")
                        .font(.caption)
                        .foregroundColor(AppTheme.textMuted)
                        .lineLimit(1)
                }
                Spacer()

                // 中间：播放控制
                HStack(spacing: AppTheme.space3) {
                    Button(action: { player.toPrev() }) {
                        Image(systemName: "backward.fill")
                    }
                    Button(action: { player.isPlaying ? player.pause() : player.resume() }) {
                        Image(systemName: player.isPlaying ? "pause.fill" : "play.fill")
                            .font(.title3)
                    }
                    Button(action: { player.toNext() }) {
                        Image(systemName: "forward.fill")
                    }
                }
                .foregroundColor(AppTheme.text)

                // 右侧：歌词/模式/音量
                HStack(spacing: AppTheme.space3) {
                    Button(action: { showLyrics = true }) {
                        Image(systemName: "text.quote")
                    }
                    Button(action: { player.toggleMode() }) {
                        Image(systemName: player.modeIcon)
                    }
                    Image(systemName: "speaker.fill")
                        .foregroundColor(AppTheme.textMuted)
                    Slider(value: Binding(get: { Double(player.volume) },
                                          set: { player.setVolume(Float($0)) }),
                           in: 0...1).frame(width: 90).tint(AppTheme.primary)
                }
                .foregroundColor(AppTheme.text)
            }
            .padding(.horizontal, AppTheme.space5)
            .padding(.vertical, AppTheme.space2)
            .background(AppTheme.bg)
            .overlay(Rectangle().frame(height: 1).foregroundColor(AppTheme.border), alignment: .top)
        }
    }
}

// MARK: - AppTab
enum AppTab: Hashable {
    case search, playlist, favorites, leaderboard, local, settings
    var title: String {
        switch self {
        case .search: return "搜索"
        case .playlist: return "播放列表"
        case .favorites: return "收藏"
        case .leaderboard: return "排行榜"
        case .local: return "本地音乐"
        case .settings: return "设置"
        }
    }
    var icon: String {
        switch self {
        case .search: return "magnifyingglass"
        case .playlist: return "list.bullet"
        case .favorites: return "star"
        case .leaderboard: return "trophy"
        case .local: return "folder"
        case .settings: return "gearshape"
        }
    }
}

#Preview {
    ContentView().environmentObject(PlayerManager())
}
