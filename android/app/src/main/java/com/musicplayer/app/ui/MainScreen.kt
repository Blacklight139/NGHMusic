// 职责：主屏幕，Scaffold + NavigationBar（搜索/列表/收藏/排行榜/本地/设置）+ 底部 MiniPlayer。
// 对齐桌面端：底部固定迷你播放栏（封面/标题/控制/进度/音量）。

package com.musicplayer.app.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.*
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.screens.*
import com.musicplayer.app.ui.theme.*

// 导航条目
sealed class AppTab(val route: String, val title: String, val icon: androidx.compose.ui.graphics.vector.ImageVector) {
    data object Search : AppTab("search", "搜索", Icons.Filled.Search)
    data object Playlist : AppTab("playlist", "播放列表", Icons.Filled.QueueMusic)
    data object Favorites : AppTab("favorites", "收藏", Icons.Filled.Star)
    data object Leaderboard : AppTab("leaderboard", "排行榜", Icons.Filled.EmojiEvents)
    data object Local : AppTab("local", "本地音乐", Icons.Filled.Folder)
    data object Settings : AppTab("settings", "设置", Icons.Filled.Settings)
}

private val tabs = listOf(AppTab.Search, AppTab.Playlist, AppTab.Favorites,
    AppTab.Leaderboard, AppTab.Local, AppTab.Settings)

@Composable
fun MainScreen(player: PlayerManager = viewModel()) {
    val navController = rememberNavController()
    val navBackStack by navController.currentBackStackEntryAsState()
    val currentRoute = navBackStack?.destination?.route

    Scaffold(
        bottomBar = {
            Column {
                MiniPlayer(player = player)
                NavigationBar(containerColor = Bg) {
                    tabs.forEach { tab ->
                        NavigationBarItem(
                            selected = currentRoute == tab.route,
                            onClick = {
                                navController.navigate(tab.route) {
                                    popUpTo(navController.graph.findStartDestination().id) { saveState = true }
                                    launchSingleTop = true
                                    restoreState = true
                                }
                            },
                            icon = { Icon(tab.icon, contentDescription = tab.title) },
                            label = { Text(tab.title, fontSize = 10.sp) },
                            colors = NavigationBarItemDefaults.colors(
                                selectedIconColor = Primary,
                                selectedTextColor = Primary,
                                unselectedIconColor = TextMuted,
                                unselectedTextColor = TextMuted
                            )
                        )
                    }
                }
            }
        }
    ) { inner ->
        NavHost(
            navController = navController,
            startDestination = AppTab.Search.route,
            modifier = Modifier.padding(inner)
        ) {
            composable(AppTab.Search.route) { SearchScreen(player) }
            composable(AppTab.Playlist.route) { PlaylistScreen(player) }
            composable(AppTab.Favorites.route) { FavoritesScreen(player) }
            composable(AppTab.Leaderboard.route) { LeaderboardScreen(player) }
            composable(AppTab.Local.route) { LocalMusicScreen(player) }
            composable(AppTab.Settings.route) { SettingsScreen(player) }
            composable("lyrics") { LyricsScreen(player) }
        }
    }
}

@Composable
private fun MiniPlayer(player: PlayerManager) {
    val state by player.state.collectAsState()
    Column {
        // 进度条
        LinearProgressIndicator(
            progress = { if (state.duration > 0) state.position.toFloat() / state.duration else 0f },
            modifier = Modifier.fillMaxWidth().height(2.dp),
            color = Primary,
            trackColor = Border
        )
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .background(Bg)
                .padding(horizontal = 16.dp, vertical = 6.dp),
            verticalAlignment = Alignment.CenterVertically
        ) {
            // 封面
            Box(
                modifier = Modifier
                    .size(40.dp)
                    .clip(RoundedCornerShape(8.dp))
                    .background(Brush.linearGradient(listOf(Primary, PrimaryDark)))
            )
            Spacer(Modifier.width(8.dp))
            // 标题
            Column(Modifier.weight(1f)) {
                Text(
                    state.currentSong?.title ?: "未在播放",
                    style = MaterialTheme.typography.bodyMedium,
                    maxLines = 1, overflow = TextOverflow.Ellipsis
                )
                Text(
                    state.currentSong?.artists?.joinToString(" / ") ?: "—",
                    style = MaterialTheme.typography.labelMedium,
                    color = TextMuted, maxLines = 1, overflow = TextOverflow.Ellipsis
                )
            }
            // 控制
            IconButton(onClick = { player.toPrev() }) { Icon(Icons.Filled.SkipPrevious, "上一首") }
            IconButton(onClick = {
                if (state.isPlaying) player.pause() else player.resume()
            }) {
                Icon(
                    if (state.isPlaying) Icons.Filled.Pause else Icons.Filled.PlayArrow,
                    "播放/暂停", modifier = Modifier.size(28.dp)
                )
            }
            IconButton(onClick = { player.toNext() }) { Icon(Icons.Filled.SkipNext, "下一首") }
            // 模式
            IconButton(onClick = { player.toggleMode() }) {
                Icon(
                    when (state.mode) {
                        com.musicplayer.app.models.PlayMode.SEQUENTIAL -> Icons.Filled.Repeat
                        com.musicplayer.app.models.PlayMode.SINGLE_LOOP -> Icons.Filled.RepeatOne
                        com.musicplayer.app.models.PlayMode.RANDOM -> Icons.Filled.Shuffle
                    },
                    "播放模式"
                )
            }
            // 音量
            Icon(Icons.Filled.VolumeUp, null, tint = TextMuted, modifier = Modifier.size(18.dp))
            Slider(
                value = state.volume,
                onValueChange = { player.setVolume(it) },
                modifier = Modifier.width(90.dp),
                colors = SliderDefaults.colors(thumbColor = Primary, activeTrackColor = Primary)
            )
        }
    }
}
