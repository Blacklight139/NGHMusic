// 职责：封装 ExoPlayer/Media3，提供 setMediaItem/play/pause/seek/toNext/toPrev/setVolume/setRepeatMode，
//       通过 StateFlow 暴露播放状态（currentSong/position/duration/isPlaying/volume/mode）。
// 集成方式：作为 ViewModel 在 MainScreen 中注入；在 AndroidManifest 注册 MediaSessionService 实现后台播放。
//
// 已修复 bug-report.md 中的 Android 静态审查问题：
// - AND-001（竞态）：进度循环对 player 的访问与 onCleared 中的 release 通过 playerLock 同步；
//                   onCleared 先 cancel 进度协程再在锁内 release，杜绝 release 后读取 currentPosition。
// - AND-002（逻辑）：moveToNext/toPrev 改用内部 playAt(index) 直接播放目标索引，
//                   不再递归调用会重算 index 的 play(song)，避免随机/顺序切歌被重定位回原 index。
// - AND-003（资源）：attach 注册的 Player.Listener 以字段持有，onCleared 显式 removeListener。

package com.musicplayer.app.player

import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import com.musicplayer.app.models.PlayMode
import com.musicplayer.app.models.Song
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch

data class PlayerState(
    val currentSong: Song? = null,
    val queue: List<Song> = emptyList(),
    val isPlaying: Boolean = false,
    val position: Long = 0L,      // 毫秒
    val duration: Long = 0L,      // 毫秒
    val volume: Float = 0.8f,
    val mode: PlayMode = PlayMode.SEQUENTIAL
)

class PlayerManager : ViewModel() {
    // @Volatile 保证 onCleared 置空对进度循环可见；playerLock 序列化 release 与读取。
    @Volatile
    private var player: ExoPlayer? = null
    private var playerListener: Player.Listener? = null
    private var progressJob: Job? = null
    private val playerLock = Any()

    private var queue: List<Song> = emptyList()
    private var index: Int = 0

    private val _state = MutableStateFlow(PlayerState())
    val state: StateFlow<PlayerState> = _state.asStateFlow()

    /** 由 Composable 在创建 ExoPlayer 后注入（持有 Context） */
    fun attach(exoPlayer: ExoPlayer) {
        this.player = exoPlayer
        exoPlayer.volume = _state.value.volume
        // AND-003：以字段持有 Listener，便于 onCleared 显式移除。
        val listener = object : Player.Listener {
            override fun onIsPlayingChanged(isPlaying: Boolean) {
                _state.value = _state.value.copy(isPlaying = isPlaying)
            }
            override fun onPlaybackStateChanged(playbackState: Int) {
                if (playbackState == Player.STATE_READY) {
                    val d = exoPlayer.duration.takeIf { it > 0 } ?: 0L
                    _state.value = _state.value.copy(duration = d)
                }
            }
            override fun onMediaItemTransition(mediaItem: MediaItem?, reason: Int) {
                if (reason == Player.MEDIA_ITEM_TRANSITION_REASON_AUTO) {
                    // 自动播放下一首（不循环列表时由 ExoPlayer 触发）
                    moveToNext(auto = true)
                }
            }
        }
        playerListener = listener
        exoPlayer.addListener(listener)
        startProgressLoop()
    }

    private fun startProgressLoop() {
        // AND-001：进度协程以字段持有，便于 onCleared 取消；player 访问包在 playerLock 内。
        progressJob = viewModelScope.launch {
            while (isActive) {
                synchronized(playerLock) {
                    val p = player ?: return@launch
                    if (p.playbackState != Player.STATE_IDLE) {
                        val pos = p.currentPosition.coerceAtLeast(0)
                        _state.value = _state.value.copy(position = pos)
                    }
                }
                delay(500)
            }
        }
    }

