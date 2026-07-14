// MARK: - LeaderboardView
// 排行榜页：选择音源 → 加载排行榜列表 → 点击排行榜展示榜单内歌曲。

import SwiftUI

struct LeaderboardView: View {
    @EnvironmentObject var player: PlayerService
    @State private var sources: [SourceInfo] = []
    @State private var selectedSourceId: String?
    @State private var leaderboards: [Leaderboard] = []
    @State private var selectedLeaderboardId: String?
    @State private var isLoading = false
    @State private var errorMessage: String?

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            if isLoading {
                loadingView
            } else if let error = errorMessage {
                errorView(error)
            } else {
                content
            }
        }
        .navigationTitle("排行榜")
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onAppear { loadSources() }
    }

    private var header: some View {
        HStack(spacing: 12) {
            Picker("音源", selection: Binding(
                get: { selectedSourceId ?? sources.first?.id ?? "" },
                set: { newId in
                    selectedSourceId = newId
                    loadLeaderboards()
                }
            )) {
                ForEach(sources) { src in
                    Text(src.name).tag(src.id)
                }
            }
            .pickerStyle(.menu)
            .frame(maxWidth: 240)
            Spacer()
            Button(action: loadLeaderboards) {
                Label("刷新", systemImage: "arrow.clockwise")
            }
            .disabled(selectedSourceId == nil)
        }
        .padding(.horizontal, NghSpacing.s4)
        .padding(.vertical, NghSpacing.s3)
    }

    @ViewBuilder
    private var content: some View {
        if leaderboards.isEmpty {
            emptyView
        } else {
            HSplitView {
                leaderboardList
                songsDetail
            }
        }
    }

    private var leaderboardList: some View {
        List(leaderboards, selection: Binding(
            get: { selectedLeaderboardId },
            set: { selectedLeaderboardId = $0 }
        )) { lb in
            HStack(spacing: 12) {
                Image(systemName: "chart.bar.xaxis")
                    .foregroundColor(Color.nghPrimary)
                Text(lb.name)
                Spacer()
                Text("\(lb.songs.count)")
                    .font(.caption)
                    .foregroundColor(Color.nghTextSecondary)
            }
            .tag(lb.id)
        }
        .frame(minWidth: 200)
    }

    @ViewBuilder
    private var songsDetail: some View {
        if let lb = leaderboards.first(where: { $0.id == selectedLeaderboardId }) {
            if lb.songs.isEmpty {
                VStack(spacing: 8) {
                    Image(systemName: "music.note.list")
                        .font(.system(size: 48))
                        .foregroundColor(Color.nghTextSecondary)
                    Text("该榜单暂无歌曲")
                        .font(.title3)
                }
                .frame(maxWidth: .infinity, maxHeight: .infinity)
            } else {
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 0) {
                        ForEach(Array(lb.songs.enumerated()), id: \.element.id) { idx, song in
                            HStack(spacing: 12) {
                                Text("\(idx + 1)")
                                    .font(.headline)
                                    .frame(width: 28, alignment: .center)
                                    .foregroundColor(idx < 3 ? Color.nghWarning : Color.nghTextSecondary)
                                SongRow(song: song)
                                    .contentShape(Rectangle())
                                    .onTapGesture {
                                        player.loadQueue(lb.songs, startIndex: idx)
                                    }
                            }
                            .padding(.horizontal, NghSpacing.s4)
                            .padding(.vertical, 4)
                            Divider().padding(.leading, 56)
                        }
                    }
                    .padding(.vertical, NghSpacing.s2)
                }
            }
        } else {
            VStack(spacing: 8) {
                Image(systemName: "chart.bar")
                    .font(.system(size: 48))
                    .foregroundColor(Color.nghTextSecondary)
                Text("选择左侧排行榜查看歌曲")
                    .font(.title3)
                    .foregroundColor(Color.nghTextSecondary)
            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        }
    }

    private var loadingView: some View {
        VStack(spacing: 12) {
            ProgressView()
            Text("加载中…")
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
            Text("加载失败")
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
            Image(systemName: "chart.bar.xaxis")
                .font(.system(size: 48))
                .foregroundColor(Color.nghTextSecondary)
            Text("暂无排行榜")
                .font(.title3)
            Text("请确保已启用音源且音源支持排行榜接口")
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - 行为

    private func loadSources() {
        Task {
            do {
                let list = try await CoreService.shared.sourceList()
                await MainActor.run {
                    self.sources = list.filter { $0.enabled }
                    if self.selectedSourceId == nil { self.selectedSourceId = self.sources.first?.id }
                    self.loadLeaderboards()
                }
            } catch {
                await MainActor.run {
                    self.errorMessage = (error as? MusicCoreError)?.description ?? error.localizedDescription
                }
            }
        }
    }

    private func loadLeaderboards() {
        guard let id = selectedSourceId else { return }
        isLoading = true
        errorMessage = nil
        Task {
            do {
                let lbs = try await CoreService.shared.getLeaderboards(sourceId: id)
                await MainActor.run {
                    self.leaderboards = lbs
                    self.selectedLeaderboardId = lbs.first?.id
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

#Preview {
    LeaderboardView()
        .environmentObject(PlayerService())
        .frame(width: 900, height: 600)
}
