// MARK: - SearchView
// 搜索页：搜索框 + 聚合搜索结果（歌曲/专辑/艺术家）。
// 通过 CoreService.shared 调用核心 FFI。

import SwiftUI

struct SearchView: View {
    @EnvironmentObject var player: PlayerService
    @State private var keyword: String = ""
    @State private var page: UInt32 = 1
    @State private var result: SearchResult?
    @State private var isLoading = false
    @State private var errorMessage: String?

    private let pageSize: UInt32 = 20

    var body: some View {
        VStack(spacing: 0) {
            searchHeader
            Divider()
            if let error = errorMessage {
                errorView(error)
            } else if isLoading {
                loadingView
            } else if let result = result, result.songs.isEmpty {
                emptyView
            } else if let result = result {
                resultsList(result)
            } else {
                placeholderView
            }
            Spacer(minLength: 0)
        }
        .navigationTitle("搜索")
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - 子视图

    private var searchHeader: some View {
        HStack(spacing: 8) {
            Image(systemName: "magnifyingglass")
                .foregroundColor(.secondary)
            TextField("搜索歌曲、专辑、艺术家", text: $keyword)
                .textFieldStyle(.plain)
                .onSubmit { performSearch(reset: true) }
            if !keyword.isEmpty {
                Button {
                    keyword = ""
                    result = nil
                    errorMessage = nil
                } label: {
                    Image(systemName: "xmark.circle.fill")
                        .foregroundColor(.secondary)
                }
                .buttonStyle(.plain)
            }
            Button {
                performSearch(reset: true)
            } label: {
                Text("搜索")
            }
            .disabled(keyword.trimmingCharacters(in: .whitespaces).isEmpty || isLoading)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    private func resultsList(_ result: SearchResult) -> some View {
        VStack(alignment: .leading, spacing: 16) {
            if !result.songs.isEmpty {
                section(title: "歌曲（\(result.songs.count)）") {
                    ForEach(result.songs) { song in
                        SongRow(song: song)
                            .contentShape(Rectangle())
                            .onTapGesture {
                                player.loadQueue(result.songs, startIndex: result.songs.firstIndex(where: { $0.id == song.id }) ?? 0)
                            }
                        Divider()
                    }
                }
            }
            if !result.albums.isEmpty {
                section(title: "专辑（\(result.albums.count)）") {
                    ForEach(result.albums) { album in
                        HStack(spacing: 12) {
                            Image(systemName: "music.note.list")
                                .font(.title3)
                                .foregroundColor(.accentColor)
                            VStack(alignment: .leading) {
                                Text(album.name).font(.body)
                                Text(album.artists.joined(separator: " / "))
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            Spacer()
                        }
                        .padding(.vertical, 6)
                        Divider()
                    }
                }
            }
            if !result.artists.isEmpty {
                section(title: "艺术家（\(result.artists.count)）") {
                    ForEach(result.artists) { artist in
                        HStack(spacing: 12) {
                            Image(systemName: "person.crop.circle")
                                .font(.title3)
                                .foregroundColor(.accentColor)
                            VStack(alignment: .leading) {
                                Text(artist.name).font(.body)
                                Text("共 \(artist.songIds.count) 首")
                                    .font(.caption)
                                    .foregroundColor(.secondary)
                            }
                            Spacer()
                        }
                        .padding(.vertical, 6)
                        Divider()
                    }
                }
            }
            if result.total > UInt64(result.songs.count) {
                HStack {
                    Spacer()
                    Text("共 \(result.total) 条结果，当前第 \(result.page) 页")
                        .font(.caption)
                        .foregroundColor(.secondary)
                    Spacer()
                }
                .padding(.bottom, 8)
            }
        }
        .padding(.horizontal, 16)
        .padding(.top, 8)
    }

    private func section<Content: View>(title: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(title)
                .font(.headline)
                .padding(.bottom, 4)
            content()
        }
    }

    private var loadingView: some View {
        VStack(spacing: 12) {
            ProgressView()
            Text("搜索中…")
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
            Text("搜索失败")
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
            Image(systemName: "magnifyingglass.circle")
                .font(.largeTitle)
                .foregroundColor(.secondary)
            Text("没有找到相关结果")
                .font(.body)
            Text("尝试使用其他关键词")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var placeholderView: some View {
        VStack(spacing: 8) {
            Image(systemName: "music.mic")
                .font(.system(size: 48))
                .foregroundColor(.secondary)
            Text("搜索您喜欢的音乐")
                .font(.title3)
            Text("支持跨音源聚合搜索")
                .font(.caption)
                .foregroundColor(.secondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - 行为

    private func performSearch(reset: Bool) {
        let trimmed = keyword.trimmingCharacters(in: .whitespaces)
        guard !trimmed.isEmpty else { return }
        if reset { page = 1 }
        isLoading = true
        errorMessage = nil
        Task {
            do {
                let r = try await CoreService.shared.search(keyword: trimmed, page: page, pageSize: pageSize)
                await MainActor.run {
                    self.result = r
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
}

// MARK: - SongRow

/// 通用歌曲行展示（封面占位 + 标题 + 艺术家 + 时长）。
struct SongRow: View {
    let song: Song

    var body: some View {
        HStack(spacing: 12) {
            Image(systemName: "music.note")
                .font(.title3)
                .frame(width: 32, height: 32)
                .background(Color.secondary.opacity(0.1))
                .clipShape(RoundedRectangle(cornerRadius: 6))
                .foregroundColor(.accentColor)
            VStack(alignment: .leading, spacing: 2) {
                Text(song.title).font(.body).lineLimit(1)
                Text(song.artistsDisplay).font(.caption).foregroundColor(.secondary).lineLimit(1)
            }
            Spacer()
            Text(song.durationDisplay)
                .font(.caption)
                .foregroundColor(.secondary)
                .monospacedDigit()
        }
        .padding(.vertical, 6)
    }
}

#Preview {
    SearchView()
        .environmentObject(PlayerService())
        .frame(width: 800, height: 600)
}
