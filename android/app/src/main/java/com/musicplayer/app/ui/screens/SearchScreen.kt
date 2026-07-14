// 职责：搜索屏幕，搜索栏 + 结果列表，豆包风格 Card 列表。
// 对齐桌面端 pages/search.js：调用 MusicRepository.search（跨音源聚合检索）。

package com.musicplayer.app.ui.screens

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Search
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.musicplayer.app.models.Song
import com.musicplayer.app.models.SongOrigin
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.repository.MusicRepository
import com.musicplayer.app.ui.theme.Background
import com.musicplayer.app.ui.theme.Danger
import com.musicplayer.app.ui.theme.NghDimensions
import com.musicplayer.app.ui.theme.Primary
import com.musicplayer.app.ui.theme.Surface
import com.musicplayer.app.ui.theme.TextPrimary
import com.musicplayer.app.ui.theme.TextSecondary
import com.musicplayer.app.ui.theme.TextTertiary
import com.musicplayer.app.ui.theme.nghClickableScale
import kotlinx.coroutines.launch

@Composable
fun SearchScreen(player: PlayerManager = viewModel()) {
    var keyword by remember { mutableStateOf("") }
    var loading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }
    var songs by remember { mutableStateOf<List<Song>>(emptyList()) }
    val scope = rememberCoroutineScope()

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
                    scope.launch {
                        loading = true
                        error = null
                        val result = MusicRepository.search(keyword)
                        songs = result?.songs ?: emptyList()
                        if (result == null) {
                            error = "未连接核心或无结果，已显示空列表"
                        }
                        loading = false
                    }
                },
                colors = ButtonDefaults.buttonColors(containerColor = Primary)
            ) {
                Icon(Icons.Filled.Search, contentDescription = null)
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
        if (songs.isEmpty() && !loading) {
            EmptyState("暂无搜索结果", "输入关键词后点击搜索", icon = Icons.Filled.Search)
        } else {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(NghDimensions.spacing3)) {
                itemsIndexed(songs) { i, song ->
                    SongRowItem(
                        index = i + 1,
                        song = song,
                        onClick = { player.play(song, songs) }
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun LazyItemScope.SongRowItem(index: Int, song: Song, onClick: () -> Unit = {}) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .animateItemPlacement(tween(200, easing = FastOutSlowInEasing))
            .nghClickableScale { onClick() },
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
            val originLabel = when (song.origin) {
                is SongOrigin.Online -> "在线"
                is SongOrigin.Local -> "本地"
                is SongOrigin.Nas -> "NAS"
            }
            AssistChip(onClick = onClick, label = { Text(originLabel, fontSize = 11.sp) })
        }
    }
}
