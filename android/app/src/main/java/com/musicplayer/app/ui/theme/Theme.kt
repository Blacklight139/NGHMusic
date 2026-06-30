// 职责：Material3 主题，简约风格，主色 #1db954。
// 集成方式：在 MainActivity.setContent 中调用 MusicPlayerTheme { ... }。

package com.musicplayer.app.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable

private val LightColorScheme = lightColorScheme(
    primary = Primary,
    onPrimary = Bg,
    primaryContainer = PrimaryDark,
    secondary = Primary,
    background = Bg,
    onBackground = TextPrimary,
    surface = Bg,
    onSurface = TextPrimary,
    surfaceVariant = BgAlt,
    onSurfaceVariant = TextMuted,
    outline = Border,
    error = Danger
)

@Composable
fun MusicPlayerTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit
) {
    // 简约风格仅提供浅色方案
    MaterialTheme(
        colorScheme = LightColorScheme,
        typography = AppTypography,
        content = content
    )
}
