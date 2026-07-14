// MARK: - LyricsView
// 职责：歌词页，逐行展示并高亮当前行，与 PlayerManager.position 同步滚动。
// 对齐 docs/sound-source-api.md：通过 MusicCoreBridge.getLyric 拉取歌词；
// 脚手架阶段（未链接核心库）抛 unavailable 时降级到占位歌词。
// 点击行跳转播放进度（player.seek(toMs:)）。

import SwiftUI

struct LyricsView: View {
    @EnvironmentObject var player: PlayerManager
    @State private var lyric: Lyric?
    @State private var currentIndex = 0
    @State private var errorMessage: String?

    /// 占位歌词（未链接核心库 / 获取失败时展示）。
    private let placeholderLyric: Lyric = Lyric(
        lines: [
            LyricLine(timeMs: 0, text: "示例歌词第一行"),
            LyricLine(timeMs: 5000, text: "示例歌词第二行"),
            LyricLine(timeMs: 10000, text: "示例歌词第三行"),
            LyricLine(timeMs: 15000, text: "示例歌词第四行"),
            LyricLine(timeMs: nil, text: "（无时间戳行）"),
        ],
        translation: nil
    )

    private var lines: [LyricLine] {
        (lyric ?? placeholderLyric).lines
    }

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(spacing: NghSpacing.s3) {
                    if let errorMessage = errorMessage {
                        Text(errorMessage)
                            .font(.caption)
                            .foregroundColor(Color.nghTextTertiary)
                            .padding(.bottom, NghSpacing.s2)
                    }
                    ForEach(Array(lines.enumerated()), id: \.offset) { index, line in
                        Text(line.text)
                            .font(.body)
                            .fontWeight(index == currentIndex ? .semibold : .regular)
                            .foregroundColor(index == currentIndex ? Color.nghPrimary : Color.nghTextTertiary)
                            .scaleEffect(index == currentIndex ? 1.04 : 1)
                            .animation(.easeInOut(duration: 0.18), value: currentIndex)
                            .id(index)
                            .onTapGesture {
                                if let t = line.timeMs { player.seek(toMs: t) }
                            }
                    }
                }
                .padding(.vertical, NghSpacing.s7)
                .frame(maxWidth: .infinity)
            }
            .background(Color.nghBackground)
            .onChange(of: player.position) { newValue in
                let ms = UInt64(newValue * 1000)
                let idx = lines.lastIndex { line in
                    (line.timeMs ?? 0) <= ms
                } ?? 0
                if idx != currentIndex {
                    currentIndex = idx
                    withAnimation { proxy.scrollTo(idx, anchor: .center) }
                }
            }
            .onChange(of: player.currentSong?.id) { _ in
                Task { await loadLyric() }
            }
        }
        .onAppear {
            Task { await loadLyric() }
        }
    }

    /// 拉取当前歌曲歌词；失败降级到占位歌词。
    private func loadLyric() async {
        guard let song = player.currentSong else {
            lyric = nil
            errorMessage = nil
            return
        }
        do {
            lyric = try await MusicCoreBridge.getLyric(sourceId: song.sourceId, songId: song.id)
            errorMessage = nil
        } catch {
            lyric = nil
            errorMessage = "（占位）\(error.localizedDescription)"
        }
    }
}
