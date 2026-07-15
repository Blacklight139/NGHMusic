// 职责：歌词屏幕，逐行展示并高亮当前行，豆包风格：active 行 Primary，非 active TextTertiary。
// 对齐桌面端 pages/lyrics.js：根据当前歌曲调用 MusicRepository.getLyric 拉取歌词，
// 与 PlayerManager.position 同步滚动；无核心连接时回退占位歌词。

package com.musicplayer.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.musicplayer.app.models.Lyric
import com.musicplayer.app.models.LyricLine
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.repository.MusicRepository
import com.musicplayer.app.ui.theme.Background
import com.musicplayer.app.ui.theme.NghDimensions
import com.musicplayer.app.ui.theme.Primary
import com.musicplayer.app.ui.theme.Surface
import com.musicplayer.app.ui.theme.TextPrimary
import com.musicplayer.app.ui.theme.TextTertiary

/** 占位歌词：核心未连接时展示，保证 UI 结构可预览。 */
private val placeholderLyric: Lyric = Lyric(
    lines = listOf(
        LyricLine(0L, "示例歌词第一行"),
        LyricLine(5000L, "示例歌词第二行"),
        LyricLine(10000L, "示例歌词第三行"),
        LyricLine(15000L, "示例歌词第四行")
    ),
    translation = null
)

@Composable
fun LyricsScreen(player: PlayerManager = viewModel()) {
    val state by player.state.collectAsState()
    val listState = rememberLazyListState()
    var lyric by remember { mutableStateOf(placeholderLyric) }

    // 当前歌曲变化时拉取歌词；失败回退占位。
    LaunchedEffect(state.currentSong?.id) {
        val song = state.currentSong
        if (song == null) {
            lyric = placeholderLyric
        } else {
            val fetched = MusicRepository.getLyric(song.sourceId, song.id)
            lyric = fetched ?: placeholderLyric
        }
    }

    val lines = lyric.lines
    val currentIndex = remember(state.position, lines) {
        var idx = 0
        lines.forEachIndexed { i, line ->
            // 7.4 跳过 timeMs 为 null 的纯文本行，避免其被当作时间戳 0 而误判为当前行。
            val timeMs = line.timeMs ?: return@forEachIndexed
            if (timeMs <= state.position) idx = i
        }
        idx
    }

    LaunchedEffect(currentIndex) {
        if (lines.isNotEmpty()) listState.animateScrollToItem(currentIndex)
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Background)
            .padding(NghDimensions.spacing4)
    ) {
        Text("歌词", style = MaterialTheme.typography.headlineSmall, color = TextPrimary)
        Spacer(Modifier.height(NghDimensions.spacing4))
        Card(
            modifier = Modifier.fillMaxWidth(),
            shape = RoundedCornerShape(NghDimensions.radiusMd),
            colors = CardDefaults.cardColors(containerColor = Surface),
            elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
        ) {
            LazyColumn(
                state = listState,
                modifier = Modifier.fillMaxSize().padding(NghDimensions.spacing4),
                horizontalAlignment = Alignment.CenterHorizontally
            ) {
                itemsIndexed(lines) { i, line ->
                    Text(
                        line.text,
                        fontSize = 16.sp,
                        fontWeight = if (i == currentIndex) FontWeight.SemiBold else FontWeight.Normal,
                        color = if (i == currentIndex) Primary else TextTertiary,
                        modifier = Modifier.padding(vertical = NghDimensions.spacing3)
                    )
                }
            }
        }
    }
}