    /**
     * 播放指定歌曲（用户主动点选入口）。
     * 当 queue 非空时替换当前队列，并按 song.id 重新定位 index。
     */
    fun play(song: Song, queue: List<Song> = emptyList()) {
        if (queue.isNotEmpty()) this.queue = queue
        val exo = player ?: return
        // 公开入口允许重算 index（用户点选的具体歌曲定位到队列位置）。
        index = this.queue.indexOfFirst { it.id == song.id }.takeIf { it >= 0 } ?: index
        _state.value = _state.value.copy(queue = this.queue)
        applyMediaItem(exo, song)
    }

    /**
     * 内部按索引播放（切歌入口）。AND-002 修复：不再调用会重算 index 的 play(song)，
     * 直接采用调用方计算好的 targetIndex，避免随机/顺序切歌被重定位回原 index。
     */
    private fun playAt(targetIndex: Int) {
        if (queue.isEmpty()) return
        val safeIndex = targetIndex.coerceIn(0, queue.lastIndex)
        index = safeIndex
        val exo = player ?: return
        applyMediaItem(exo, queue[safeIndex])
    }

    /** 实际设置 MediaItem 并开始播放。 */
    private fun applyMediaItem(exo: ExoPlayer, song: Song) {
        _state.value = _state.value.copy(currentSong = song)
        val url = song.playUrl ?: song.localPath
        if (url.isNullOrBlank()) return
        synchronized(playerLock) {
            exo.setMediaItem(MediaItem.fromUri(Uri.parse(url)))
            exo.prepare()
            exo.playWhenReady = true
        }
    }

    fun pause() {
        synchronized(playerLock) { player?.pause() }
    }
    fun resume() {
        synchronized(playerLock) { player?.play() }
    }

    fun seek(positionMs: Long) {
        synchronized(playerLock) { player?.seekTo(positionMs) }
        _state.value = _state.value.copy(position = positionMs)
    }

    fun toNext() = moveToNext(auto = false)

    private fun moveToNext(auto: Boolean) {
        if (queue.isEmpty()) return
        when (_state.value.mode) {
            PlayMode.SINGLE_LOOP -> {
                // 单曲循环：回到当前曲目起点继续播放，不变更 index。
                synchronized(playerLock) {
                    player?.seekTo(0)
                    player?.play()
                }
            }
            PlayMode.RANDOM -> {
                // 随机：在队列范围内取一个目标索引（与当前不同时优先）。
                val next = if (queue.size > 1) {
                    var candidate = (0 until queue.size).random()
                    while (candidate == index) candidate = (0 until queue.size).random()
                    candidate
                } else 0
                playAt(next)
            }
            PlayMode.SEQUENTIAL -> {
                // 顺序：到末尾后回到第一首（循环列表）。
                playAt((index + 1) % queue.size)
            }
        }
    }

    fun toPrev() {
        if (queue.isEmpty()) return
        // AND-002：直接计算目标索引并 playAt，不重算。
        playAt((index - 1 + queue.size) % queue.size)
    }

    fun setVolume(v: Float) {
        _state.value = _state.value.copy(volume = v)
        synchronized(playerLock) { player?.volume = v }
    }

    fun setMode(mode: PlayMode) {
        _state.value = _state.value.copy(mode = mode)
        synchronized(playerLock) {
            player?.repeatMode = when (mode) {
                PlayMode.SINGLE_LOOP -> Player.REPEAT_MODE_ONE
                else -> Player.REPEAT_MODE_OFF
            }
        }
    }

    fun toggleMode() {
        val all = PlayMode.entries
        val cur = _state.value.mode
        val next = all[(all.indexOf(cur) + 1) % all.size]
        setMode(next)
    }

    override fun onCleared() {
        // AND-001：先取消进度协程，再在锁内安全释放 player（cancel 后协程不再读取 player）。
        progressJob?.cancel()
        progressJob = null
        synchronized(playerLock) {
            // AND-003：显式移除 Listener，不依赖 release 隐式清理。
            playerListener?.let { player?.removeListener(it) }
            playerListener = null
            player?.release()
            player = null
        }
        super.onCleared()
    }
}
