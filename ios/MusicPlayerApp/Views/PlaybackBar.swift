// MARK: - PlaybackBar
// 职责：底部播放控制条。封面占位 + 标题/艺术家 + 上一首/播放/下一首 + 进度条 + 歌词/模式/音量。
// 与 PlayerManager（AVPlayer 封装）联动；图标统一 SF Symbols，无 emoji。
// 由 ContentView 在有曲目（player.currentSong != nil）时挂载于 TabView 下方。

import SwiftUI

struct PlaybackBar: View {
    @EnvironmentObject var player: PlayerManager
    @Binding var showLyrics: Bool

    var body: some View {
        VStack(spacing: 0) {
            ProgressView(value: Double(player.position), total: max(Double(player.duration), 1))
                .progressViewStyle(.linear)
                .tint(Color.nghPrimary)

            HStack(spacing: NghSpacing.s4) {
                // 左侧：封面占位 + 曲目信息
                RoundedRectangle(cornerRadius: NghRadius.sm, style: .continuous)
                    .fill(LinearGradient(colors: [Color.nghPrimary, Color.nghPrimaryHover],
                                         startPoint: .topLeading, endPoint: .bottomTrailing))
                    .overlay(
                        Image(systemName: "music.note")
                            .font(.system(size: 16, weight: .semibold))
                            .foregroundColor(.white)
                    )
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

                // 右侧：歌词 / 模式 / 音量
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
