// MARK: - LyricsView
// 歌词页：展示当前播放歌曲的同步歌词（高亮当前行）。
// 通过 CoreService.shared 获取歌词，监听 PlayerService.currentTime 高亮对应行。

import SwiftUI

struct LyricsView: View {
    @ObservedObject var player: PlayerService
    @State private var lyric: Lyric?
    @State private var errorMessage: String?
    @State private var isLoading = false
    /// 缓存的当前高亮行索引，在 onChange(of: currentTime) 中更新。
    /// 避免渲染时每行都调用 activeLineIndex（O(n)）导致整体 O(n²)。
    @State private var cachedActiveLineIndex: Int?

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
                .foregroundColor(Color.nghPrimary)
            VStack(alignment: .leading, spacing: 2) {
                Text(player.currentSong?.title ?? "未在播放")
                    .font(.headline)
                Text(player.currentSong?.artistsDisplay ?? "—")
                    .font(.caption)
                    .foregroundColor(Color.nghTextSecondary)
            }
            Spacer()
            Button(action: loadLyric) {
                Label("刷新", systemImage: "arrow.clockwise")
            }
        }
        .padding(.horizontal, NghSpacing.s4)
        .padding(.vertical, NghSpacing.s3)
    }

    private func lyricsList(_ lyric: Lyric) -> some View {
        ScrollViewReader { proxy in
            ScrollView {
                LazyVStack(alignment: .leading, spacing: 8) {
                    ForEach(Array(lyric.lines.enumerated()), id: \.offset) { idx, line in
                        Text(line.text)
                            .font(isActiveLine(idx, line) ? .title3 : .body)
                            .fontWeight(isActiveLine(idx, line) ? .semibold : .regular)
                            .foregroundColor(isActiveLine(idx, line) ? Color.nghPrimary : Color.nghTextSecondary)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .padding(.vertical, 2)
                            .id(idx)
                    }
                }
                .padding(.horizontal, 24)
                .padding(.vertical, NghSpacing.s4)
            }
            .onChange(of: player.currentTime) { newTime in
                // 在此计算一次并缓存，渲染时直接比较，避免 O(n²)。
                let idx = activeLineIndex(lyric, timeMs: UInt64(newTime * 1000))
                cachedActiveLineIndex = idx
                if let idx = idx {
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
                .foregroundColor(Color.nghTextSecondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func errorView(_ message: String) -> some View {
        VStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle")
                .font(.largeTitle)
                .foregroundColor(Color.nghWarning)
            Text("歌词加载失败")
                .font(.headline)
            Text(message)
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 32)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var emptyView: some View {
        VStack(spacing: 8) {
            Image(systemName: "text.quote")
                .font(.system(size: 48))
                .foregroundColor(Color.nghTextSecondary)
            Text("暂无歌词")
                .font(.title3)
            Text("播放歌曲后将自动加载歌词")
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - 行为

    private func loadLyric() {
        guard let song = player.currentSong else {
            lyric = nil
            errorMessage = nil
            cachedActiveLineIndex = nil
            return
        }
        isLoading = true
        errorMessage = nil
        // 切歌时重置缓存，避免旧歌词的高亮索引错位应用到新歌词
        cachedActiveLineIndex = nil
        Task {
            do {
                let l = try await CoreService.shared.getLyric(sourceId: song.sourceId, songId: song.id)
                await MainActor.run {
                    self.lyric = l
                    // 立即根据当前播放时间初始化高亮行，避免首次渲染到下一次 currentTime tick 之间无高亮
                    self.cachedActiveLineIndex = self.activeLineIndex(l, timeMs: UInt64(self.player.currentTime * 1000))
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

    private func isActiveLine(_ idx: Int, _ line: LyricLine) -> Bool {
        guard line.timeMs != nil else { return false }
        return cachedActiveLineIndex == idx
    }
}

#Preview {
    LyricsView(player: PlayerService())
        .frame(width: 800, height: 600)
}
