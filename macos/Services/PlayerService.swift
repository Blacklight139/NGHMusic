// MARK: - PlayerService
// 包装 AVPlayer（AVFoundation），对外暴露 play/pause/next/prev/seek/volume/mode。
// 播放状态持久化到 Application Support/NghMusic/player_state.json。
// 通过 ObservableObject + @Published 让 SwiftUI 视图自动更新。

import Foundation
import AVFoundation
import Combine
import os.log

/// 播放器服务，包装 AVPlayer 并持久化播放状态。
public final class PlayerService: ObservableObject {

    // MARK: - 公开状态（视图订阅）

    /// 当前播放的歌曲（nil 表示未加载）。
    @Published public private(set) var currentSong: Song?
    /// 是否正在播放。
    @Published public private(set) var isPlaying: Bool = false
    /// 当前播放位置（秒）。
    @Published public private(set) var currentTime: Double = 0
    /// 当前曲目总时长（秒）。
    @Published public private(set) var duration: Double = 0
    /// 音量 0...1。
    @Published public var volume: Float = 1.0 {
        didSet {
            player.volume = volume
            persistState()
        }
    }
    /// 播放模式。
    @Published public var mode: PlayMode = .sequential {
        didSet { persistState() }
    }

    /// 当前播放队列。
    @Published public private(set) var queue: [Song] = []
    /// 当前播放索引。
    @Published public private(set) var currentIndex: Int = -1

    // MARK: - 私有

    private let player = AVPlayer()
    private var timeObserverToken: Any?
    private var statusObserver: NSKeyValueObservation?
    private var endObserver: NSObjectProtocol?

    private let logger = Logger(subsystem: "com.nghmusic.macos", category: "PlayerService")

    /// 状态持久化目录：~/Library/Application Support/NghMusic/。
    private let stateDir: URL
    private let stateFile: URL

    /// 串行队列用于后台持久化，避免文件 I/O 阻塞主线程。
    private let persistQueue = DispatchQueue(label: "com.nghmusic.macos.persist", qos: .utility)
    /// 时间观察者持久化的节流时间戳，避免每 0.5s 都写盘。
    private var lastPersistTime: Date = .distantPast

    // MARK: - 初始化

    public init() {
        let fm = FileManager.default
        let appSupport = fm.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? URL(fileURLWithPath: NSTemporaryDirectory())
        let dir = appSupport.appendingPathComponent("NghMusic", isDirectory: true)
        try? fm.createDirectory(at: dir, withIntermediateDirectories: true)
        self.stateDir = dir
        self.stateFile = dir.appendingPathComponent("player_state.json")

        // 配置 AVPlayer
        player.volume = volume
        player.allowsExternalPlayback = false
        player.actionAtItemEnd = .pause

        installTimeObserver()
        installItemEndObserver()
        restoreState()
    }

    deinit {
        if let token = timeObserverToken {
            player.removeTimeObserver(token)
        }
        if let endObserver = endObserver {
            NotificationCenter.default.removeObserver(endObserver)
        }
        statusObserver?.invalidate()
    }

    // MARK: - 播放控制

    /// 加载新的播放队列并从指定索引开始播放。
    /// - Parameters:
    ///   - songs: 队列歌曲列表
    ///   - startIndex: 起始索引（默认 0）
    public func loadQueue(_ songs: [Song], startIndex: Int = 0) {
        queue = songs
        guard !songs.isEmpty, startIndex >= 0, startIndex < songs.count else {
            currentIndex = -1
            currentSong = nil
            stop()
            return
        }
        play(at: startIndex)
    }

    /// 播放指定索引的歌曲（不替换队列）。
    public func play(at index: Int) {
        guard index >= 0, index < queue.count else { return }
        currentIndex = index
        let song = queue[index]
        currentSong = song
        load(song: song)
        player.play()
        isPlaying = true
        persistState()
    }

    /// 切换播放/暂停。
    public func togglePlayPause() {
        guard currentSong != nil else { return }
        if isPlaying {
            pause()
        } else {
            resume()
        }
    }

    /// 暂停。
    public func pause() {
        player.pause()
        isPlaying = false
        persistState()
    }

    /// 恢复播放。
    public func resume() {
        guard currentSong != nil else { return }
        player.play()
        isPlaying = true
        persistState()
    }

    /// 停止播放并清空当前项。
    public func stop() {
        player.pause()
        player.replaceCurrentItem(with: nil)
        isPlaying = false
        currentTime = 0
        duration = 0
        persistState()
    }

    /// 下一首（按 mode 选择）。
    public func next() {
        guard !queue.isEmpty else { return }
        switch mode {
        case .singleLoop:
            seek(to: 0)
            player.play()
            isPlaying = true
        case .random:
            let nextIdx = Int.random(in: 0..<queue.count)
            play(at: nextIdx)
        case .sequential:
            let nextIdx = currentIndex + 1
            if nextIdx >= queue.count {
                stop()
            } else {
                play(at: nextIdx)
            }
        }
    }

    /// 上一首。
    public func previous() {
        guard !queue.isEmpty else { return }
        let prev = max(0, currentIndex - 1)
        play(at: prev)
    }

