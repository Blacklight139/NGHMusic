// 职责：设置屏幕，核心版本、音源管理（LXMusic 风格列表 + 文件导入）、本地目录、缓存。
// 对齐桌面端 pages/settings.js：调用 MusicCoreBridge 音源管理方法。
// 音源导入通过 ActivityResultContracts.GetContent 接收 application/json URI。

package com.musicplayer.app.ui.screens

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.expandVertically
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.DragHandle
import androidx.compose.material.icons.filled.FileUpload
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowUp
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import kotlinx.coroutines.launch
import com.musicplayer.app.bridge.MusicCoreBridge
import com.musicplayer.app.models.SourceInfo
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.ui.theme.*

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SettingsScreen(player: PlayerManager = viewModel()) {
    val scope = rememberCoroutineScope()
    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current
    var version by remember { mutableStateOf("（未连接）") }
    var sources by remember { mutableStateOf<List<SourceInfo>>(emptyList()) }
    var expandedId by remember { mutableStateOf<String?>(null) }
    var pendingDelete by remember { mutableStateOf<SourceInfo?>(null) }

    // 文件选择：接收 application/json URI -> 读取流 -> 导入 -> 刷新
    val pickJsonLauncher = rememberLauncherForActivityResult(
        contract = ActivityResultContracts.GetContent()
    ) { uri ->
        if (uri != null) {
            scope.launch {
                val json = runCatching {
                    context.contentResolver.openInputStream(uri)?.bufferedReader()?.use { it.readText() }
                }.getOrNull()
                if (json.isNullOrBlank()) {
                    snackbarHostState.showSnackbar("无法读取文件或内容为空")
                } else {
                    runCatching { MusicCoreBridge.importSourceFromJson(json) }
                        .onSuccess { info ->
                            sources = MusicCoreBridge.listSourcesOrdered()
                            snackbarHostState.showSnackbar("音源导入成功：${info.name}")
                        }
                        .onFailure {
                            snackbarHostState.showSnackbar("导入失败：${it.message ?: "未知错误"}")
                        }
                }
            }
        }
    }

    LaunchedEffect(Unit) {
        version = MusicCoreBridge.appVersion()
        sources = MusicCoreBridge.listSourcesOrdered()
    }

    Scaffold(
        snackbarHost = { SnackbarHost(snackbarHostState) },
        containerColor = Background
    ) { inner ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(inner)
                .verticalScroll(rememberScrollState())
                .padding(NghDimensions.spacing4)
        ) {
            Text("设置", style = MaterialTheme.typography.headlineSmall, color = TextPrimary)
            Text("音源管理 / 缓存 / 本地目录",
                style = MaterialTheme.typography.labelMedium, color = TextSecondary)
            Spacer(Modifier.height(NghDimensions.spacing4))

            // 版本
            Row(Modifier.fillMaxWidth()) {
                Text("核心版本", style = MaterialTheme.typography.labelMedium, color = TextSecondary)
                Spacer(Modifier.weight(1f))
                Text(version, style = MaterialTheme.typography.labelMedium, color = TextPrimary)
            }

            Spacer(Modifier.height(NghDimensions.spacing6))

            // ---- 音源管理 Section ----
            SourceManagementSection(
                sources = sources,
                expandedId = expandedId,
                onToggleExpand = { id ->
                    expandedId = if (expandedId == id) null else id
                },
                onImportClick = { pickJsonLauncher.launch("application/json") },
                onToggleEnabled = { id, enabled ->
                    scope.launch {
                        MusicCoreBridge.setSourceEnabled(id, enabled)
                        sources = MusicCoreBridge.listSourcesOrdered()
                    }
                },
                onMoveUp = { index ->
                    if (index > 0) {
                        val ordered = sources.toMutableList()
                        java.util.Collections.swap(ordered, index, index - 1)
                        scope.launch {
                            MusicCoreBridge.reorderSources(ordered.map { it.id })
                            sources = MusicCoreBridge.listSourcesOrdered()
                        }
                    }
                },
                onMoveDown = { index ->
                    if (index < sources.size - 1) {
                        val ordered = sources.toMutableList()
                        java.util.Collections.swap(ordered, index, index + 1)
                        scope.launch {
                            MusicCoreBridge.reorderSources(ordered.map { it.id })
                            sources = MusicCoreBridge.listSourcesOrdered()
                        }
                    }
                },
                onDelete = { pendingDelete = it }
            )

            Spacer(Modifier.height(NghDimensions.spacing6))
            Text("本地音乐目录", style = MaterialTheme.typography.titleMedium, color = TextPrimary, fontWeight = FontWeight.Medium)
            Spacer(Modifier.height(NghDimensions.spacing2))
            EmptyState("暂无目录，将在此添加 / 移除本地目录并触发扫描")

            Spacer(Modifier.height(NghDimensions.spacing6))
            Text("播放缓存", style = MaterialTheme.typography.titleMedium, color = TextPrimary, fontWeight = FontWeight.Medium)
            Spacer(Modifier.height(NghDimensions.spacing2))
            EmptyState("缓存容量与清理功能将在此提供")
        }
    }

    // 删除确认对话框
    pendingDelete?.let { target ->
        AlertDialog(
            onDismissRequest = { pendingDelete = null },
            title = { Text("删除音源") },
            text = { Text("确定删除音源「${target.name}」？") },
            confirmButton = {
                TextButton(onClick = {
                    val t = target
                    pendingDelete = null
                    scope.launch {
                        MusicCoreBridge.deleteSource(t.id)
                        sources = MusicCoreBridge.listSourcesOrdered()
                        snackbarHostState.showSnackbar("已删除「${t.name}」")
                    }
                }) { Text("删除", color = Danger) }
            },
            dismissButton = {
                TextButton(onClick = { pendingDelete = null }) { Text("取消") }
            }
        )
    }
}

