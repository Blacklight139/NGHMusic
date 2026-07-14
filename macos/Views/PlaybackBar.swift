// MARK: - PlaybackBar
// 底部播放控制条：当前歌曲信息 + 播放/暂停 + 上/下一首 + 进度 + 音量 + 模式。
// 使用 SF Symbols（systemName），不使用 emoji。
// 监听 PlayerService 状态自动刷新。

import SwiftUI

struct PlaybackBar: View {
    @ObservedObject var player: PlayerService

    @State private var isDraggingSlider = false
    @State private var dragValue: Double = 0

    var body: some View {
        HStack(spacing: NghSpacing.s4) {
            currentSongInfo
            Divider().frame(height: 40)
            playbackControls
            Divider().frame(height: 40)
            progressSection
            Divider().frame(height: 40)
            HStack(spacing: NghSpacing.s3) {
                modeButton
                volumeSection
            }
        }
        .padding(.horizontal, NghSpacing.s4)
        .padding(.vertical, NghSpacing.s2)
        .background(Color.nghSurface)
        .overlay(
            Rectangle()
                .fill(Color.nghBorder)
                .frame(height: 1),
            alignment: .top
        )
    }

    // MARK: - 子视图

    /// 左侧：当前歌曲封面占位 + 标题 + 艺术家。
    private var currentSongInfo: some View {
        HStack(spacing: NghSpacing.s3) {
            Image(systemName: "music.note")
                .font(.title2)
                .frame(width: 40, height: 40)
                .background(Color.nghPrimarySoft)
                .clipShape(RoundedRectangle(cornerRadius: NghRadius.sm))
                .foregroundColor(Color.nghPrimary)
            VStack(alignment: .leading, spacing: 2) {
                Text(player.currentSong?.title ?? "未在播放")
                    .font(.body)
                    .lineLimit(1)
                Text(player.currentSong?.artistsDisplay ?? "—")
                    .font(.caption)
                    .foregroundColor(Color.nghTextSecondary)
                    .lineLimit(1)
            }
            .frame(minWidth: 160, idealWidth: 200, maxWidth: 240, alignment: .leading)
        }
    }

    /// 中间：上一首 / 播放暂停 / 下一首。
    private var playbackControls: some View {
        HStack(spacing: NghSpacing.s5) {
            Button(action: player.previous) {
                Image(systemName: "backward.fill")
                    .font(.title3)
            }
            .buttonStyle(.plain)
            .disabled(player.currentIndex < 0)

            Button(action: player.togglePlayPause) {
                Image(systemName: player.isPlaying ? "pause.circle.fill" : "play.circle.fill")
                    .font(.system(size: 30))
                    .foregroundColor(Color.nghPrimary)
            }
            .buttonStyle(.plain)
            .disabled(player.currentSong == nil)

            Button(action: player.next) {
                Image(systemName: "forward.fill")
                    .font(.title3)
            }
            .buttonStyle(.plain)
            .disabled(player.queue.isEmpty)
        }
    }

    /// 进度条 + 时间显示。
    private var progressSection: some View {
        HStack(spacing: NghSpacing.s2) {
            Text(player.currentTimeDisplay)
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
                .monospacedDigit()
                .frame(width: 44, alignment: .trailing)
            Slider(value: Binding(
                get: { isDraggingSlider ? dragValue : player.currentTime },
                set: { newValue in
                    isDraggingSlider = true
                    dragValue = newValue
                }
            ), in: 0...max(player.duration, 1), onEditingChanged: { editing in
                if !editing {
                    player.seek(to: dragValue)
                    isDraggingSlider = false
                }
            })
            .disabled(player.duration <= 0)
            Text(player.durationDisplay)
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
                .monospacedDigit()
                .frame(width: 44, alignment: .leading)
        }
        .frame(maxWidth: .infinity)
    }

    /// 播放模式按钮（点击切换）。
    private var modeButton: some View {
        Button(action: player.toggleMode) {
            Image(systemName: player.mode.symbolName)
                .font(.title3)
                .help(player.mode.displayName)
        }
        .buttonStyle(.plain)
    }

    /// 音量滑块。
    private var volumeSection: some View {
        HStack(spacing: NghSpacing.s2) {
            Image(systemName: volumeIcon(for: player.volume))
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
            Slider(value: $player.volume, in: 0...1)
                .frame(width: 80)
        }
    }

    /// 根据音量值选择合适的 SF Symbol。
    private func volumeIcon(for value: Float) -> String {
        if value <= 0 { return "speaker.slash.fill" }
        if value < 0.34 { return "speaker.wave.1.fill" }
        if value < 0.67 { return "speaker.wave.2.fill" }
        return "speaker.wave.3.fill"
    }
}

#Preview {
    PlaybackBar(player: PlayerService())
        .frame(width: 900, height: 80)
}
