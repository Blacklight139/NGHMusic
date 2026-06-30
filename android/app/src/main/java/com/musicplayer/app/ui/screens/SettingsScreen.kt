// 职责：设置屏幕，核心版本、音源导入（JSON）、本地目录、缓存，简约风格占位。
// 对齐桌面端 pages/settings.js：调用 MusicCoreBridge.importSource。

package com.musicplayer.app.ui.screens

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.coroutines.launch
import com.musicplayer.app.bridge.MusicCoreBridge
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.theme.*

@Composable
fun SettingsScreen(player: PlayerManager = viewModel()) {
    var sourceJson by remember { mutableStateOf("") }
    var result by remember { mutableStateOf("") }
    var resultOk by remember { mutableStateOf(false) }
    var version by remember { mutableStateOf("（未连接）") }
    val scope = androidx.compose.runtime.rememberCoroutineScope()

    LaunchedEffect(Unit) { version = MusicCoreBridge.appVersion() }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp)
    ) {
        Text("设置", style = MaterialTheme.typography.headlineSmall, color = TextPrimary)
        Text("音源管理 / 缓存 / 本地目录",
            style = MaterialTheme.typography.labelMedium, color = TextMuted)
        Spacer(Modifier.height(16.dp))

        // 版本
        Row(Modifier.fillMaxWidth()) {
            Text("核心版本", style = MaterialTheme.typography.labelMedium, color = TextMuted)
            Spacer(Modifier.weight(1f))
            Text(version, style = MaterialTheme.typography.labelMedium, color = TextPrimary)
        }

        Spacer(Modifier.height(24.dp))
        Text("音源导入", style = MaterialTheme.typography.titleMedium, color = TextPrimary, fontWeight = FontWeight.Medium)
        Text("粘贴标准音源 JSON Schema 内容",
            style = MaterialTheme.typography.labelMedium, color = TextMuted)
        Spacer(Modifier.height(8.dp))

        OutlinedTextField(
            value = sourceJson,
            onValueChange = { sourceJson = it },
            modifier = Modifier.fillMaxWidth().height(160.dp),
            textStyle = androidx.compose.ui.text.TextStyle(
                fontSize = 12.sp,
                fontFamily = androidx.compose.ui.text.font.FontFamily.Monospace
            ),
            shape = RoundedCornerShape(8.dp)
        )
        Spacer(Modifier.height(8.dp))
        Button(
            onClick = {
                if (sourceJson.isBlank()) {
                    result = "请先粘贴音源 JSON"; resultOk = false; return@Button
                }
                scope.launch {
                    runCatching { MusicCoreBridge.importSource(sourceJson) }
                        .onSuccess { result = it; resultOk = true }
                        .onFailure { result = "（占位）导入失败：${it.message}"; resultOk = false }
                }
            },
            colors = ButtonDefaults.buttonColors(containerColor = Primary)
        ) { Text("导入") }

        if (result.isNotEmpty()) {
            Spacer(Modifier.height(8.dp))
            Surface(
                shape = RoundedCornerShape(8.dp),
                color = Bg,
                border = androidx.compose.foundation.BorderStroke(1.dp, if (resultOk) Primary else Border)
            ) {
                Text(result, modifier = Modifier.padding(12.dp),
                    fontSize = 12.sp, color = if (resultOk) Primary else Danger)
            }
        }

        Spacer(Modifier.height(24.dp))
        Text("本地音乐目录", style = MaterialTheme.typography.titleMedium, color = TextPrimary, fontWeight = FontWeight.Medium)
        Spacer(Modifier.height(8.dp))
        EmptyState("暂无目录，将在此添加 / 移除本地目录并触发扫描")

        Spacer(Modifier.height(24.dp))
        Text("播放缓存", style = MaterialTheme.typography.titleMedium, color = TextPrimary, fontWeight = FontWeight.Medium)
        Spacer(Modifier.height(8.dp))
        EmptyState("缓存容量与清理功能将在此提供")
    }
}
