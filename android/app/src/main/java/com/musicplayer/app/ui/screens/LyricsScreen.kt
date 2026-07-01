// 职责：歌词屏幕，逐行展示并高亮当前行，豆包风格：active 行 Primary，非 active TextTertiary。
// 对齐桌面端 pages/lyrics.js：与 PlayerManager.position 同步滚动。

package com.musicplayer.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.theme.*

private data class LyricLine(val timeMs: Long?, val text: String)

@Composable
fun LyricsScreen(player: PlayerManager = viewModel()) {
    val lines = remember {
        listOf(
            LyricLine(0L, "示例歌词第一行"),
            LyricLine(5000L, "示例歌词第二行"),
            LyricLine(10000L, "示例歌词第三行"),
            LyricLine(15000L, "示例歌词第四行")
        )
    }
    val state by player.state.collectAsState()
    val listState = rememberLazyListState()

    val currentIndex = remember(state.position) {
        var idx = 0
        lines.forEachIndexed { i, line ->
            if ((line.timeMs ?: 0) <= state.position) idx = i
        }
        idx
    }

    LaunchedEffect(currentIndex) {
        listState.animateScrollToItem(currentIndex)
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
