// 职责：搜索屏幕，搜索栏 + 结果列表，豆包风格 Card 列表。
// 对齐桌面端 pages/search.js：调用 MusicCoreBridge.search。

package com.musicplayer.app.ui.screens

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.musicplayer.app.bridge.MusicCoreBridge
import com.musicplayer.app.models.Song
import com.musicplayer.app.models.SongOrigin
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.theme.*

@Composable
fun SearchScreen(player: PlayerManager = viewModel()) {
    var keyword by remember { mutableStateOf("") }
    var loading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var songs by remember {
        mutableStateOf(
            listOf(
                Song("s1", "demo", "示例歌曲一", listOf("艺术家A"),
                    album = "专辑X", durationMs = 210000,
                    origin = SongOrigin.Online("demo", "")),
                Song("s2", "demo", "示例歌曲二", listOf("艺术家B"),
                    durationMs = 184000, origin = SongOrigin.Online("demo", ""))
            )
        )
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Background)
            .padding(NghDimensions.spacing4)
    ) {
        Text("搜索", style = MaterialTheme.typography.headlineSmall, color = TextPrimary)
        Text("跨音源聚合检索", style = MaterialTheme.typography.labelMedium, color = TextSecondary)
        Spacer(Modifier.height(NghDimensions.spacing4))

        Row(verticalAlignment = Alignment.CenterVertically) {
            OutlinedTextField(
                value = keyword,
                onValueChange = { keyword = it },
                placeholder = { Text("输入歌曲 / 艺术家 / 专辑") },
                modifier = Modifier.weight(1f),
                shape = RoundedCornerShape(NghDimensions.radiusSm),
                singleLine = true
            )
            Spacer(Modifier.width(NghDimensions.spacing2))
            Button(
                onClick = {
                    if (keyword.isBlank()) return@Button
                    loading = true; error = null
                },
                colors = ButtonDefaults.buttonColors(containerColor = Primary)
            ) {
                Icon(Icons.Filled.Search, null)
                Spacer(Modifier.width(NghDimensions.spacing1))
                Text("搜索")
            }
        }

        if (loading) {
            Spacer(Modifier.height(NghDimensions.spacing4))
            CircularProgressIndicator(color = Primary)
        }
        error?.let {
            Spacer(Modifier.height(NghDimensions.spacing2))
            Text(it, color = Danger, fontSize = 12.sp)
        }

        Spacer(Modifier.height(NghDimensions.spacing4))
        LazyColumn(verticalArrangement = Arrangement.spacedBy(NghDimensions.spacing3)) {
            itemsIndexed(songs) { i, song -> SongRowItem(i + 1, song) }
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun LazyItemScope.SongRowItem(index: Int, song: Song) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .animateItemPlacement(tween(200, easing = FastOutSlowInEasing))
            .nghClickableScale { },
        shape = RoundedCornerShape(NghDimensions.radiusMd),
        colors = CardDefaults.cardColors(containerColor = Surface),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .padding(NghDimensions.spacing4),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Text("$index", color = TextTertiary, fontSize = 12.sp, modifier = Modifier.width(24.dp))
            Spacer(Modifier.width(NghDimensions.spacing2))
            Column(Modifier.weight(1f)) {
                Text(song.title, style = MaterialTheme.typography.titleSmall, color = TextPrimary)
                Text(
                    song.artists.joinToString(" / "),
                    style = MaterialTheme.typography.labelMedium,
                    color = TextSecondary
                )
            }
            Spacer(Modifier.width(NghDimensions.spacing2))
            AssistChip(onClick = {}, label = { Text("在线", fontSize = 11.sp) })
        }
    }
}
