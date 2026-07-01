// 职责：排行榜屏幕，榜单卡片网格，豆包风格 Card。

package com.musicplayer.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.musicplayer.app.models.Leaderboard
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.theme.*

@Composable
fun LeaderboardScreen(player: PlayerManager = viewModel()) {
    val boards = remember {
        listOf(
            Leaderboard("l1", "demo", "热歌榜", null, emptyList()),
            Leaderboard("l2", "demo", "新歌榜", null, emptyList()),
            Leaderboard("l3", "demo", "飙升榜", null, emptyList())
        )
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Background)
            .padding(NghDimensions.spacing4)
    ) {
        Text("排行榜", style = MaterialTheme.typography.headlineSmall, color = TextPrimary)
        Text("各音源热门榜单", style = MaterialTheme.typography.labelMedium, color = TextSecondary)
        Spacer(Modifier.height(NghDimensions.spacing4))

        LazyVerticalGrid(
            columns = GridCells.Adaptive(minSize = 160.dp),
            verticalArrangement = Arrangement.spacedBy(NghDimensions.spacing3),
            horizontalArrangement = Arrangement.spacedBy(NghDimensions.spacing3)
        ) {
            items(boards) { b -> GroupCard(title = b.name, subtitle = "${b.songs.size} 首") }
        }
    }
}
