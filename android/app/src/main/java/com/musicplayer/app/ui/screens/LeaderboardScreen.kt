// 职责：排行榜屏幕，榜单卡片网格，豆包风格 Card。
// 对齐桌面端 pages/leaderboard.js：调用 MusicRepository.listSourcesOrdered + getLeaderboards。
// 默认取首个启用音源的榜单；核心未连接时回退占位榜单。

package com.musicplayer.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.musicplayer.app.models.Leaderboard
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.repository.MusicRepository
import com.musicplayer.app.ui.theme.Background
import com.musicplayer.app.ui.theme.NghDimensions
import com.musicplayer.app.ui.theme.TextPrimary
import com.musicplayer.app.ui.theme.TextSecondary

/** 占位榜单：核心未连接时展示。 */
private val placeholderBoards: List<Leaderboard> = listOf(
    Leaderboard("l1", "demo", "热歌榜", null, emptyList()),
    Leaderboard("l2", "demo", "新歌榜", null, emptyList()),
    Leaderboard("l3", "demo", "飙升榜", null, emptyList())
)

@Composable
fun LeaderboardScreen(player: PlayerManager = viewModel()) {
    var boards by remember { mutableStateOf(placeholderBoards) }
    var loaded by remember { mutableStateOf(false) }

    // 首次进入：取首个启用音源拉取榜单；失败回退占位。
    LaunchedEffect(Unit) {
        val sources = MusicRepository.listSourcesOrdered()
        val firstEnabled = sources.firstOrNull { it.enabled }
        if (firstEnabled != null) {
            val fetched = MusicRepository.getLeaderboards(firstEnabled.id)
            if (fetched != null) boards = fetched
        }
        loaded = true
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Background)
            .padding(NghDimensions.spacing4)
    ) {
        Text("排行榜", style = MaterialTheme.typography.headlineSmall, color = TextPrimary)
        Text(
            "各音源热门榜单",
            style = MaterialTheme.typography.labelMedium,
            color = TextSecondary
        )
        Spacer(Modifier.height(NghDimensions.spacing4))

        if (boards.isEmpty() && loaded) {
            EmptyState("暂无排行榜", "请先在设置中导入并启用音源")
        } else {
            LazyVerticalGrid(
                columns = GridCells.Adaptive(minSize = 160.dp),
                verticalArrangement = Arrangement.spacedBy(NghDimensions.spacing3),
                horizontalArrangement = Arrangement.spacedBy(NghDimensions.spacing3),
                modifier = Modifier.fillMaxWidth()
            ) {
                items(boards) { b ->
                    GroupCard(title = b.name, subtitle = "${b.songs.size} 首") {
                        if (b.songs.isNotEmpty()) player.play(b.songs.first(), b.songs)
                    }
                }
            }
        }
    }
}