@Composable
private fun SourceManagementSection(
    sources: List<SourceInfo>,
    expandedId: String?,
    onToggleExpand: (String) -> Unit,
    onImportClick: () -> Unit,
    onToggleEnabled: (String, Boolean) -> Unit,
    onMoveUp: (Int) -> Unit,
    onMoveDown: (Int) -> Unit,
    onDelete: (SourceInfo) -> Unit
) {
    Card(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(NghDimensions.radiusMd),
        colors = CardDefaults.cardColors(containerColor = Surface),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(Modifier.padding(NghDimensions.spacing4)) {
            // 标题行
            Row(verticalAlignment = Alignment.CenterVertically) {
                Column(Modifier.weight(1f)) {
                    Text("音源管理",
                        style = MaterialTheme.typography.titleMedium,
                        color = TextPrimary,
                        fontWeight = FontWeight.SemiBold)
                    Text("已导入 ${sources.size} 个音源",
                        style = MaterialTheme.typography.labelMedium,
                        color = TextSecondary)
                }
                IconButton(onClick = onImportClick) {
                    Icon(Icons.Filled.FileUpload, "导入音源", tint = Primary)
                }
            }

            Spacer(Modifier.height(NghDimensions.spacing2))

            if (sources.isEmpty()) {
                Box(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(vertical = NghDimensions.spacing6),
                    contentAlignment = Alignment.Center
                ) {
                    Text("暂无音源，点击右上角导入",
                        style = MaterialTheme.typography.bodyMedium,
                        color = TextSecondary)
                }
            } else {
                sources.forEachIndexed { index, item ->
                    SourceItemCard(
                        item = item,
                        index = index,
                        total = sources.size,
                        expanded = expandedId == item.id,
                        onToggleExpand = { onToggleExpand(item.id) },
                        onToggleEnabled = { onToggleEnabled(item.id, it) },
                        onMoveUp = { onMoveUp(index) },
                        onMoveDown = { onMoveDown(index) },
                        onDelete = { onDelete(item) }
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SourceItemCard(
    item: SourceInfo,
    index: Int,
    total: Int,
    expanded: Boolean,
    onToggleExpand: () -> Unit,
    onToggleEnabled: (Boolean) -> Unit,
    onMoveUp: () -> Unit,
    onMoveDown: () -> Unit,
    onDelete: () -> Unit
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = NghDimensions.spacing1)
            .nghClickableScale { onToggleExpand() },
        shape = RoundedCornerShape(NghDimensions.radiusMd),
        colors = CardDefaults.cardColors(containerColor = Surface),
        elevation = CardDefaults.cardElevation(defaultElevation = 2.dp)
    ) {
        Column(Modifier.padding(NghDimensions.spacing3)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                // 拖动手柄
                Icon(
                    Icons.Filled.DragHandle,
                    contentDescription = "拖动排序",
                    tint = TextTertiary,
                    modifier = Modifier.size(20.dp)
                )
                Spacer(Modifier.width(NghDimensions.spacing2))
                // 名称 + 来源标签
                Column(Modifier.weight(1f)) {
                    Text(
                        item.name,
                        style = MaterialTheme.typography.bodyLarge,
                        color = TextPrimary,
                        fontWeight = FontWeight.Bold,
                        maxLines = 1
                    )
                    Spacer(Modifier.height(NghDimensions.spacing1))
                    SourceTypeTag(sourceType = item.sourceType)
                }
                Spacer(Modifier.width(NghDimensions.spacing2))
                // 启用开关
                Switch(
                    checked = item.enabled,
                    onCheckedChange = onToggleEnabled,
                    colors = SwitchDefaults.colors(
                        checkedThumbColor = Surface,
                        checkedTrackColor = Primary,
                        uncheckedThumbColor = Surface,
                        uncheckedTrackColor = Border
                    )
                )
                // 上移
                IconButton(onClick = onMoveUp, enabled = index > 0) {
                    Icon(Icons.Filled.KeyboardArrowUp, "上移", tint = if (index > 0) TextPrimary else TextTertiary)
                }
                // 下移
                IconButton(onClick = onMoveDown, enabled = index < total - 1) {
                    Icon(Icons.Filled.KeyboardArrowDown, "下移", tint = if (index < total - 1) TextPrimary else TextTertiary)
                }
                // 删除
                IconButton(onClick = onDelete) {
                    Icon(Icons.Filled.Delete, "删除", tint = Danger)
                }
            }

            // 展开详情
            AnimatedVisibility(
                visible = expanded,
                enter = expandVertically(),
                exit = shrinkVertically()
            ) {
                Column(
                    modifier = Modifier
                        .padding(top = NghDimensions.spacing2)
                        .fillMaxWidth()
                ) {
                    DetailRow("ID", item.id)
                    DetailRow("版本", item.version)
                    DetailRow("类型", item.sourceType)
                    DetailRow("优先级", item.priority.toString())
                    DetailRow("启用", if (item.enabled) "是" else "否")
                    if (!item.description.isNullOrBlank()) {
                        DetailRow("说明", item.description)
                    }
                }
            }
        }
    }
}

@Composable
private fun SourceTypeTag(sourceType: String) {
    val (color, label) = when (sourceType.lowercase()) {
        "json" -> Primary to "JSON"
        "community" -> Warning to "社区"
        "local" -> Success to "本地"
        else -> TextTertiary to sourceType
    }
    Surface(
        shape = RoundedCornerShape(NghDimensions.radiusPill),
        color = color.copy(alpha = 0.12f)
    ) {
        Text(
            text = label,
            color = color,
            style = MaterialTheme.typography.labelSmall,
            fontWeight = FontWeight.Medium,
            modifier = Modifier.padding(horizontal = NghDimensions.spacing2, vertical = 2.dp)
        )
    }
}

@Composable
private fun DetailRow(label: String, value: String) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 2.dp),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Text(
            label,
            style = MaterialTheme.typography.labelMedium,
            color = TextSecondary,
            modifier = Modifier.width(64.dp)
        )
        Text(
            value,
            style = MaterialTheme.typography.bodyMedium,
            color = TextPrimary
        )
    }
}
