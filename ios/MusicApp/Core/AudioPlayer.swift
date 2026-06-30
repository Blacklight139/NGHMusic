import Foundation
import AVFoundation
import Combine
import SwiftUI

/// 基于 AVPlayer 的原生音频播放组件。
///
/// 职责：
/// - 播放/暂停、上一首/下一首、进度拖动、音量、播放模式（顺序/单曲循环/随机）。
/// - 通过 `addPeriodicTimeObserver` 监听播放进度，并监听 `AVPlayerItemDidPlayToEndTime` 自动推进。
/// - 处理 AVPlayer 缓冲状态与错误（currentItem.status / timeControlStatus）。
/// - 播放状态持久化：队列、当前索引、进度、音量、模式，重启后恢复（不自动续播）。
///
/// 本类为非 actor 隔离的 ObservableObject；所有 @Published 变更均发生在主线程
/// （观察者回调使用 .main 队列，UI 交互亦在主线程），以兼容 Swift 5/6 默认语言模式。
final class AudioPlayer: NSObject, ObservableObject {

    // MARK: - 发布状态

    /// 当前播放歌曲。
    @Published private(set) var currentSong: Song?
    /// 当前播放队列（now playing list）。
    @Published private(set) var queue: [Song] = []
    /// 当前播放索引（-1 表示空）。
    @Published private(set) var currentIndex: Int = -1

    /// 是否正在播放。
    @Published private(set) var isPlaying: Bool = false
    /// 当前播放位置（秒）。
    @Published private(set) var currentTime: Double = 0
    /// 总时长（秒）。
    @Published private(set) var duration: Double = 0
    /// 是否正在缓冲。
    @Published private(set) var isBuffering: Bool = false
    /// 最近一次错误信息（nil 表示无错误）。
    @Published private(set) var errorMessage: String?

    /// 音量 0...1。
    @Published var volume: Float = 0.8 {
        didSet {
            let clamped = max(0, min(1, volume))
            player.volume = clamped
            persist()
        }
    }

    /// 播放模式。
    @Published var playMode: PlayMode = .sequence {
        didSet { persist() }
    }

    // MARK: - 私有

    private let player = AVPlayer()
    private var timeObserver: Any?
    private var endObserver: NSObjectProtocol?
    private var itemStatusObservation: NSKeyValueObservation?
    private var bufferingObservation: NSKeyValueObservation?
    private var timeControlObservation: NSKeyValueObservation?

    private let stateKey = "music.player.persistedState"

    // MARK: - 生命周期

    override init() {
        super.init()
        configureAudioSession()
        setupObservers()
        player.volume = volume
        restoreState()
    }

    deinit {
        if let t = timeObserver { player.removeTimeObserver(t) }
        if let o = endObserver { NotificationCenter.default.removeObserver(o) }
        itemStatusObservation?.invalidate()
        bufferingObservation?.invalidate()
        timeControlObservation?.invalidate()
    }

    /// 配置 AVAudioSession（后台播放需在 Info.plist 开启 background mode: audio）。
    private func configureAudioSession() {
        let session = AVAudioSession.sharedInstance()
        do {
            try session.setCategory(.playback, mode: .default, options: [])
            try session.setActive(true, options: [])
        } catch {
            // 会话配置失败不致命，仅记录
            errorMessage = "音频会话配置失败：\(error.localizedDescription)"
        }
    }

    // MARK: - 观察者

    private func setupObservers() {
        // 进度（0.2s 节流，主线程回调）
        let interval = CMTime(seconds: 0.2, preferredTimescale: 600)
        timeObserver = player.addPeriodicTimeObserver(forInterval: interval, queue: .main) { [weak self] time in
            guard let self else { return }
            let t = time.seconds
            if t.isFinite && !t.isNaN { self.currentTime = max(0, t) }
            if let d = self.player.currentItem?.duration.seconds, d.isFinite, !d.isNaN, d > 0 {
                self.duration = d
            }
            // 持久化进度（节流：每 2 秒写一次）
            if Int(self.currentTime) % 2 == 0 { self.persist() }
        }

        // 播放结束
        endObserver = NotificationCenter.default.addObserver(
            forName: .AVPlayerItemDidPlayToEndTime,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            DispatchQueue.main.async { self?.handleDidEnd() }
        }

        // 播放控制状态 -> isPlaying / isBuffering
        timeControlObservation = player.observe(\.timeControlStatus, options: [.new]) { [weak self] player, _ in
            DispatchQueue.main.async {
                guard let self else { return }
                switch player.timeControlStatus {
                case .playing:
                    self.isPlaying = true
                    self.isBuffering = false
                case .paused:
                    self.isPlaying = false
                case .waitingToPlayAtSpecifiedRate:
                    self.isBuffering = true
                @unknown default:
                    break
                }
            }
        }
    }

