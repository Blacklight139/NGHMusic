// 职责：Compose 字体类型 token，简约风格。

package com.musicplayer.app.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

private val Default = Typography()

val AppTypography = Typography(
    headlineSmall = Default.headlineSmall.copy(fontWeight = FontWeight.SemiBold, fontSize = 22.sp),
    titleLarge = Default.titleLarge.copy(fontWeight = FontWeight.SemiBold),
    titleMedium = Default.titleMedium.copy(fontWeight = FontWeight.Medium),
    bodyMedium = Default.bodyMedium.copy(fontFamily = FontFamily.Default, color = TextPrimary),
    labelSmall = Default.labelSmall.copy(color = TextMuted),
    labelMedium = TextStyle(
        fontWeight = FontWeight.Normal,
        fontSize = 12.sp,
        color = TextMuted
    )
)