    /// 跳转到指定秒。
    public func seek(to seconds: Double) {
        let time = CMTime(seconds: seconds, preferredTimescale: 600)
        player.seek(to: time)
        currentTime = seconds
    }

    /// 切换播放模式（循环切换）。
    public func toggleMode() {
        mode = mode.next
    }

    // MARK: - 私有实现

    private func load(song: Song) {
        guard let urlString = effectivePlayUrl(for: song), let url = URL(string: urlString) else {
            logger.error("无法解析歌曲可播放 URL: \(song.title, privacy: .public)")
            return
        }
        // 释放旧 item 的 KVO
        statusObserver?.invalidate()
        let item = AVPlayerItem(url: url)
        player.replaceCurrentItem(with: item)

        // 监听 duration / status
        statusObserver = item.observe(\.status, options: [.new, .initial]) { [weak self] observedItem, _ in
            guard let self = self else { return }
            DispatchQueue.main.async { [weak self] in
                // 确保观察的 item 仍是当前播放项，避免快速切歌时旧 item 的
                // 回调覆盖新 item 的 duration（竞态条件）。
                guard let self = self, self.player.currentItem === observedItem else { return }
                if let d = observedItem.duration.seconds, d.isFinite, !d.isNaN {
                    self.duration = d
                }
            }
        }
        duration = song.durationMs.map { Double($0) / 1000.0 } ?? 0
    }

    /// 解析歌曲的最终可播放 URL：
    /// - 优先 play_url（在线）
    /// - 其次 local_path（本地文件，需 file:// scheme 包装）
    private func effectivePlayUrl(for song: Song) -> String? {
        if let local = song.localPath, !local.isEmpty {
            // 本地文件路径
            return URL(fileURLWithPath: local).absoluteString
        }
        return song.playUrl
    }

    private func installTimeObserver() {
        let interval = CMTime(seconds: 0.5, preferredTimescale: 600)
        timeObserverToken = player.addPeriodicTimeObserver(
            forInterval: interval,
            queue: .main
        ) { [weak self] time in
            guard let self = self else { return }
            self.currentTime = time.seconds
            if let item = self.player.currentItem,
               let d = item.duration.seconds, d.isFinite, !d.isNaN, d > 0 {
                self.duration = d
            }
            // 持久化位置（节流：最多每 3 秒写一次盘，避免阻塞主线程）
            let now = Date()
            if now.timeIntervalSince(self.lastPersistTime) >= 3.0 {
                self.lastPersistTime = now
                self.persistState()
            }
        }
    }

    private func installItemEndObserver() {
        endObserver = NotificationCenter.default.addObserver(
            forName: .AVPlayerItemDidPlayToEndTime,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.next()
        }
    }

    // MARK: - 状态持久化

    private func persistState() {
        let state = PlayState(
            currentSongId: currentSong?.id,
            playlistId: nil,
            index: currentIndex >= 0 ? currentIndex : nil,
            positionMs: UInt64(max(0, currentTime) * 1000),
            durationMs: UInt64(max(0, duration) * 1000),
            isPlaying: isPlaying,
            volume: volume,
            mode: mode
        )
        // 文件 I/O 放到后台串行队列执行，避免阻塞主线程；
        // 串行队列保证多次写入按顺序执行，不会互相覆盖。
        persistQueue.async { [weak self] in
            guard let self = self else { return }
            do {
                let data = try JSONEncoder().encode(state)
                try data.write(to: self.stateFile, options: [.atomic])
            } catch {
                self.logger.warning("持久化播放状态失败: \(error.localizedDescription, privacy: .public)")
            }
        }
    }

    private func restoreState() {
        guard FileManager.default.fileExists(atPath: stateFile.path),
              let data = try? Data(contentsOf: stateFile),
              let state = try? JSONDecoder().decode(PlayState.self, from: data) else {
            return
        }
        volume = state.volume
        player.volume = state.volume
        mode = state.mode
        // currentSong / queue 在重新加载歌曲前不恢复，仅恢复音量与模式
        logger.info("已恢复播放状态: volume=\(state.volume, privacy: .public), mode=\(state.mode.rawValue, privacy: .public)")
    }

    // MARK: - 便利属性

    /// 当前播放进度（0...1），无时长时返回 0。
    public var progress: Double {
        guard duration > 0 else { return 0 }
        return min(1, max(0, currentTime / duration))
    }

    /// 当前时间文本（mm:ss）。
    public var currentTimeDisplay: String {
        formatTime(currentTime)
    }

    /// 总时长文本（mm:ss）。
    public var durationDisplay: String {
        formatTime(duration)
    }

    private func formatTime(_ seconds: Double) -> String {
        guard seconds.isFinite, seconds >= 0 else { return "--:--" }
        let total = Int(seconds)
        let m = total / 60
        let s = total % 60
        return String(format: "%02d:%02d", m, s)
    }

    /// Application Support 状态目录（供 SettingsView 显示）。
    public var applicationSupportDirectory: URL { stateDir }
}
