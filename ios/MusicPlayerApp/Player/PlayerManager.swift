// MARK: - PlayerManager
// 职责：封装 AVPlayer（import AVFoundation），提供 play/pause/resume/seek/toNext/toPrev/setVolume/setMode，
//       通过 @Published 暴露 currentSong/position/duration/isPlaying，使用 AVPlayerItem KVO 监听进度。
// 集成方式：在 Xcode 中需添加 AVFoundation framework；由 MusicPlayerApp 作为 @StateObject 注入环境。

import AVFoundation
import Combine
import Foundation

final class PlayerManager: NSObject, ObservableObject {
    @Published var currentSong: Song?
    @Published var position: TimeInterval = 0   // 秒
    @Published var duration: TimeInterval = 0   // 秒
    @Published var isPlaying: Bool = false
    @Published var volume: Float = 0.8
    @Published var mode: PlayMode = .sequential

    private let player = AVPlayer()
    private var queue: [Song] = []
    private var index: Int = 0
    private var timeObserver: Any?
    private var itemObserver: NSKeyValueObservation?

    override init() {
        super.init()
        // 周期性进度回调（每 0.5 秒）
        timeObserver = player.addPeriodicTimeObserver(
            forInterval: CMTime(seconds: 0.5, preferredTimescale: 600),
            queue: .main
        ) { [weak self] time in
            guard let self = self else { return }
            self.position = CMTimeGetSeconds(time)
            if let item = self.player.currentItem {
                let d = CMTimeGetSeconds(item.duration)
                if d.isFinite && !d.isNaN { self.duration = d }
            }
        }
        // 监听播放结束自动下一首
        NotificationCenter.default.addObserver(
            self, selector: #selector(onItemEnd),
            name: .AVPlayerItemDidPlayToEndTime, object: nil
        )
    }

    deinit {
        if let t = timeObserver { player.removeTimeObserver(t) }
        itemObserver?.invalidate()
        NotificationCenter.default.removeObserver(self)
    }

    // MARK: - 播放控制
    func play(url: URL) {
        let item = AVPlayerItem(url: url)
        itemObserver = item.observe(\.status, options: [.new]) { [weak self] item, _ in
            guard let self = self else { return }
            if item.status == .readyToPlay {
                let d = CMTimeGetSeconds(item.duration)
                if d.isFinite && !d.isNaN { self.duration = d }
            }
        }
        player.replaceCurrentItem(with: item)
        player.volume = volume
        player.play()
        isPlaying = true
    }

    /// 播放队列中的某首歌（解析 play_url 后调用 play(url:)）
    func play(song: Song, in queue: [Song] = []) {
        if !queue.isEmpty { self.queue = queue }
        currentSong = song
        index = self.queue.firstIndex(where: { $0.id == song.id }) ?? 0
        guard let urlString = song.playUrl, let url = URL(string: urlString) else { return }
        play(url: url)
    }

    func pause() { player.pause(); isPlaying = false }
    func resume() { player.play(); isPlaying = true }

    func seek(toMs ms: UInt64) {
        let target = CMTime(seconds: TimeInterval(ms) / 1000.0, preferredTimescale: 600)
        player.seek(to: target, toleranceBefore: .zero, toleranceAfter: .zero)
    }

    func toNext() {
        guard !queue.isEmpty else { return }
        switch mode {
        case .singleLoop:
            if let song = currentSong { play(song: song) }
        case .random:
            index = Int.random(in: 0..<queue.count)
            play(song: queue[index])
        case .sequential:
            index = (index + 1) % queue.count
            play(song: queue[index])
        }
    }

    func toPrev() {
        guard !queue.isEmpty else { return }
        index = (index - 1 + queue.count) % queue.count
        play(song: queue[index])
    }

    func setVolume(_ v: Float) {
        volume = v
        player.volume = v
    }

    func setMode(_ m: PlayMode) { mode = m }

    func toggleMode() {
        let all: [PlayMode] = PlayMode.allCases
        if let i = all.firstIndex(of: mode) {
            mode = all[(i + 1) % all.count]
        }
    }

    var modeIcon: String {
        switch mode {
        case .sequential: return "repeat"
        case .singleLoop: return "repeat.1"
        case .random: return "shuffle"
        }
    }

    @objc private func onItemEnd() { toNext() }
}
