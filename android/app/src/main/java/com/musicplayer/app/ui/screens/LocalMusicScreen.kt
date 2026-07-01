// 职责：本地音乐屏幕，按歌曲/专辑/艺术家/文件夹浏览，豆包风格 Card 列表 + 统一空状态。

package com.musicplayer.app.ui.screens

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.LibraryMusic
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.musicplayer.app.models.Song
import com.musicplayer.app.models.SongOrigin
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.theme.*

@Composable
fun LocalMusicScreen(player: PlayerManager = viewModel()) {
    val songs = remember {
        listOf(
            Song("lo1", "local", "本地示例一", listOf("未知艺术家"),
                album = "本地专辑", durationMs = 198000, localPath = "/music/a.mp3",
                origin = SongOrigin.Local("/music/a.mp3")),
            Song("lo2", "local", "本地示例二", listOf("艺术家C"),
                durationMs = 165000, localPath = "/music/b.flac",
                origin = SongOrigin.Local("/music/b.flac"))
        )
    }
    var filter by remember { mutableIntStateOf(0) }
    val filters = listOf("歌曲", "专辑", "艺术家", "文件夹")

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Background)
            .padding(NghDimensions.spacing4)
    ) {
        Text("本地音乐", style = MaterialTheme.typography.headlineSmall, color = TextPrimary)
        Text("扫描本地目录并播放", style = MaterialTheme.typography.labelMedium, color = TextSecondary)
        Spacer(Modifier.height(NghDimensions.spacing4))

        SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
            filters.forEachIndexed { i, label ->
                SegmentedButton(
                    selected = filter == i,
                    onClick = { filter = i },
                    shape = SegmentedButtonDefaults.itemShape(i, filters.size)
                ) { Text(label, style = MaterialTheme.typography.labelMedium) }
            }
        }

        Spacer(Modifier.height(NghDimensions.spacing4))
        if (songs.isEmpty()) {
            EmptyState("尚未扫描本地音乐", "前往设置添加目录", icon = Icons.Filled.LibraryMusic)
        } else {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(NghDimensions.spacing3)) {
                itemsIndexed(songs) { i, song -> SongRowItem(i + 1, song) }
            }
        }
    }
}
