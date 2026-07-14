// MARK: - PlaylistView
// 播放列表页：展示当前播放队列与可选多 playlist 容器。
// Scaffold：展示 PlayerService.queue 当前内容；可清空、可定位到当前播放项。

import SwiftUI

struct PlaylistView: View {
    @ObservedObject var player: PlayerService
    @State private var errorMessage: String?

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            if player.queue.isEmpty {
                emptyView
            } else {
                queueList
            }
        }
        .navigationTitle("播放列表")
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var header: some View {
        HStack {
            Text("当前队列（\(player.queue.count) 首）")
                .font(.headline)
            Spacer()
            Button {
                player.loadQueue([], startIndex: 0)
            } label: {
                Label("清空", systemImage: "trash")
                    .foregroundColor(Color.nghDanger)
            }
            .disabled(player.queue.isEmpty)
        }
        .padding(.horizontal, NghSpacing.s4)
        .padding(.vertical, NghSpacing.s3)
    }

    private var queueList: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                ForEach(Array(player.queue.enumerated()), id: \.element.id) { index, song in
                    HStack(spacing: NghSpacing.s3) {
                        Image(systemName: index == player.currentIndex ? "play.circle.fill" : "music.note")
                            .foregroundColor(index == player.currentIndex ? Color.nghPrimary : Color.nghTextSecondary)
                            .frame(width: 24)
                        VStack(alignment: .leading, spacing: 2) {
                            Text(song.title).font(.body).lineLimit(1)
                                .foregroundColor(index == player.currentIndex ? Color.nghPrimary : Color.nghText)
                            Text(song.artistsDisplay).font(.caption).foregroundColor(Color.nghTextSecondary)
                        }
                        Spacer()
                        Text(song.durationDisplay).font(.caption).foregroundColor(Color.nghTextSecondary).monospacedDigit()
                    }
                    .padding(.horizontal, NghSpacing.s4)
                    .padding(.vertical, NghSpacing.s2)
                    .contentShape(Rectangle())
                    .onTapGesture { player.play(at: index) }
                    Divider().padding(.leading, 56)
                }
            }
        }
    }

    private var emptyView: some View {
        VStack(spacing: NghSpacing.s2) {
            Image(systemName: "music.note.list")
                .font(.system(size: 48))
                .foregroundColor(Color.nghTextTertiary)
            Text("播放队列为空")
                .font(.title3)
            Text("从搜索或排行榜中开始播放歌曲")
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

#Preview {
    PlaylistView(player: PlayerService())
        .frame(width: 800, height: 600)
}