    /// 为当前 AVPlayerItem 安装状态/缓冲观察。
    private func observeCurrentItem(_ item: AVPlayerItem) {
        itemStatusObservation = item.observe(\.status, options: [.new]) { [weak self] item, _ in
            DispatchQueue.main.async {
                guard let self else { return }
                switch item.status {
                case .readyToPlay:
                    self.errorMessage = nil
                case .failed:
                    self.errorMessage = item.error?.localizedDescription ?? "播放失败"
                    self.isPlaying = false
                default:
                    break
                }
            }
        }
        bufferingObservation = item.observe(\.isPlaybackLikelyToKeepUp, options: [.new]) { [weak self] item, _ in
            DispatchQueue.main.async {
                guard let self else { return }
                self.isBuffering = !item.isPlaybackLikelyToKeepUp
            }
        }
    }

    // MARK: - 播放控制

    /// 以指定队列与起始索引开始播放。
    func play(_ song: Song, in queue: [Song] = []) {
        let resolvedQueue = queue.isEmpty ? [song] : queue
        guard let index = resolvedQueue.firstIndex(where: { $0.id == song.id }) else {
            // 不在队列中：以单曲为队列
            self.queue = [song]
            self.currentIndex = 0
            load(song: song, autoplay: true)
            return
        }
        self.queue = resolvedQueue
        self.currentIndex = index
        load(song: song, autoplay: true)
    }

    /// 加载并（可选）播放某歌曲。
    private func load(song: Song, autoplay: Bool) {
        currentSong = song
        currentTime = 0
        duration = song.duration ?? 0
        errorMessage = nil

        guard let urlString = song.playUrl, let url = URL(string: urlString) else {
            errorMessage = "该歌曲无可用播放地址"
            isPlaying = false
            return
        }

        let item = AVPlayerItem(url: url)
        observeCurrentItem(item)
        player.replaceCurrentItem(with: item)

        if autoplay {
            player.play()
        }
        persist()
    }

    /// 播放/暂停切换。
    func togglePlayPause() {
        guard currentSong != nil else {
            // 无当前歌曲：尝试从队列头部播放
            if let first = queue.first { play(first, in: queue) }
            return
        }
        if isPlaying {
            player.pause()
        } else {
            player.play()
        }
    }

    /// 下一首（依播放模式）。
    func next() {
        guard !queue.isEmpty else { return }
        switch playMode {
        case .shuffle:
            currentIndex = randomIndex(excluding: currentIndex)
        case .repeatOne, .sequence:
            currentIndex = (currentIndex + 1) % queue.count
        }
        load(song: queue[currentIndex], autoplay: true)
    }

    /// 上一首。
    func previous() {
        guard !queue.isEmpty else { return }
        // 若已播放超过 3 秒，回到当前歌曲开头
        if currentTime > 3 {
            seek(to: 0)
            return
        }
        if playMode == .shuffle {
            currentIndex = randomIndex(excluding: currentIndex)
        } else {
            currentIndex = (currentIndex - 1 + queue.count) % queue.count
        }
        load(song: queue[currentIndex], autoplay: true)
    }

    /// 跳转到指定秒数。
    func seek(to seconds: Double) {
        let target = max(0, min(seconds, duration > 0 ? duration : seconds))
        let time = CMTime(seconds: target, preferredTimescale: 600)
        player.seek(to: time, toleranceBefore: .positiveInfinity, toleranceAfter: .positiveInfinity) { [weak self] _ in
            DispatchQueue.main.async {
                self?.currentTime = target
            }
        }
    }

    /// 处理播放结束（依播放模式）。
    private func handleDidEnd() {
        guard !queue.isEmpty else { return }
        switch playMode {
        case .repeatOne:
            seek(to: 0)
            player.play()
        case .shuffle:
            currentIndex = randomIndex(excluding: currentIndex)
            load(song: queue[currentIndex], autoplay: true)
        case .sequence:
            if currentIndex < queue.count - 1 {
                currentIndex += 1
                load(song: queue[currentIndex], autoplay: true)
            } else {
                // 顺序播放到队列末尾：回到第一首并暂停
                currentIndex = 0
                load(song: queue[0], autoplay: false)
                seek(to: 0)
            }
        }
    }

    /// 在队列中取一个不同于当前索引的随机索引。
    private func randomIndex(excluding current: Int) -> Int {
        guard queue.count > 1 else { return max(0, current) }
        var idx = current
        while idx == current { idx = Int.random(in: 0..<queue.count) }
        return idx
    }

