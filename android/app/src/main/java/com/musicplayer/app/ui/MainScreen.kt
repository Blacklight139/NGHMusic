// 职责：主屏幕，Scaffold + NavigationBar（搜索/列表/收藏/排行榜/本地/设置）+ 底部 PlaybackBar。
// 对齐桌面端：底部固定播放栏（封面/标题/控制/进度/音量），点击信息区进入歌词页。
// 播放栏抽离为 ui/components/PlaybackBar.kt，本文件负责导航壳与各屏幕路由注册。

package com.musicplayer.app.ui

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.scaleIn
import androidx.compose.animation.scaleOut
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.QueueMusic
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material.icons.filled.TrendingUp
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.navigation.NavGraph.Companion.findStartDestination
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.components.PlaybackBar
import com.musicplayer.app.ui.screens.FavoritesScreen
import com.musicplayer.app.ui.screens.LeaderboardScreen
import com.musicplayer.app.ui.screens.LocalMusicScreen
import com.musicplayer.app.ui.screens.LyricsScreen
import com.musicplayer.app.ui.screens.NasScreen
import com.musicplayer.app.ui.screens.PlaylistScreen
import com.musicplayer.app.ui.screens.SearchScreen
import com.musicplayer.app.ui.screens.SettingsScreen
import com.musicplayer.app.ui.theme.Background
import com.musicplayer.app.ui.theme.Primary
import com.musicplayer.app.ui.theme.TextPrimary
import com.musicplayer.app.ui.theme.TextSecondary

// 导航条目（主 Tab）
sealed class AppTab(val route: String, val title: String, val icon: ImageVector) {
    data object Search : AppTab("search", "搜索", Icons.Filled.Search)
    data object Playlist : AppTab("playlist", "播放列表", Icons.Filled.QueueMusic)
    data object Favorites : AppTab("favorites", "收藏", Icons.Filled.Favorite)
    data object Leaderboard : AppTab("leaderboard", "排行榜", Icons.Filled.TrendingUp)
    data object Local : AppTab("local", "本地音乐", Icons.Filled.Folder)
    data object Nas : AppTab("nas", "NAS", Icons.Filled.Storage)
    data object Settings : AppTab("settings", "设置", Icons.Filled.Settings)
}

/** 歌词/全屏播放页路由（非底部 Tab，由播放栏点击进入）。 */
object AppRoute {
    const val LYRICS = "lyrics"
}

private val tabs = listOf(
    AppTab.Search, AppTab.Playlist, AppTab.Favorites,
    AppTab.Leaderboard, AppTab.Local, AppTab.Nas, AppTab.Settings
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainScreen(player: PlayerManager = viewModel()) {
    val navController = rememberNavController()
    val navBackStack by navController.currentBackStackEntryAsState()
    val currentRoute = navBackStack?.destination?.route

    Scaffold(
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text("逆光音乐", fontSize = 18.sp, fontWeight = FontWeight.Medium)
                        Text("NGHMusic", fontSize = 12.sp, color = TextSecondary)
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = Background,
                    titleContentColor = TextPrimary,
                    navigationIconColor = Primary
                )
            )
        },
        bottomBar = {
            Column {
                // 底部播放栏：点击信息区进入歌词页
                PlaybackBar(
                    player = player,
                    onOpenLyrics = { navController.navigate(AppRoute.LYRICS) }
                )
                NavigationBar(containerColor = Background) {
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
                                unselectedIconColor = TextSecondary,
                                unselectedTextColor = TextSecondary
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
            modifier = Modifier.padding(inner),
            // 豆包风格柔和切换：进入 fade + 轻微 scaleIn(0.98)，退出 fadeOut + scaleOut(0.98)。
            enterTransition = {
                fadeIn(tween(200, easing = FastOutSlowInEasing)) +
                    scaleIn(initialScale = 0.98f, animationSpec = tween(200, easing = FastOutSlowInEasing))
            },
            exitTransition = {
                fadeOut(tween(150)) + scaleOut(targetScale = 0.98f, animationSpec = tween(150))
            },
            popEnterTransition = {
                fadeIn(tween(200, easing = FastOutSlowInEasing)) +
                    scaleIn(initialScale = 0.98f, animationSpec = tween(200, easing = FastOutSlowInEasing))
            },
            popExitTransition = {
                fadeOut(tween(150)) + scaleOut(targetScale = 0.98f, animationSpec = tween(150))
            }
        ) {
            composable(AppTab.Search.route) { SearchScreen(player) }
            composable(AppTab.Playlist.route) { PlaylistScreen(player) }
            composable(AppTab.Favorites.route) { FavoritesScreen(player) }
            composable(AppTab.Leaderboard.route) { LeaderboardScreen(player) }
            composable(AppTab.Local.route) { LocalMusicScreen(player) }
            composable(AppTab.Nas.route) { NasScreen(player) }
            composable(AppTab.Settings.route) { SettingsScreen(player) }
            composable(AppRoute.LYRICS) { LyricsScreen(player) }
        }
    }
}
