// 职责：播放列表屏幕，展示当前播放队列，豆包风格线性列表（无卡片，分隔线）+ 统一空状态。
// 数据来源：PlayerManager.state.queue（搜索/本地点击播放时写入）。
// 复用 SearchScreen.SongRowItem 保持跨页列表行一致（高度/间距/排版/当前播放高亮）。

package com.musicplayer.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Inbox
import androidx.compose.material.icons.filled.QueueMusic
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.theme.Background
import com.musicplayer.app.ui.theme.NghDimensions
import com.musicplayer.app.ui.theme.TextPrimary
import com.musicplayer.app.ui.theme.TextSecondary
import com.musicplayer.app.ui.theme.TextTertiary

@Composable
fun PlaylistScreen(player: PlayerManager = viewModel()) {
    val state by player.state.collectAsState()
    val songs = state.queue
    val currentSong = state.currentSong

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
            LazyColumn(verticalArrangement = Arrangement.spacedBy(0.dp)) {
                itemsIndexed(songs, key = { _, item -> item.id }) { i, song ->
                    SongRowItem(
                        index = i + 1,
                        song = song,
                        onClick = { player.play(song, songs) },
                        isCurrent = song == currentSong
                    )
                }
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
