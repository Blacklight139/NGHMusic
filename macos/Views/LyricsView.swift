// MARK: - LyricsView
// 歌词页：展示当前播放歌曲的同步歌词（高亮当前行）。
// 通过 CoreService.shared 获取歌词，监听 PlayerService.currentTime 高亮对应行。

import SwiftUI

struct LyricsView: View {
    @ObservedObject var player: PlayerService
    @State private var lyric: Lyric?
    @State private var errorMessage: String?
    @State private var isLoading = false

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            if isLoading {
                loadingView
            } else if let error = errorMessage {
                errorView(error)
            } else if let lyric = lyric, !lyric.lines.isEmpty {
                lyricsList(lyric)
            } else {
                emptyView
            }
        }
        .navigationTitle("歌词")
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onChange(of: player.currentSong?.id) { _ in
            loadLyric()
        }
        .onAppear { loadLyric() }
    }

    private var header: some View {
        HStack(spacing: 12) {
            Image(systemName: "music.note")
                .font(.title3)
                .foregroundColor(.accentColor)
            VStack(alignment: .leading, spacing: 2) {
                Text(player.currentSong?.title ?? "未在播放")
                    .font(.headline)
                Text(player.currentSong?.artistsDisplay ?? "—")
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            Spacer()
            Button(action: loadLyric) {
                Label("刷新", systemImage: "arrow.clockwise")
            }
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    private func lyricsList(_ lyric: Lyric) -> some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 8) {
                    ForEach(Array(lyric.lines.enumerated()), id: \.offset) { idx, line in
                        Text(line.text)
                            .font(isActiveLine(idx, line, lyric) ? .title3 : .body)
                            .fontWeight(isActiveLine(idx, line, lyric) ? .semibold : .regular)
                            .foregroundColor(isActiveLine(idx, line, lyric) ? .accentColor : .secondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.vertical, 2)
                            .id(idx)
                    }
                }
                .padding(.horizontal, 24)
                .padding(.vertical, 16)
            }
            .onChange(of: player.currentTime) { newTime in
                if let idx = activeLineIndex(lyric, timeMs: UInt64(newTime * 1000)) {
                    withAnimation { proxy.scrollTo(idx, anchor: .center) }
                }
            }
        }
    }

    private var loadingView: some View {
        VStack(spacing: 12) {
            ProgressView()
            Text("加载歌词…")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func errorView(_ message: String) -> some View {
        VStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle")
                .font(.largeTitle)
                .foregroundColor(.orange)
            Text("歌词加载失败")
                .font(.headline)
            Text(message)
                .font(.caption)
                .foregroundColor(.secondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 32)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var emptyView: some View {
        VStack(spacing: 8) {
            Image(systemName: "text.quote")
                .font(.system(size: 48))
                .foregroundColor(.secondary)
            Text("暂无歌词")
                .font(.title3)
            Text("播放歌曲后将自动加载歌词")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - 行为

    private func loadLyric() {
        guard let song = player.currentSong else {
            lyric = nil
            errorMessage = nil
            return
        }
        isLoading = true
        errorMessage = nil
        Task {
            do {
                let l = try await CoreService.shared.getLyric(sourceId: song.sourceId, songId: song.id)
                await MainActor.run {
                    self.lyric = l
                    self.isLoading = false
                }
            } catch {
                await MainActor.run {
                    self.errorMessage = (error as? MusicCoreError)?.description ?? error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }

    /// 当前播放时间对应的活跃行索引。
    private func activeLineIndex(_ lyric: Lyric, timeMs: UInt64) -> Int? {
        var idx: Int?
        for (i, line) in lyric.lines.enumerated() {
            guard let t = line.timeMs else { continue }
            if t <= timeMs { idx = i } else { break }
        }
        return idx
    }

    private func isActiveLine(_ idx: Int, _ line: LyricLine, _ lyric: Lyric) -> Bool {
        guard line.timeMs != nil else { return false }
        return activeLineIndex(lyric, timeMs: UInt64(player.currentTime * 1000)) == idx
    }
}

#Preview {
    LyricsView(player: PlayerService())
        .frame(width: 800, height: 600)
}
