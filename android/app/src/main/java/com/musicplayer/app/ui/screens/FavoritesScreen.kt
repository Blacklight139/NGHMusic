// 职责：收藏屏幕，收藏分组卡片网格，简约风格占位。

package com.musicplayer.app.ui.screens

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.theme.*

data class FavoriteGroup(val id: String, val name: String, val songIds: List<String>)

@Composable
fun FavoritesScreen(player: PlayerManager = viewModel()) {
    val groups = remember {
        listOf(
            FavoriteGroup("f1", "我的收藏", listOf("s1", "s2", "s3")),
            FavoriteGroup("f2", "睡前音乐", listOf("s4", "s5"))
        )
    }

    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Text("收藏", style = MaterialTheme.typography.headlineSmall, color = TextPrimary)
        Text("多分组管理，支持导入/导出",
            style = MaterialTheme.typography.labelMedium, color = TextMuted)
        Spacer(Modifier.height(16.dp))

        if (groups.isEmpty()) {
            EmptyState("还没有收藏分组，点击右上角创建")
        } else {
            LazyVerticalGrid(
                columns = GridCells.Adaptive(minSize = 160.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp)
            ) {
                items(groups) { g ->
                    Card(title = g.name, subtitle = "${g.songIds.size} 首")
                }
            }
        }
    }
}

@Composable
fun Card(title: String, subtitle: String) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .border(1.dp, Border, RoundedCornerShape(8.dp))
            .padding(16.dp)
    ) {
        Text(title, style = MaterialTheme.typography.titleSmall, color = TextPrimary)
        Spacer(Modifier.height(2.dp))
        Text(subtitle, style = MaterialTheme.typography.labelMedium, color = TextMuted)
    }
}
