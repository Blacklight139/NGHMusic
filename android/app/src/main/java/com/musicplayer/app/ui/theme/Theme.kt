// 职责：Material3 主题，豆包风格设计系统（逆光音乐 / NGHMusic），主色 #4E6EF2。
// 集成方式：在 MainActivity.setContent 中调用 MusicPlayerTheme { ... }。
// 提供 NghDimensions 尺寸 token，供圆角 / 间距统一引用。

package com.musicplayer.app.ui.theme

import androidx.compose.animation.core.EaseOut
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.composed
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp

private val LightColorScheme = lightColorScheme(
    primary = Primary,
    onPrimary = Color.White,
    primaryContainer = PrimarySoft,
    onPrimaryContainer = PrimaryHover,
    secondary = Primary,
    onSecondary = Color.White,
    background = Background,
    onBackground = TextPrimary,
    surface = Surface,
    onSurface = TextPrimary,
    surfaceVariant = SurfaceAlt,
    onSurfaceVariant = TextSecondary,
    surfaceTint = Primary,
    outline = Border,
    outlineVariant = BorderSoft,
    error = Danger,
    onError = Color.White
)

/**
 * 豆包风格尺寸 token：圆角与间距，与桌面/iOS/HarmonyOS 端一致。
 */
object NghDimensions {
    val radiusSm = 8.dp
    val radiusMd = 12.dp
    val radiusLg = 16.dp
    val radiusPill = 999.dp

    val spacing1 = 4.dp
    val spacing2 = 8.dp
    val spacing3 = 12.dp
    val spacing4 = 16.dp
    val spacing5 = 20.dp
    val spacing6 = 24.dp
    val spacing7 = 32.dp
    val spacing8 = 40.dp
}

@Composable
fun MusicPlayerTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit
) {
    // 豆包风格仅提供浅色方案
    MaterialTheme(
        colorScheme = LightColorScheme,
        typography = AppTypography,
        content = content
    )
}

/**
 * 豆包风格点击缩放反馈：按下 scale 0.97，松开回弹 1.0，tween(150, EaseOut)。
 * 仅视觉反馈（无 ripple），保持列表卡片克制观感；点击通过 onTap 触发回调。
 */
fun Modifier.nghClickableScale(onClick: () -> Unit): Modifier = composed {
    var pressed by remember { mutableStateOf(false) }
    val scale by animateFloatAsState(
        targetValue = if (pressed) 0.97f else 1f,
        animationSpec = tween(durationMillis = 150, easing = EaseOut),
        label = "nghClickableScale"
    )
    this
        .graphicsLayer {
            scaleX = scale
            scaleY = scale
        }
        // 7.1 修复：以 onClick 作为 key，lambda 在 onClick 变化时重启，捕获最新回调；
        //      原 pointerInput(Unit) 仅创建一次，会一直调用首次的 onClick 闭包。
        .pointerInput(onClick) {
            detectTapGestures(
                onPress = {
                    pressed = true
                    tryAwaitRelease()
                    pressed = false
                },
                onTap = { onClick() }
            )
        }
}
