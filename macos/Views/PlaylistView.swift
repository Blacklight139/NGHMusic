// MARK: - PlaylistView
// 播放列表页：展示当前播放队列与可选多 playlist 容器。
// Scaffold：展示 PlayerService.queue 当前内容；可清空、可定位到当前播放项。

import SwiftUI

struct PlaylistView: View {
    @ObservedObject var player: PlayerService
    @State private var errorMessage: String?
    @State private var hoveredIndex: Int? = nil

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
                    let isCurrent = index == player.currentIndex
                    HStack(spacing: NghSpacing.s3) {
                        Image(systemName: isCurrent ? "play.circle.fill" : "music.note")
                            .foregroundColor(isCurrent ? Color.nghPrimary : Color.nghTextSecondary)
                            .font(.title3)
                            .frame(width: 40, height: 40)
                            .background(isCurrent ? Color.nghPrimarySoft : Color.clear)
                            .clipShape(RoundedRectangle(cornerRadius: NghRadius.sm))
                        VStack(alignment: .leading, spacing: NghSpacing.s1) {
                            Text(song.title).font(.body).lineLimit(1)
                                .foregroundColor(isCurrent ? Color.nghPrimary : Color.nghText)
                                .fontWeight(isCurrent ? .semibold : .regular)
                            Text(song.artistsDisplay).font(.caption).foregroundColor(Color.nghTextSecondary)
                        }
                        Spacer()
                        Text(song.durationDisplay).font(.caption).foregroundColor(Color.nghTextTertiary).monospacedDigit()
                    }
                    .padding(.horizontal, NghSpacing.s4)
                    .padding(.vertical, NghSpacing.s3)
                    .background(hoveredIndex == index ? Color.nghSurfaceAlt : Color.clear)
                    .contentShape(Rectangle())
                    .onHover { hoveredIndex = $0 ? index : nil }
                    .onTapGesture { player.play(at: index) }
                    Divider().padding(.leading, NghSpacing.s7)
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
