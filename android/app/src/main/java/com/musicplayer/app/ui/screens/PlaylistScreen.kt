// 职责：播放列表屏幕，展示当前队列歌曲，豆包风格 Card 列表 + 统一空状态。

package com.musicplayer.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Inbox
import androidx.compose.material.icons.filled.QueueMusic
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.musicplayer.app.models.Song
import com.musicplayer.app.models.SongOrigin
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.theme.*

@Composable
fun PlaylistScreen(player: PlayerManager = viewModel()) {
    val songs = remember {
        listOf(
            Song("p1", "demo", "播放列表曲目一", listOf("艺术家A"),
                durationMs = 200000, origin = SongOrigin.Online("demo", "")),
            Song("p2", "demo", "播放列表曲目二", listOf("艺术家B"),
                durationMs = 175000, origin = SongOrigin.Online("demo", ""))
        )
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Background)
            .padding(NghDimensions.spacing4)
    ) {
        Text("播放列表", style = MaterialTheme.typography.headlineSmall, color = TextPrimary)
        Text(
            "当前队列共 ${songs.size} 首",
            style = MaterialTheme.typography.labelMedium,
            color = TextSecondary
        )
        Spacer(Modifier.height(NghDimensions.spacing4))

        if (songs.isEmpty()) {
            EmptyState("播放列表为空", "去搜索添加歌曲吧", icon = Icons.Filled.QueueMusic)
        } else {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(NghDimensions.spacing3)) {
                itemsIndexed(songs) { i, song -> SongRowItem(i + 1, song) }
            }
        }
    }
}

/**
 * 豆包风格统一空状态：Box 居中 Column { Icon + Text 标题 + Text 副标题 }。
 * 图标默认 Inbox，颜色 TextTertiary；标题 TextSecondary，副标题 TextTertiary。
 */
@Composable
fun EmptyState(
    text: String,
    subtitle: String? = null,
    icon: ImageVector = Icons.Filled.Inbox
) {
    Box(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = NghDimensions.spacing6),
        contentAlignment = Alignment.Center
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Icon(
                icon,
                contentDescription = null,
                tint = TextTertiary,
                modifier = Modifier.size(40.dp)
            )
            Spacer(Modifier.height(NghDimensions.spacing2))
            Text(
                text,
                style = MaterialTheme.typography.titleSmall,
                color = TextSecondary,
                textAlign = TextAlign.Center
            )
            if (subtitle != null) {
                Spacer(Modifier.height(NghDimensions.spacing1))
                Text(
                    subtitle,
                    style = MaterialTheme.typography.labelMedium,
                    color = TextTertiary,
                    textAlign = TextAlign.Center
                )
            }
        }
    }
}
