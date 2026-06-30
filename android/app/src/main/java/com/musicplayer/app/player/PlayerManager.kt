// 职责：封装 ExoPlayer/Media3，提供 setMediaItem/play/pause/seek/toNext/toPrev/setVolume/setRepeatMode，
//       通过 StateFlow 暴露播放状态（currentSong/position/duration/isPlaying/volume/mode）。
// 集成方式：作为 ViewModel 在 MainScreen 中注入；在 AndroidManifest 注册 MediaSessionService 实现后台播放。

package com.musicplayer.app.player

import android.net.Uri
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import androidx.media3.common.MediaItem
import androidx.media3.common.Player
import androidx.media3.exoplayer.ExoPlayer
import com.musicplayer.app.models.PlayMode
import com.musicplayer.app.models.Song
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class PlayerState(
    val currentSong: Song? = null,
    val isPlaying: Bool = false,
    val position: Long = 0L,      // 毫秒
    val duration: Long = 0L,     // 毫秒
    val volume: Float = 0.8f,
    val mode: PlayMode = PlayMode.SEQUENTIAL
)

class PlayerManager : ViewModel() {
    private var player: ExoPlayer? = null
    private var queue: List<Song> = emptyList()
    private var index: Int = 0

    private val _state = MutableStateFlow(PlayerState())
    val state: StateFlow<PlayerState> = _state.asStateFlow()

    /** 由 Composable 在创建 ExoPlayer 后注入（持有 Context） */
    fun attach(exoPlayer: ExoPlayer) {
        this.player = exoPlayer
        exoPlayer.volume = _state.value.volume
        exoPlayer.addListener(object : Player.Listener {
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
        })
        startProgressLoop()
    }

    private fun startProgressLoop() {
        viewModelScope.launch {
            while (true) {
                player?.let {
                    val pos = it.currentPosition.coerceAtLeast(0)
                    _state.value = _state.value.copy(position = pos)
                }
                delay(500)
            }
        }
    }

    fun play(song: Song, queue: List<Song> = emptyList()) {
        if (queue.isNotEmpty()) this.queue = queue
        val exo = player ?: return
        index = this.queue.indexOfFirst { it.id == song.id }.takeIf { it >= 0 } ?: index
        _state.value = _state.value.copy(currentSong = song)
        val url = song.playUrl ?: return
        exo.setMediaItem(MediaItem.fromUri(Uri.parse(url)))
        exo.prepare()
        exo.playWhenReady = true
    }

    fun pause() { player?.pause() }
    fun resume() { player?.play() }

    fun seek(positionMs: Long) {
        player?.seekTo(positionMs)
        _state.value = _state.value.copy(position = positionMs)
    }

    fun toNext() = moveToNext(auto = false)

    private fun moveToNext(auto: Boolean) {
        if (queue.isEmpty()) return
        when (_state.value.mode) {
            PlayMode.SINGLE_LOOP -> {
                player?.seekTo(0)
                player?.play()
            }
            PlayMode.RANDOM -> {
                index = (0 until queue.size).random()
                play(queue[index])
            }
            PlayMode.SEQUENTIAL -> {
                index = (index + 1) % queue.size
                play(queue[index])
            }
        }
    }

    fun toPrev() {
        if (queue.isEmpty()) return
        index = (index - 1 + queue.size) % queue.size
        play(queue[index])
    }

    fun setVolume(v: Float) {
        _state.value = _state.value.copy(volume = v)
        player?.volume = v
    }

    fun setMode(mode: PlayMode) {
        _state.value = _state.value.copy(mode = mode)
        player?.repeatMode = when (mode) {
            PlayMode.SINGLE_LOOP -> Player.REPEAT_MODE_ONE
            else -> Player.REPEAT_MODE_OFF
        }
    }

    fun toggleMode() {
        val all = PlayMode.entries
        val cur = _state.value.mode
        val next = all[(all.indexOf(cur) + 1) % all.size]
        setMode(next)
    }

    override fun onCleared() {
        player?.release()
        super.onCleared()
    }
}
