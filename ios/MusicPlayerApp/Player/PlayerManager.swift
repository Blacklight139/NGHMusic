// MARK: - PlayerManager
// 职责：封装 AVPlayer（import AVFoundation），提供 play/pause/resume/seek/toNext/toPrev/
//       setVolume/setMode，通过 @Published 暴露 currentSong/queue/currentIndex/position/
//       duration/isPlaying/volume/mode，使用 AVPlayer 周期性时间回调监听进度。
//
// Bug 修复（见 docs/bug-report.md）：
// - IOS-001：play(song:in:) 当 playUrl 为 nil 或 URL 非法时，不再设置 currentSong / isPlaying，
//   避免出现「UI 显示正在播放但无音频」的状态不一致。本地歌曲回退使用 localPath 文件 URL。
// - IOS-002：PlayMode 已声明 CaseIterable（见 Models/Song.swift），toggleMode 可安全遍历。
// - IOS-003：时间观察者与 item status 观察均使用 [weak self]，避免强引用环导致 deinit 不触发；
//   deinit 中显式移除时间观察者与通知。
//
// 集成方式：在 Xcode 中需添加 AVFoundation framework；由 MusicPlayerApp 作为 @StateObject 注入环境。

import AVFoundation
import Combine
import Foundation

final class PlayerManager: NSObject, ObservableObject {
    @Published var currentSong: Song?
    @Published var queue: [Song] = []
    @Published var currentIndex: Int = 0
    @Published var position: TimeInterval = 0   // 秒
    @Published var duration: TimeInterval = 0   // 秒
    @Published var isPlaying: Bool = false
    // IOS-006 修复：volume 变更需同步到 AVPlayer，否则调节音量无效果。
    @Published var volume: Float = 0.8 {
        didSet { player.volume = volume }
    }
    @Published var mode: PlayMode = .sequential

    private let player = AVPlayer()
    private var timeObserver: Any?
    private var itemObserver: NSKeyValueObservation?

    override init() {
        super.init()
        // IOS-007 修复：配置 AVAudioSession 为 playback 类别并激活，
        // 否则静音模式下无音频、锁屏可能暂停、与其他 App 音频冲突。
        do {
            try AVAudioSession.sharedInstance().setCategory(.playback, mode: .default)
            try AVAudioSession.sharedInstance().setActive(true)
        } catch {
            // 配置失败仅记录，不阻断播放器初始化
            NSLog("AVAudioSession 配置失败：\(error)")
        }
        // 周期性进度回调（每 0.5 秒）；[weak self] 避免强引用环（IOS-003）
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

    /// 用指定 URL 播放（内部：构造 AVPlayerItem、监听 status、替换并播放）。
    func play(url: URL) {
        let item = AVPlayerItem(url: url)
        itemObserver = item.observe(\.status, options: [.new]) { [weak self] observedItem, _ in
            // [weak self] 避免强引用环（IOS-003）
            guard let self = self else { return }
            // IOS-008 修复：仅处理当前 currentItem 的回调，防止旧 item 的延迟
            // 回调覆盖新 item 的状态（快速切歌时旧观察者可能晚于新 item 触发）。
            guard self.player.currentItem === observedItem else { return }
            switch observedItem.status {
            case .readyToPlay:
                let d = CMTimeGetSeconds(observedItem.duration)
                if d.isFinite && !d.isNaN { self.duration = d }
            case .failed:
                // IOS-009 修复：加载失败时复位 isPlaying，避免「显示播放中但无音频」。
                self.isPlaying = false
            default:
                break
            }
        }
        player.replaceCurrentItem(with: item)
        player.volume = volume
        player.play()
        isPlaying = true
    }

    /// 播放队列中的某首歌。
    /// IOS-001 修复：先校验可播放 URL（playUrl 优先，回退 localPath 文件 URL），
    /// 仅在 URL 合法时才更新 currentSong / queue / currentIndex 并开始播放；
    /// 若无可播放 URL 则提前返回，不改变任何播放状态。
    func play(song: Song, in newQueue: [Song] = []) {
        guard let url = playableURL(for: song) else {
            // IOS-001：playUrl 为 nil 或 URL 非法 → 不设置 currentSong / isPlaying
            return
        }
        if !newQueue.isEmpty { queue = newQueue }
        if let idx = queue.firstIndex(where: { $0.id == song.id }) {
            currentIndex = idx
        } else if newQueue.isEmpty {
            // IOS-010 修复：newQueue 为空且 song 不在当前队列 → 单独播放该曲，
            // 避免 currentIndex 落在无关歌曲上导致 toNext/toPrev 跳错。
            queue = [song]
            currentIndex = 0
        } else {
            currentIndex = 0
        }
        currentSong = song
        play(url: url)
    }

    /// 解析歌曲的可播放 URL：优先 playUrl，回退 localPath 文件 URL；均无效返回 nil。
    private func playableURL(for song: Song) -> URL? {
        if let urlString = song.playUrl, !urlString.isEmpty, let url = URL(string: urlString) {
            return url
        }
        if let path = song.localPath, !path.isEmpty {
            return URL(fileURLWithPath: path)
        }
        return nil
    }

    func pause() { player.pause(); isPlaying = false }
    func resume() { player.play(); isPlaying = true }

    /// 清空播放队列并彻底停止播放（IOS-004 修复）。
    /// 仅 pause() 会保留 AVPlayer 内部 currentItem，resume() 仍会继续播放旧音频；
    /// 此处 replaceCurrentItem(with: nil) 彻底移除媒体，并复位所有 @Published 状态，
    /// 保证 UI「无播放」与音频实际停止一致。同时失效 item status 观察者。
    func clear() {
        player.replaceCurrentItem(with: nil)
        itemObserver?.invalidate()
        itemObserver = nil
        queue = []
        currentSong = nil
        currentIndex = 0
        position = 0
        duration = 0
        isPlaying = false
    }

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
            // IOS-011 修复：随机模式排除当前索引，避免重复播放同一首。
            if queue.count > 1 {
                var newIndex = Int.random(in: 0..<queue.count)
                while newIndex == currentIndex { newIndex = Int.random(in: 0..<queue.count) }
                currentIndex = newIndex
            }
            play(song: queue[currentIndex])
        case .sequential:
            currentIndex = (currentIndex + 1) % queue.count
            play(song: queue[currentIndex])
        }
    }

    func toPrev() {
        guard !queue.isEmpty else { return }
        currentIndex = (currentIndex - 1 + queue.count) % queue.count
        play(song: queue[currentIndex])
    }

    func setVolume(_ v: Float) {
        volume = v
        player.volume = v
    }

    func setMode(_ m: PlayMode) { mode = m }

    /// 循环切换播放模式（PlayMode 已 CaseIterable，IOS-002）。
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