    // MARK: - 队列管理（供播放列表页使用）

    /// 替换整个队列并定位到指定歌曲。
    func setQueue(_ songs: [Song], startAt song: Song? = nil) {
        queue = songs
        if let song = song, let idx = songs.firstIndex(where: { $0.id == song.id }) {
            currentIndex = idx
            load(song: song, autoplay: true)
        } else {
            currentIndex = -1
        }
        persist()
    }

    /// 从队列移除指定索引歌曲。
    func remove(at index: Int) {
        guard queue.indices.contains(index) else { return }
        let wasCurrent = index == currentIndex
        queue.remove(at: index)
        // 修正索引
        if queue.isEmpty {
            currentIndex = -1
            currentSong = nil
            player.replaceCurrentItem(with: nil)
            isPlaying = false
        } else if wasCurrent {
            let clamped = min(index, queue.count - 1)
            currentIndex = clamped
            load(song: queue[clamped], autoplay: isPlaying)
        } else if index < currentIndex {
            currentIndex -= 1
        }
        persist()
    }

    /// 移动队列项（拖动排序）。
    func move(from source: IndexSet, to destination: Int) {
        let oldCurrent = currentIndex
        queue.move(fromOffsets: source, toOffset: destination)
        // 重新定位当前歌曲
        if let current = currentSong, let idx = queue.firstIndex(where: { $0.id == current.id }) {
            currentIndex = idx
        } else if oldCurrent >= 0, queue.indices.contains(oldCurrent) {
            currentIndex = oldCurrent
        } else {
            currentIndex = -1
        }
        persist()
    }

    /// 清空队列。
    func clearQueue() {
        queue = []
        currentIndex = -1
        currentSong = nil
        player.replaceCurrentItem(with: nil)
        isPlaying = false
        currentTime = 0
        duration = 0
        persist()
    }

    // MARK: - 持久化

    /// 持久化快照（与 PlayerState 模型对齐 + 队列与索引，用于恢复）。
    private struct PersistedState: Codable {
        var queue: [Song]
        var currentIndex: Int
        var positionSec: Double
        var volume: Float
        var mode: PlayMode
    }

    private func persist() {
        let state = PersistedState(
            queue: queue,
            currentIndex: currentIndex,
            positionSec: currentTime,
            volume: volume,
            mode: playMode
        )
        if let data = try? JSONEncoder().encode(state) {
            UserDefaults.standard.set(data, forKey: stateKey)
        }
    }

    private func restoreState() {
        guard let data = UserDefaults.standard.data(forKey: stateKey),
              let state = try? JSONDecoder().decode(PersistedState.self, from: data) else {
            return
        }
        volume = max(0, min(1, state.volume))
        playMode = state.mode
        queue = state.queue
        currentIndex = state.currentIndex
        // 仅恢复歌曲与位置，不自动续播（避免冷启动意外出声）
        if queue.indices.contains(state.currentIndex) {
            let song = queue[state.currentIndex]
            currentSong = song
            duration = song.duration ?? 0
            if let urlString = song.playUrl, let url = URL(string: urlString) {
                let item = AVPlayerItem(url: url)
                observeCurrentItem(item)
                player.replaceCurrentItem(with: item)
                // 恢复进度：等 item 可定位后 seek
                let target = CMTime(seconds: state.positionSec, preferredTimescale: 600)
                item.seek(to: target, toleranceBefore: .positiveInfinity, toleranceAfter: .positiveInfinity, completionHandler: { [weak self] _ in
                    DispatchQueue.main.async { self?.currentTime = state.positionSec }
                })
            }
        }
    }

    /// 暴露与 Rust `PlayerState` 模型对齐的快照（供外部读取/导出）。
    var playerState: PlayerState {
        PlayerState(
            current: currentSong?.songRef,
            playlistId: nil,
            positionSec: currentTime,
            volume: volume,
            mode: playMode,
            playing: isPlaying
        )
    }
}

// MARK: - 进度便捷计算

extension AudioPlayer {
    /// 播放进度 0...1。
    var progress: Double {
        guard duration > 0 else { return 0 }
        return min(1, max(0, currentTime / duration))
    }

    /// 当前时间格式化 mm:ss。
    var formattedCurrentTime: String {
        let total = Int(currentTime.rounded())
        return String(format: "%d:%02d", total / 60, total % 60)
    }

    /// 总时长格式化 mm:ss。
    var formattedDuration: String {
        let total = Int(duration.rounded())
        return String(format: "%d:%02d", total / 60, total % 60)
    }
}
