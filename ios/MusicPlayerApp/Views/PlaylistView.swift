// MARK: - PlaylistView
// 职责：播放列表页，展示当前播放队列（PlayerManager.queue）。
// 点击歌曲调用 PlayerManager.play(song:in:)；高亮当前播放曲目。
// 队列为空时引导用户去搜索添加。

import SwiftUI

struct PlaylistView: View {
    @EnvironmentObject var player: PlayerManager

    var body: some View {
        PageContainer(title: "播放列表", subtitle: "当前队列共 \(player.queue.count) 首") {
            if player.queue.isEmpty {
                EmptyState(text: "播放列表为空，去搜索添加歌曲吧")
            } else {
                // Linear 风格：spacing 0，每行自带底部分割线，当前曲目用主色文字高亮（无描边）。
                VStack(spacing: 0) {
                    ForEach(Array(player.queue.enumerated()), id: \.element.id) { index, song in
                        Button {
                            player.play(song: song, in: player.queue)
                        } label: {
                            SongRow(index: index + 1, song: song, isCurrent: isCurrent(song))
                        }
                        .nghPressableStyle()
                        .transition(.opacity.combined(with: .move(edge: .top)))
                    }
                    if !player.queue.isEmpty {
                        Button(role: .destructive) {
                            // IOS-004 修复：用 clear() 彻底移除 AVPlayer currentItem，
                            // 而非仅 pause()（pause 后 resume() 仍会继续播放旧音频）。
                            player.clear()
                        } label: {
                            Label("清空播放列表", systemImage: "trash")
                                .font(.subheadline)
                                .foregroundColor(Color.nghDanger)
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, NghSpacing.s3)
                        }
                        .padding(.top, NghSpacing.s4)
                    }
                }
                // iOS 15+：列表项出现时 staggered fade-in。
                .animation(.easeOut(duration: 0.3), value: player.queue.count)
            }
        }
    }

    /// 是否为当前正在播放的曲目。
    private func isCurrent(_ song: Song) -> Bool {
        player.currentSong?.id == song.id
    }
}
