// 职责：底部播放栏组件（PlaybackBar），逆光音乐 / NGHMusic。
// 对齐桌面端 components/PlaybackBar.vue：封面占位 + 标题/艺术家 + 进度条 + 上/下首/播放暂停/模式 + 音量。
// 仅使用 Material Icons（androidx.compose.material:material-icons-extended），不含 emoji。
// 有曲目时通过 AnimatedVisibility 淡入 + 垂直滑入；点击信息区回调 onOpenLyrics 进入歌词页。

package com.musicplayer.app.ui.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Pause
import androidx.compose.material.icons.filled.PlayArrow
import androidx.compose.material.icons.filled.Repeat
import androidx.compose.material.icons.filled.RepeatOne
import androidx.compose.material.icons.filled.Shuffle
import androidx.compose.material.icons.filled.SkipNext
import androidx.compose.material.icons.filled.SkipPrevious
import androidx.compose.material.icons.filled.VolumeUp
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import com.musicplayer.app.models.PlayMode
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.theme.Border
import com.musicplayer.app.ui.theme.NghDimensions
import com.musicplayer.app.ui.theme.Primary
import com.musicplayer.app.ui.theme.PrimaryHover
import com.musicplayer.app.ui.theme.Surface as SurfaceColor
import com.musicplayer.app.ui.theme.TextSecondary
import com.musicplayer.app.ui.theme.nghClickableScale

/**
 * 底部播放栏。
 *
 * @param player 播放管理器，提供状态与控制。
 * @param onOpenLyrics 点击信息区（封面/标题）时回调，由宿主导航至歌词/全屏播放页。
 */
@Composable
fun PlaybackBar(
    player: PlayerManager,
    onOpenLyrics: () -> Unit = {}
) {
    val state by player.state.collectAsState()
    // 有曲目时显示：fade + 垂直滑入/滑出（200ms，FastOutSlowInEasing）。
    AnimatedVisibility(
        visible = state.currentSong != null,
        enter = fadeIn(tween(200, easing = FastOutSlowInEasing)) +
            slideInVertically(animationSpec = tween(200, easing = FastOutSlowInEasing), initialOffsetY = { it }),
        exit = fadeOut(tween(200, easing = FastOutSlowInEasing)) +
            slideOutVertically(animationSpec = tween(200, easing = FastOutSlowInEasing), targetOffsetY = { it })
    ) {
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = NghDimensions.spacing3)
                .padding(bottom = NghDimensions.spacing2),
            shape = RoundedCornerShape(NghDimensions.radiusLg),
            color = SurfaceColor,
            shadowElevation = 4.dp
        ) {
            Column {
                // 进度条
                LinearProgressIndicator(
                    progress = { if (state.duration > 0) state.position.toFloat() / state.duration else 0f },
                    modifier = Modifier.fillMaxWidth().height(2.dp),
                    color = Primary,
                    trackColor = Border
                )
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = NghDimensions.spacing3, vertical = NghDimensions.spacing2),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    // 封面（点击进入歌词）
                    Box(
                        modifier = Modifier
                            .size(40.dp)
                            .clip(RoundedCornerShape(NghDimensions.radiusSm))
                            .background(Brush.linearGradient(listOf(Primary, PrimaryHover)))
                            .nghClickableScale { onOpenLyrics() }
                    )
                    Spacer(Modifier.width(NghDimensions.spacing2))
                    // 标题 / 艺术家（点击进入歌词）
                    Column(
                        modifier = Modifier
                            .weight(1f)
                            .nghClickableScale { onOpenLyrics() }
                    ) {
                        Text(
                            state.currentSong?.title ?: "未在播放",
                            style = MaterialTheme.typography.bodyMedium,
                            maxLines = 1, overflow = TextOverflow.Ellipsis
                        )
                        Text(
                            state.currentSong?.artists?.joinToString(" / ") ?: "—",
                            style = MaterialTheme.typography.labelMedium,
                            color = TextSecondary, maxLines = 1, overflow = TextOverflow.Ellipsis
                        )
                    }
                    // 上一首
                    IconButton(onClick = { player.toPrev() }) {
                        Icon(Icons.Filled.SkipPrevious, contentDescription = "上一首")
                    }
                    // 播放 / 暂停
                    IconButton(onClick = {
                        if (state.isPlaying) player.pause() else player.resume()
                    }) {
                        Icon(
                            if (state.isPlaying) Icons.Filled.Pause else Icons.Filled.PlayArrow,
                            contentDescription = "播放/暂停",
                            modifier = Modifier.size(28.dp)
                        )
                    }
                    // 下一首
                    IconButton(onClick = { player.toNext() }) {
                        Icon(Icons.Filled.SkipNext, contentDescription = "下一首")
                    }
                    // 播放模式
                    IconButton(onClick = { player.toggleMode() }) {
                        Icon(
                            when (state.mode) {
                                PlayMode.SEQUENTIAL -> Icons.Filled.Repeat
                                PlayMode.SINGLE_LOOP -> Icons.Filled.RepeatOne
                                PlayMode.RANDOM -> Icons.Filled.Shuffle
                            },
                            contentDescription = "播放模式"
                        )
                    }
                    // 音量
                    Icon(Icons.Filled.VolumeUp, contentDescription = null, tint = TextSecondary, modifier = Modifier.size(18.dp))
                    Slider(
                        value = state.volume,
                        onValueChange = { player.setVolume(it) },
                        modifier = Modifier.width(90.dp),
                        colors = SliderDefaults.colors(thumbColor = Primary, activeTrackColor = Primary)
                    )
                }
            }
        }
    }
}
