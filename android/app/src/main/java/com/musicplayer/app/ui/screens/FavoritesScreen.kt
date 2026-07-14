// 职责：收藏屏幕，收藏分组卡片网格，豆包风格 Card + 统一空状态。
// 说明：收藏分组持久化接口未在 docs/api 中暴露（core 模型含 FavoriteGroup，但无 HTTP 端点），
//       此处保留占位分组以维持结构完整；点击分组卡片回调 onClick，供后续接入播放/详情。

package com.musicplayer.app.ui.screens

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.tween
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyGridItemScope
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.FavoriteBorder
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.theme.Background
import com.musicplayer.app.ui.theme.NghDimensions
import com.musicplayer.app.ui.theme.SurfaceAlt
import com.musicplayer.app.ui.theme.TextPrimary
import com.musicplayer.app.ui.theme.TextSecondary
import com.musicplayer.app.ui.theme.nghClickableScale

data class FavoriteGroup(val id: String, val name: String, val songIds: List<String>)

@Composable
fun FavoritesScreen(player: PlayerManager = viewModel()) {
    val groups = remember {
        listOf(
            FavoriteGroup("f1", "我的收藏", listOf("s1", "s2", "s3")),
            FavoriteGroup("f2", "睡前音乐", listOf("s4", "s5"))
        )
    }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .background(Background)
            .padding(NghDimensions.spacing4)
    ) {
        Text("收藏", style = MaterialTheme.typography.headlineSmall, color = TextPrimary)
        Text(
            "多分组管理，支持导入/导出",
            style = MaterialTheme.typography.labelMedium,
            color = TextSecondary
        )
        Spacer(Modifier.height(NghDimensions.spacing4))

        if (groups.isEmpty()) {
            EmptyState("还没有收藏分组", "点击右上角创建", icon = Icons.Filled.FavoriteBorder)
        } else {
            LazyVerticalGrid(
                columns = GridCells.Adaptive(minSize = 160.dp),
                verticalArrangement = Arrangement.spacedBy(NghDimensions.spacing3),
                horizontalArrangement = Arrangement.spacedBy(NghDimensions.spacing3),
                modifier = Modifier.fillMaxWidth()
            ) {
                items(groups) { g ->
                    GroupCard(title = g.name, subtitle = "${g.songIds.size} 首") {
                        // 占位：接入收藏详情/播放后替换
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
fun LazyGridItemScope.GroupCard(
    title: String,
    subtitle: String,
    onClick: () -> Unit = {}
) {
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .animateItemPlacement(tween(200, easing = FastOutSlowInEasing))
            .nghClickableScale { onClick() }
            .clip(RoundedCornerShape(NghDimensions.radiusMd))
            .background(SurfaceAlt)
            .padding(NghDimensions.spacing4)
    ) {
        Text(
            title,
            style = MaterialTheme.typography.titleSmall,
            color = TextPrimary,
            fontWeight = FontWeight.Medium
        )
        Spacer(Modifier.height(NghDimensions.spacing1))
        Text(subtitle, style = MaterialTheme.typography.labelMedium, color = TextSecondary)
    }
}
