// MARK: - ContentView
// 职责：根视图，品牌位「逆光音乐 / NGHMusic」+ TabView 导航（搜索/列表/收藏/排行榜/本地/设置）+ 底部迷你播放器。
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

                SettingsView()
                    .tabItem { Label(AppTab.settings.title, systemImage: AppTab.settings.icon) }
                    .tag(AppTab.settings)
                    .transition(.opacity.combined(with: .move(edge: .trailing)))
            }
            .tint(Color.nghPrimary)
            // iOS 15+：selection 变化时柔和过渡（opacity + move），200ms easeInOut。
            .animation(.easeInOut(duration: 0.2), value: selectedTab)

            if player.currentSong != nil {
                MiniPlayerBar(showLyrics: $showLyrics)
                    .environmentObject(player)
                    .transition(.move(edge: .bottom).combined(with: .opacity))
            }
        }
        // iOS 15+：MiniPlayer 出现/消失弹簧动画（有曲目播放时滑入，无曲目时滑出）。
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

// MARK: - MiniPlayerBar
// 底部迷你播放组件：封面/标题/艺术家 + 上一首/播放/下一首 + 进度条 + 音量。
struct MiniPlayerBar: View {
    @EnvironmentObject var player: PlayerManager
    @Binding var showLyrics: Bool

    var body: some View {
        VStack(spacing: 0) {
            ProgressView(value: Double(player.position), total: max(Double(player.duration), 1))
                .progressViewStyle(.linear)
                .tint(Color.nghPrimary)

            HStack(spacing: NghSpacing.s4) {
                // 左侧：曲目信息
                RoundedRectangle(cornerRadius: NghRadius.sm, style: .continuous)
                    .fill(LinearGradient(colors: [Color.nghPrimary, Color.nghPrimaryHover],
                                         startPoint: .topLeading, endPoint: .bottomTrailing))
                    .frame(width: 40, height: 40)
                VStack(alignment: .leading, spacing: 2) {
                    Text(player.currentSong?.title ?? "未在播放")
                        .font(.subheadline).fontWeight(.medium)
                        .foregroundColor(Color.nghText)
                        .lineLimit(1)
                    Text(player.currentSong?.artists.joined(separator: " / ") ?? "—")
                        .font(.caption)
                        .foregroundColor(Color.nghTextSecondary)
                        .lineLimit(1)
                }
                Spacer()

                // 中间：播放控制
                HStack(spacing: NghSpacing.s3) {
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
                .foregroundColor(Color.nghText)

                // 右侧：歌词/模式/音量
                HStack(spacing: NghSpacing.s3) {
                    Button(action: { showLyrics = true }) {
                        Image(systemName: "text.quote")
                    }
                    Button(action: { player.toggleMode() }) {
                        Image(systemName: player.modeIcon)
                    }
                    Image(systemName: "speaker.fill")
                        .foregroundColor(Color.nghTextTertiary)
                    Slider(value: Binding(get: { Double(player.volume) },
                                          set: { player.setVolume(Float($0)) }),
                           in: 0...1).frame(width: 90).tint(Color.nghPrimary)
                }
                .foregroundColor(Color.nghText)
            }
            .padding(.horizontal, NghSpacing.s4)
            .padding(.vertical, NghSpacing.s2)
        }
        .background(
            RoundedRectangle(cornerRadius: NghRadius.lg, style: .continuous).fill(Color.nghSurface)
        )
        .clipShape(RoundedRectangle(cornerRadius: NghRadius.lg, style: .continuous))
        .nghCardShadow()
        .padding(.horizontal, NghSpacing.s3)
        .padding(.bottom, NghSpacing.s2)
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
        case .playlist: return "music.note.list"
        case .favorites: return "heart"
        case .leaderboard: return "trophy"
        case .local: return "folder"
        case .settings: return "gearshape"
        }
    }
}

#Preview {
    ContentView().environmentObject(PlayerManager())
}
