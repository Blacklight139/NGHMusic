// 职责：播放列表屏幕，展示当前队列歌曲，简约风格占位。

package com.musicplayer.app.ui.screens

import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
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

    Column(modifier = Modifier.fillMaxSize().padding(16.dp)) {
        Text("播放列表", style = MaterialTheme.typography.headlineSmall, color = TextPrimary)
        Text("当前队列共 ${songs.size} 首",
            style = MaterialTheme.typography.labelMedium, color = TextMuted)
        Spacer(Modifier.height(16.dp))

        if (songs.isEmpty()) {
            EmptyState("播放列表为空，去搜索添加歌曲吧")
        } else {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                itemsIndexed(songs) { i, song -> SongRowItem(i + 1, song) }
            }
        }
    }
}

@Composable
fun EmptyState(text: String) {
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(8.dp),
        border = androidx.compose.foundation.BorderStroke(1.dp, Border),
        color = Bg
    ) {
        Text(text, style = MaterialTheme.typography.labelMedium, color = TextMuted,
            textAlign = TextAlign.Center, modifier = Modifier.padding(24.dp))
    }
}
