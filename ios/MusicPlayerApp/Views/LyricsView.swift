// MARK: - LyricsView
// 职责：歌词页，逐行展示并高亮当前行，简约风格占位。
// 对齐桌面端 pages/lyrics.js：与 PlayerManager.position 同步滚动。

import SwiftUI

struct LyricsView: View {
    @EnvironmentObject var player: PlayerManager
    @State private var lines: [LyricLineUI] = [
        LyricLineUI(timeMs: 0, text: "示例歌词第一行"),
        LyricLineUI(timeMs: 5000, text: "示例歌词第二行"),
        LyricLineUI(timeMs: 10000, text: "示例歌词第三行"),
        LyricLineUI(timeMs: 15000, text: "示例歌词第四行"),
    ]
    @State private var currentIndex = 0

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView {
                VStack(spacing: NghSpacing.s3) {
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
                let idx = lines.lastIndex { line in
                    (line.timeMs ?? 0) <= newValue
                } ?? 0
                if idx != currentIndex {
                    currentIndex = idx
                    withAnimation { proxy.scrollTo(idx, anchor: .center) }
                }
            }
        }
    }
}

struct LyricLineUI {
    let timeMs: UInt64?
    let text: String
}
