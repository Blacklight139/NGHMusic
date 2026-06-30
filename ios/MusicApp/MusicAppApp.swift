import SwiftUI

/// 应用入口。跨平台音乐播放器 iOS 端（SwiftUI + AVPlayer + 共享 Rust 核心）。
@main
struct MusicAppApp: App {
    // 全局共享状态对象
    @StateObject private var audioPlayer = AudioPlayer()
    @StateObject private var sourcesStore = SourcesStore()
    @StateObject private var playlistStore = PlaylistStore()
    @StateObject private var favoritesStore = FavoritesStore()

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(audioPlayer)
                .environmentObject(sourcesStore)
                .environmentObject(playlistStore)
                .environmentObject(favoritesStore)
                .appBackground()
        }
    }
}

/// 根视图：TabView 五个 Tab + 悬浮迷你播放器 + 全屏播放页。
struct RootView: View {
    @EnvironmentObject private var audioPlayer: AudioPlayer
    @State private var selectedTab = 0
    @State private var showPlayer = false

    var body: some View {
        ZStack(alignment: .bottom) {
            TabView(selection: $selectedTab) {
                // 首页：排行榜 / 推荐
                NavigationStack {
                    RankingView()
                }
                .tabItem { Label("首页", systemImage: "music.note.house") }
                .tag(0)

                // 搜索
                NavigationStack {
                    SearchView()
                }
                .tabItem { Label("搜索", systemImage: "magnifyingglass") }
                .tag(1)

                // 播放列表
                NavigationStack {
                    PlaylistView()
                }
                .tabItem { Label("播放列表", systemImage: "play.square.stack") }
                .tag(2)

                // 收藏夹
                NavigationStack {
                    FavoritesView()
                }
                .tabItem { Label("收藏", systemImage: "heart") }
                .tag(3)

                // 设置（音源导入）
                NavigationStack {
                    SettingsView()
                }
                .tabItem { Label("设置", systemImage: "gearshape") }
                .tag(4)
            }
            .tint(Theme.Palette.primary)

            // 迷你播放器悬浮于 TabBar 之上
            if audioPlayer.currentSong != nil {
                MiniPlayerView()
                    .padding(.horizontal, Theme.Spacing.s)
                    .padding(.bottom, 2)
                    .onTapGesture { showPlayer = true }
                    .transition(.move(edge: .bottom).combined(with: .opacity))
                    .zIndex(1)
            }
        }
        .animation(.easeInOut(duration: 0.2), value: audioPlayer.currentSong != nil)
        .fullScreenCover(isPresented: $showPlayer) {
            PlayerView()
                .environmentObject(audioPlayer)
        }
    }
}

// MARK: - 迷你播放器

/// 悬浮于 TabBar 之上的迷你播放条：封面、标题/艺术家、播放/下一首、底部进度。
struct MiniPlayerView: View {
    @EnvironmentObject private var audioPlayer: AudioPlayer

    var body: some View {
        HStack(spacing: Theme.Spacing.m) {
            CoverArt(urlString: audioPlayer.currentSong?.coverUrl, size: 44)

            VStack(alignment: .leading, spacing: 2) {
                Text(audioPlayer.currentSong?.title ?? "")
                    .font(Theme.Typography.bodyEmphasized)
                    .foregroundStyle(Theme.Palette.textPrimary)
                    .lineLimit(1)
                Text(audioPlayer.currentSong?.artist ?? "")
                    .font(Theme.Typography.small)
                    .foregroundStyle(Theme.Palette.textSecondary)
                    .lineLimit(1)
            }

            Spacer()

            Button {
                audioPlayer.togglePlayPause()
            } label: {
                Image(systemName: audioPlayer.isPlaying ? "pause.fill" : "play.fill")
                    .font(.title3)
                    .foregroundStyle(Theme.Palette.textPrimary)
                    .frame(width: 36, height: 36)
            }
            .buttonStyle(.plain)

            Button {
                audioPlayer.next()
            } label: {
                Image(systemName: "forward.fill")
                    .font(.title3)
                    .foregroundStyle(Theme.Palette.textPrimary)
                    .frame(width: 36, height: 36)
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, Theme.Spacing.m)
        .padding(.vertical, Theme.Spacing.s)
        .background(Theme.Palette.surface)
        .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.l, style: .continuous))
        .overlay(alignment: .bottom) {
            GeometryReader { geo in
                Rectangle()
                    .fill(Theme.Palette.primary)
                    .frame(width: geo.size.width * audioPlayer.progress, height: 2)
            }
            .frame(height: 2)
        }
        .shadow(color: .black.opacity(0.35), radius: 10, y: 3)
    }
}

// MARK: - 共享组件

/// 封面图（带占位与加载态）。全 App 复用，保证视觉统一。
struct CoverArt: View {
    var urlString: String?
    var size: CGFloat = 56

    var body: some View {
        Group {
            if let urlString, let url = URL(string: urlString) {
                AsyncImage(url: url) { phase in
                    switch phase {
                    case .empty:
                        placeholder
                    case .success(let image):
                        image.resizable().scaledToFill()
                    case .failure:
                        placeholder
                    @unknown default:
                        placeholder
                    }
                }
            } else {
                placeholder
            }
        }
        .frame(width: size, height: size)
        .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.cover, style: .continuous))
        .background(Theme.Palette.surface)
    }

    private var placeholder: some View {
        RoundedRectangle(cornerRadius: Theme.Radius.cover, style: .continuous)
            .fill(Theme.Palette.surface)
            .overlay(
                Image(systemName: "music.note")
                    .font(.system(size: size * 0.38))
                    .foregroundStyle(Theme.Palette.textTertiary)
            )
    }
}

/// 圆形按钮（播放控制等）。
struct IconButton: View {
    let systemName: String
    var size: CGFloat = 24
    var action: () -> Void

    var body: some View {
        Button(action: action) {
            Image(systemName: systemName)
                .font(.system(size: size, weight: .semibold))
                .foregroundStyle(Theme.Palette.textPrimary)
                .frame(minWidth: 44, minHeight: 44)
        }
        .buttonStyle(.plain)
    }
}

/// 音源来源徽章：用于在搜索结果等列表中标识数据来源音源。
struct SourceBadge: View {
    let name: String

    var body: some View {
        Text(name)
            .font(Theme.Typography.small)
            .foregroundStyle(Theme.Palette.primaryActive)
            .padding(.horizontal, Theme.Spacing.s)
            .padding(.vertical, 2)
            .background(Theme.Palette.primaryDim)
            .clipShape(Capsule())
    }
}

/// 空状态占位视图。
struct EmptyStateView: View {
    var systemImage: String = "tray"
    var title: String
    var message: String?

    var body: some View {
        VStack(spacing: Theme.Spacing.m) {
            Image(systemName: systemImage)
                .font(.system(size: 40))
                .foregroundStyle(Theme.Palette.textTertiary)
            Text(title)
                .font(Theme.Typography.bodyEmphasized)
                .foregroundStyle(Theme.Palette.textSecondary)
            if let message {
                Text(message)
                    .font(Theme.Typography.caption)
                    .foregroundStyle(Theme.Palette.textTertiary)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal, Theme.Spacing.xl)
            }
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
