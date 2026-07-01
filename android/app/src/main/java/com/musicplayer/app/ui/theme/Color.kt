// 职责：Compose 配色 token，豆包风格设计系统（逆光音乐 / NGHMusic）。
// 与桌面/iOS/HarmonyOS 端保持一致：主色 #4E6EF2，背景 #F7F8FA，文本 #1F1F1F。
// 兼容历史引用：保留 Bg / TextMuted / PrimaryDark 作为别名指向新 token。

package com.musicplayer.app.ui.theme

import androidx.compose.ui.graphics.Color

// ---- 品牌 / 主色 ----
val Primary = Color(0xFF4E6EF2)        // 柔和蓝紫
val PrimaryHover = Color(0xFF3D5AE0)
val PrimarySoft = Color(0x144E6EF2)    // 8% 透明度

// ---- 背景与表面 ----
val Background = Color(0xFFF7F8FA)
val Surface = Color(0xFFFFFFFF)
val SurfaceAlt = Color(0xFFF0F2F5)
val SidebarBackground = Color(0xFFFFFFFF)

// ---- 文本 ----
val TextPrimary = Color(0xFF1F1F1F)
val TextSecondary = Color(0xFF6B6B6B)
val TextTertiary = Color(0xFF999999)

// ---- 边框 ----
val Border = Color(0xFFEDEEF0)
val BorderSoft = Color(0xFFF5F6F8)

// ---- 状态色 ----
val Danger = Color(0xFFF5483B)
val Success = Color(0xFF00B96B)
val Warning = Color(0xFFFA8C16)

// ---- 历史别名（保持下游 Screen 编译兼容，不改变业务逻辑）----
val Bg = Background
val TextMuted = TextSecondary
val PrimaryDark = PrimaryHover
