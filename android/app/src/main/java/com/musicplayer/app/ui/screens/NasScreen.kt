// 职责：NAS / 协议音源浏览页（Android）：
// - 飞牛 NAS：健康检查、登录、目录浏览、点击音频文件经 feiniu_stream 拉流播放。
// - 协议源：添加 / 列出 / 删除、浏览目录、点击音频文件经 protocol_stream 拉流播放。
// 重试策略遵循 docs：502/504 指数退避（1s/2s/4s，最多 3 次），401 提示重新登录，
// 404 修正路径，501 占位提示（SMB/DLNA/NFS）。
// Material 3 组件 + Material Icons（无 emoji），豆包风格 token（NghDimensions / nghClickableScale）。

package com.musicplayer.app.ui.screens

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.MusicNote
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.filled.Storage
import androidx.compose.material3.Button
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.google.gson.Gson
import com.google.gson.JsonSyntaxException
import com.musicplayer.app.models.Song
import com.musicplayer.app.models.SongOrigin
import com.musicplayer.app.player.PlayerManager
import com.musicplayer.app.repository.MusicRepository
import com.musicplayer.app.ui.theme.NghDimensions
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch

// ---- 本地 JSON 解析数据类（MusicRepository 飞牛/协议方法返回原始 JSON）----

private data class NasFileDto(
    val name: String? = null,
    val is_dir: Boolean? = null,
    val isDir: Boolean? = null,
    val size: Long? = null,
    val modified: String? = null
) {
    val isDirValue: Boolean get() = is_dir ?: isDir ?: false
    val sizeValue: Long get() = size ?: 0L
}

private data class FeiniuFilesResp(val path: String? = null, val files: List<NasFileDto>? = null)
private data class StreamResp(val url: String? = null)
private data class HealthResp(val healthy: Boolean? = null, val base_url: String? = null)
private data class ProtocolSourceDto(
    val id: String? = null,
    val protocol: String? = null,
    val root: String? = null,
    val enabled: Boolean? = null,
    val placeholder: Boolean? = null
)
private data class ProtocolListResp(val sources: List<ProtocolSourceDto>? = null)
private data class ProtocolListFilesResp(val path: String? = null, val entries: List<String>? = null)

private val nasGson = Gson()

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NasScreen(player: PlayerManager) {
    val scope = rememberCoroutineScope()
    val scroll = rememberScrollState()

    var mode by remember { mutableStateOf("feiniu") }
    var error by remember { mutableStateOf<String?>(null) }
    var info by remember { mutableStateOf<String?>(null) }

    // 飞牛状态
    var baseUrl by remember { mutableStateOf("") }
    var username by remember { mutableStateOf("") }
    var password by remember { mutableStateOf("") }
    var healthText by remember { mutableStateOf("未检测") }
    var healthOk by remember { mutableStateOf<Boolean?>(null) }
    var feiniuPath by remember { mutableStateOf("/") }
    var feiniuFiles by remember { mutableStateOf<List<NasFileDto>>(emptyList()) }
    var feiniuLoading by remember { mutableStateOf(false) }

    // 协议源状态
    var protocolSources by remember { mutableStateOf<List<ProtocolSourceDto>>(emptyList()) }
    var selectedSourceId by remember { mutableStateOf<String?>(null) }
    var protocolPath by remember { mutableStateOf("/") }
    var protocolEntries by remember { mutableStateOf<List<String>>(emptyList()) }
    var protocolLoading by remember { mutableStateOf(false) }
    var showAddSheet by remember { mutableStateOf(false) }
    var newSourceJson by remember { mutableStateOf("") }

    Column(
        Modifier
            .fillMaxSize()
            .verticalScroll(scroll)
            .padding(NghDimensions.spacing4)
    ) {
        Text("NAS / 协议音源", fontSize = 22.sp, fontWeight = FontWeight.Medium)
        Text("飞牛 NAS 与网络协议源浏览播放", fontSize = 12.sp, color = MaterialTheme.colorScheme.outline)
        Spacer(Modifier.height(NghDimensions.spacing4))

        // 模式切换
        Row(horizontalArrangement = Arrangement.spacedBy(NghDimensions.spacing2)) {
            FilterChip(
                selected = mode == "feiniu",
                onClick = { mode = "feiniu" },
                label = { Text("飞牛 NAS") },
                leadingIcon = { Icon(Icons.Filled.Storage, null, Modifier.size(16.dp)) }
            )
            FilterChip(
                selected = mode == "protocol",
                onClick = {
                    mode = "protocol"
                    scope.launch { loadProtocolSources({ protocolSources = it }, { error = it }) }
                },
                label = { Text("协议源") },
                leadingIcon = { Icon(Icons.Filled.Folder, null, Modifier.size(16.dp)) }
            )
        }
        Spacer(Modifier.height(NghDimensions.spacing4))

        if (mode == "feiniu") {
            // ---- 健康检查 ----
            Card(modifier = Modifier.fillMaxWidth()) {
                Row(
                    Modifier.padding(NghDimensions.spacing3),
                    verticalAlignment = Alignment.CenterVertically
                ) {
                    Text(
                        if (healthOk == true) "✓" else if (healthOk == false) "×" else "?",
                        color = if (healthOk == true) Color(0xFF00B96B)
                        else if (healthOk == false) MaterialTheme.colorScheme.error
                        else MaterialTheme.colorScheme.outline
                    )
                    Spacer(Modifier.width(NghDimensions.spacing2))
                    Text(healthText, fontSize = 13.sp, color = MaterialTheme.colorScheme.outline)
                    Spacer(Modifier.weight(1f))
                    Button(
                        onClick = {
                            scope.launch {
                                healthText = "检测中…"
                                healthOk = null
                                try {
                                    val raw = retry { MusicRepository.feiniuHealth() }
                                    val resp = nasGson.fromJsonSafe<HealthResp>(raw, HealthResp::class.java)
                                    val ok = resp?.healthy == true
                                    healthOk = ok
                                    healthText = if (ok) "飞牛服务可达" else "飞牛服务不可达"
                                } catch (e: Exception) {
                                    healthOk = false
                                    healthText = "健康检查失败"
                                    error = formatError(e)
                                }
                            }
                        }
                    ) { Text("健康检查", fontSize = 13.sp) }
                }
            }
            Spacer(Modifier.height(NghDimensions.spacing3))

            // ---- 登录表单 ----
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(NghDimensions.spacing3)) {
                    Text("飞牛登录", fontSize = 16.sp, fontWeight = FontWeight.SemiBold)
                    Spacer(Modifier.height(NghDimensions.spacing2))
                    OutlinedTextField(
                        value = baseUrl, onValueChange = { baseUrl = it },
                        label = { Text("服务地址") },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true
                    )
                    Spacer(Modifier.height(NghDimensions.spacing2))
                    OutlinedTextField(
                        value = username, onValueChange = { username = it },
                        label = { Text("用户名") },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true
                    )
                    Spacer(Modifier.height(NghDimensions.spacing2))
                    OutlinedTextField(
                        value = password, onValueChange = { password = it },
                        label = { Text("密码") },
                        modifier = Modifier.fillMaxWidth(),
                        singleLine = true,
                        visualTransformation = androidx.compose.ui.text.input.PasswordVisualTransformation()
                    )
                    Spacer(Modifier.height(NghDimensions.spacing3))
                    Button(
                        onClick = {
                            val b = baseUrl.trim()
                            val u = username.trim()
                            if (b.isEmpty() || u.isEmpty()) {
                                error = "请填写服务地址与用户名"; return@Button
                            }
                            scope.launch {
                                feiniuLoading = true
                                try {
                                    MusicRepository.feiniuLogin(b, u, password)
                                    feiniuPath = "/"
                                    refreshFeiniu(
                                        path = feiniuPath,
                                        loading = { feiniuLoading = it },
                                        setFiles = { feiniuFiles = it },
                                        setError = { error = it }
                                    )
                                    info = "登录成功"
                                } catch (e: Exception) {
                                    if (isStatus(e, 401)) error = "用户名或密码错误（401），请检查凭据后重试。"
                                    else error = "登录失败：" + formatError(e)
                                }
                                feiniuLoading = false
                            }
                        },
                        modifier = Modifier.fillMaxWidth()
                    ) { Text("登录") }
                }
            }
            Spacer(Modifier.height(NghDimensions.spacing3))

            // ---- 浏览区 ----
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(NghDimensions.spacing3)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        IconButton(
                            onClick = {
                                feiniuPath = parentPath(feiniuPath)
                                scope.launch {
                                    refreshFeiniu(feiniuPath, { feiniuLoading = it }, { feiniuFiles = it }, { error = it })
                                }
                            },
                            enabled = feiniuPath != "/" && feiniuPath.isNotEmpty()
                        ) { Icon(Icons.AutoMirrored.Filled.ArrowBack, "返回") }
                        Text(feiniuPath, fontSize = 13.sp, color = MaterialTheme.colorScheme.outline, modifier = Modifier.weight(1f))
                        IconButton(onClick = {
                            scope.launch {
                                refreshFeiniu(feiniuPath, { feiniuLoading = it }, { feiniuFiles = it }, { error = it })
                            }
                        }) { Icon(Icons.Filled.Refresh, "刷新") }
                    }
                    if (feiniuLoading) {
                        Box(Modifier.fillMaxWidth().padding(NghDimensions.spacing3), contentAlignment = Alignment.Center) {
                            CircularProgressIndicator(Modifier.size(22.dp))
                        }
                    }
                    if (feiniuFiles.isEmpty() && !feiniuLoading) {
                        Text("未列出文件，登录后刷新", fontSize = 13.sp, color = MaterialTheme.colorScheme.outline,
                            modifier = Modifier.padding(NghDimensions.spacing3))
                    } else {
                        // 7.3 改用 LazyColumn + items()，提升长列表性能；因父级使用 verticalScroll，
                        //     这里以 heightIn 限定最大高度，避免无限高度约束崩溃。
                        LazyColumn(modifier = Modifier.fillMaxWidth().heightIn(max = 480.dp)) {
                            items(feiniuFiles) { f ->
                                NasFileRow(
                                    name = f.name ?: "",
                                    isDir = f.isDirValue,
                                    size = f.sizeValue,
                                    onClick = {
                                        if (f.isDirValue) {
                                            feiniuPath = joinPath(feiniuPath, f.name ?: "", true)
                                            scope.launch {
                                                refreshFeiniu(feiniuPath, { feiniuLoading = it }, { feiniuFiles = it }, { error = it })
                                            }
                                        } else if (isAudio(f.name ?: "")) {
                                            scope.launch {
                                                feiniuLoading = true
                                                val full = joinPath(feiniuPath, f.name ?: "", false)
                                                try {
                                                    val raw = retry { MusicRepository.feiniuStream(full) }
                                                    val url = nasGson.fromJsonSafe<StreamResp>(raw, StreamResp::class.java)?.url
                                                    if (url.isNullOrEmpty()) { error = "未获取到播放地址"; return@launch }
                                                    val song = Song(
                                                        id = "feiniu-" + (f.name ?: ""),
                                                        sourceId = "feiniu",
                                                        title = f.name ?: "",
                                                        artists = emptyList(),
                                                        playUrl = url,
                                                        origin = SongOrigin.Nas("feiniu", url)
                                                    )
                                                    // 4.5 传入 queue = listOf(song)，使该曲成为播放队列，
                                                    // 避免因队列为空导致切歌/单曲循环失效。
                                                    player.play(song, queue = listOf(song))
                                                    info = "开始播放：" + (f.name ?: "")
                                                } catch (e: Exception) {
                                                    error = "获取播放地址失败：" + formatError(e)
                                                }
                                                feiniuLoading = false
                                            }
                                        }
                                    }
                                )
                            }
                        }
                    }
                }
            }
        } else {
            // ---- 协议源管理 ----
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(Modifier.padding(NghDimensions.spacing3)) {
                    Row(verticalAlignment = Alignment.CenterVertically) {
                        Text("协议源", fontSize = 16.sp, fontWeight = FontWeight.SemiBold, modifier = Modifier.weight(1f))
                        IconButton(onClick = {
                            scope.launch { loadProtocolSources({ protocolSources = it }, { error = it }) }
                        }) { Icon(Icons.Filled.Refresh, "刷新") }
                        Button(onClick = { showAddSheet = true }) {
                            Icon(Icons.Filled.Add, null, Modifier.size(16.dp))
                            Spacer(Modifier.width(NghDimensions.spacing1))
                            Text("添加", fontSize = 13.sp)
                        }
                    }
                    if (protocolSources.isEmpty()) {
                        Text("尚无协议源，点击「添加」创建（WebDAV / FTP 可用，SMB / DLNA / NFS 为占位）",
                            fontSize = 12.sp, color = MaterialTheme.colorScheme.outline,
                            modifier = Modifier.padding(NghDimensions.spacing2))
                    } else {
                        protocolSources.forEach { src ->
                            ProtocolSourceRow(
                                id = src.id ?: "",
                                protocol = src.protocol ?: "",
                                root = src.root ?: "",
                                placeholder = src.placeholder == true,
                                onBrowse = {
                                    selectedSourceId = src.id
                                    if (src.placeholder != true) {
                                        protocolPath = if (src.root.isNullOrEmpty()) "/" else src.root
                                        scope.launch {
                                            refreshProtocolEntries(
                                                src.id ?: "", protocolPath,
                                                { protocolLoading = it }, { protocolEntries = it }, { error = it }
                                            )
                                        }
                                    }
                                },
                                onDelete = {
                                    scope.launch {
                                        try {
                                            MusicRepository.protocolDelete(src.id ?: "")
                                            if (selectedSourceId == src.id) selectedSourceId = null
                                            loadProtocolSources({ protocolSources = it }, { error = it })
                                            info = "已删除协议源 " + (src.id ?: "")
                                        } catch (e: Exception) {
                                            error = "删除失败：" + formatError(e)
                                        }
                                    }
                                }
                            )
                        }
                    }
                }
            }
            Spacer(Modifier.height(NghDimensions.spacing3))

            // ---- 协议浏览 ----
            val selId = selectedSourceId
            val sel = protocolSources.firstOrNull { it.id == selId }
            if (sel != null) {
                Card(modifier = Modifier.fillMaxWidth()) {
                    Column(Modifier.padding(NghDimensions.spacing3)) {
                        Text("当前协议源：${sel.protocol} · ${sel.id}", fontSize = 15.sp, fontWeight = FontWeight.SemiBold)
                        Spacer(Modifier.height(NghDimensions.spacing2))
                        if (sel.placeholder == true) {
                            Text("${sel.protocol} 为占位实现，需启用对应 feature，建议使用 WebDAV / FTP",
                                fontSize = 12.sp, color = MaterialTheme.colorScheme.error)
                        } else {
                            Row(verticalAlignment = Alignment.CenterVertically) {
                                IconButton(
                                    onClick = {
                                        protocolPath = parentPath(protocolPath)
                                        scope.launch {
                                            refreshProtocolEntries(sel.id ?: "", protocolPath,
                                                { protocolLoading = it }, { protocolEntries = it }, { error = it })
                                        }
                                    },
                                    enabled = protocolPath != "/" && protocolPath.isNotEmpty()
                                ) { Icon(Icons.AutoMirrored.Filled.ArrowBack, "返回") }
                                Text(protocolPath, fontSize = 13.sp, color = MaterialTheme.colorScheme.outline, modifier = Modifier.weight(1f))
                                IconButton(onClick = {
                                    scope.launch {
                                        refreshProtocolEntries(sel.id ?: "", protocolPath,
                                            { protocolLoading = it }, { protocolEntries = it }, { error = it })
                                    }
                                }) { Icon(Icons.Filled.Refresh, "刷新") }
                            }
                            if (protocolLoading) {
                                Box(Modifier.fillMaxWidth().padding(NghDimensions.spacing3), contentAlignment = Alignment.Center) {
                                    CircularProgressIndicator(Modifier.size(22.dp))
                                }
                            }
                            if (protocolEntries.isEmpty() && !protocolLoading) {
                                Text("未列出条目，点击刷新", fontSize = 13.sp, color = MaterialTheme.colorScheme.outline,
                                    modifier = Modifier.padding(NghDimensions.spacing2))
                            } else {
                                protocolEntries.forEach { rawName ->
                                    val isDir = rawName.endsWith("/")
                                    val name = if (isDir) rawName.dropLast(1) else rawName
                                    NasFileRow(
                                        name = name, isDir = isDir, size = 0L,
                                        onClick = {
                                            if (isDir) {
                                                protocolPath = joinPath(protocolPath, name, true)
                                                scope.launch {
                                                    refreshProtocolEntries(sel.id ?: "", protocolPath,
                                                        { protocolLoading = it }, { protocolEntries = it }, { error = it })
                                                }
                                            } else if (isAudio(name)) {
                                                scope.launch {
                                                    protocolLoading = true
                                                    val full = joinPath(protocolPath, name, false)
                                                    try {
                                                        val raw = retry { MusicRepository.protocolStream(sel.id ?: "", full) }
                                                        val url = nasGson.fromJsonSafe<StreamResp>(raw, StreamResp::class.java)?.url
                                                        if (url.isNullOrEmpty()) { error = "未获取到播放地址"; return@launch }
                                                        val song = Song(
                                                            id = "proto-$name",
                                                            sourceId = "protocol",
                                                            title = name,
                                                            artists = emptyList(),
                                                            playUrl = url,
                                                            origin = SongOrigin.Nas("protocol", url)
                                                        )
                                                        // 4.5 传入 queue = listOf(song)，使该曲成为播放队列。
                                                        player.play(song, queue = listOf(song))
                                                        info = "开始播放：$name"
                                                    } catch (e: Exception) {
                                                        error = "获取播放地址失败：" + formatError(e)
                                                    }
                                                    protocolLoading = false
                                                }
                                            }
                                        }
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
        Spacer(Modifier.height(NghDimensions.spacing6))
    }

    // 错误 / 信息提示（简易对话框）
    error?.let {
        SimpleDialog(title = "错误", message = it, onDismiss = { error = null })
    }
    info?.let {
        SimpleDialog(title = "逆光音乐", message = it, onDismiss = { info = null })
    }
    if (showAddSheet) {
        AddSourceSheet(
            json = newSourceJson,
            onJsonChange = { newSourceJson = it },
            onDismiss = { showAddSheet = false; newSourceJson = "" },
            onConfirm = {
                scope.launch {
                    try {
                        MusicRepository.protocolAdd(newSourceJson.trim())
                        showAddSheet = false
                        newSourceJson = ""
                        loadProtocolSources({ protocolSources = it }, { error = it })
                        info = "协议源已添加"
                    } catch (e: Exception) {
                        error = "添加协议源失败：" + formatError(e)
                    }
                }
            }
        )
    }
}

// ---- 子组件 ----

@Composable
private fun NasFileRow(name: String, isDir: Boolean, size: Long, onClick: () -> Unit) {
    // 7.6 移除 nghClickableScale 修饰符，避免与 Surface(onClick=...) 双重点击处理；
    //     点击逻辑统一由 Surface 的 onClick 承担。
    Surface(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(NghDimensions.radiusMd)
    ) {
        Row(
            Modifier.padding(horizontal = NghDimensions.spacing3, vertical = NghDimensions.spacing2),
            verticalAlignment = Alignment.CenterVertically
        ) {
            Icon(
                if (isDir) Icons.Filled.Folder else Icons.Filled.MusicNote,
                null, Modifier.size(20.dp),
                tint = MaterialTheme.colorScheme.primary
            )
            Spacer(Modifier.width(NghDimensions.spacing3))
            Column(Modifier.weight(1f)) {
                Text(name, fontSize = 14.sp, fontWeight = FontWeight.Medium)
            }
            if (!isDir && size > 0) {
                Text(formatBytes(size), fontSize = 11.sp, color = MaterialTheme.colorScheme.outline)
            }
        }
    }
}

@Composable
private fun ProtocolSourceRow(
    id: String, protocol: String, root: String, placeholder: Boolean,
    onBrowse: () -> Unit, onDelete: () -> Unit
) {
    Row(
        Modifier.fillMaxWidth().padding(vertical = NghDimensions.spacing2),
        verticalAlignment = Alignment.CenterVertically
    ) {
        Column(Modifier.weight(1f)) {
            Text("$protocol · $id", fontSize = 14.sp, fontWeight = FontWeight.Medium)
            if (placeholder) {
                Text("占位实现，浏览不可用", fontSize = 11.sp, color = MaterialTheme.colorScheme.error)
            } else {
                Text(root.ifEmpty { "/" }, fontSize = 11.sp, color = MaterialTheme.colorScheme.outline)
            }
        }
        IconButton(onClick = onBrowse) { Icon(Icons.Filled.Folder, "浏览") }
        IconButton(onClick = onDelete) { Icon(Icons.Filled.Delete, "删除", tint = MaterialTheme.colorScheme.error) }
    }
}

@Composable
private fun AddSourceSheet(
    json: String, onJsonChange: (String) -> Unit,
    onDismiss: () -> Unit, onConfirm: () -> Unit
) {
    androidx.compose.material3.AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("添加协议源") },
        text = {
            Column {
                Text("粘贴协议源配置 JSON（参考 docs/api/protocol-api.md）",
                    fontSize = 11.sp, color = MaterialTheme.colorScheme.outline)
                Spacer(Modifier.height(NghDimensions.spacing2))
                OutlinedTextField(
                    value = json, onValueChange = onJsonChange,
                    modifier = Modifier.fillMaxWidth().height(180.dp),
                    placeholder = { Text("""{"protocol":"webdav","auth":{...}}""", fontSize = 11.sp) }
                )
            }
        },
        confirmButton = {
            TextButton(onClick = onConfirm, enabled = json.isNotBlank()) { Text("添加") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("取消") } }
    )
}

@Composable
private fun SimpleDialog(title: String, message: String, onDismiss: () -> Unit) {
    androidx.compose.material3.AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text(title) },
        text = { Text(message) },
        confirmButton = { TextButton(onClick = onDismiss) { Text("好") } }
    )
}

// ---- 业务逻辑（suspend）----

private suspend fun refreshFeiniu(
    path: String,
    loading: (Boolean) -> Unit,
    setFiles: (List<NasFileDto>) -> Unit,
    setError: (String) -> Unit
) {
    loading(true)
    setFiles(emptyList())
    try {
        val raw = retry { MusicRepository.feiniuListFiles(path) }
        val resp = nasGson.fromJsonSafe<FeiniuFilesResp>(raw, FeiniuFilesResp::class.java)
        val files = resp?.files ?: emptyList()
        setFiles(files.sortedWith(compareBy<NasFileDto> { if (it.isDirValue) 0 else 1 }.thenBy { it.name }))
    } catch (e: Exception) {
        setFiles(emptyList())
        setError("列目录失败：" + formatError(e))
    }
    loading(false)
}

private suspend fun loadProtocolSources(
    setSources: (List<ProtocolSourceDto>) -> Unit,
    setError: (String) -> Unit
) {
    try {
        val raw = MusicRepository.protocolList()
        val resp = nasGson.fromJsonSafe<ProtocolListResp>(raw, ProtocolListResp::class.java)
        setSources(resp?.sources ?: emptyList())
    } catch (e: Exception) {
        setSources(emptyList())
        setError("加载协议源失败：" + formatError(e))
    }
}

private suspend fun refreshProtocolEntries(
    id: String, path: String,
    loading: (Boolean) -> Unit,
    setEntries: (List<String>) -> Unit,
    setError: (String) -> Unit
) {
    loading(true)
    setEntries(emptyList())
    try {
        val raw = retry { MusicRepository.protocolListFiles(id, path) }
        val resp = nasGson.fromJsonSafe<ProtocolListFilesResp>(raw, ProtocolListFilesResp::class.java)
        val entries = resp?.entries ?: emptyList()
        setEntries(entries.sortedWith(compareBy<String> { if (it.endsWith("/")) 0 else 1 }.thenBy { it }))
    } catch (e: Exception) {
        setEntries(emptyList())
        setError("浏览失败：" + formatError(e))
    }
    loading(false)
}

/** 对 502/504 类网络错误做指数退避重试（1s/2s/4s，最多 3 次）。 */
private suspend fun <T> retry(block: suspend () -> T?): T? {
    val delays = longArrayOf(1000, 2000, 4000)
    var last: Exception? = null
    for (attempt in 0..3) {
        try {
            return block()
        } catch (e: Exception) {
            last = e
            if (!isRetryable(e) || attempt == 3) break
            delay(delays[attempt])
        }
    }
    throw last ?: RuntimeException("重试后仍失败")
}

private fun isRetryable(e: Throwable): Boolean {
    val msg = e.message ?: ""
    return msg.contains("502") || msg.contains("504") ||
            msg.contains("不可达") || msg.contains("请求失败")
}

private fun isStatus(e: Throwable, status: Int): Boolean = (e.message ?: "").contains(status.toString())

private fun formatError(e: Throwable): String {
    val msg = e.message ?: "未知错误"
    if (isStatus(e, 401)) return msg + "\n提示：未登录或 token 失效，请重新登录。"
    if (isStatus(e, 404)) return msg + "\n提示：路径不存在，请修正路径。"
    if (isStatus(e, 501)) return msg + "\n提示：该协议为占位实现，请使用 WebDAV / FTP 或启用对应 feature。"
    return msg
}

// ---- 工具 ----

private fun joinPath(parent: String, name: String, isDir: Boolean): String {
    val p = if (parent.endsWith("/")) parent.dropLast(1) else parent
    val n = if (name.startsWith("/")) name.drop(1) else name
    var full = if (p.isEmpty()) "/$n" else "$p/$n"
    if (isDir && !full.endsWith("/")) full += "/"
    return full
}

private fun parentPath(path: String): String {
    if (path.isEmpty() || path == "/") return "/"
    var p = if (path.endsWith("/")) path.dropLast(1) else path
    if (p.isEmpty()) return "/"
    val idx = p.lastIndexOf('/')
    return when {
        idx < 0 -> "/"
        idx == 0 -> "/"
        else -> p.substring(0, idx)
    }
}

private fun isAudio(name: String): Boolean {
    val exts = listOf(".mp3", ".flac", ".wav", ".m4a", ".aac", ".ogg", ".opus", ".wma")
    val lower = name.lowercase()
    return exts.any { lower.endsWith(it) }
}

private fun formatBytes(bytes: Long): String {
    val units = arrayOf("B", "KB", "MB", "GB", "TB")
    var v = bytes.toDouble()
    var i = 0
    while (v >= 1024 && i < units.size - 1) { v /= 1024; i++ }
    // 7.7 使用 Locale.US，避免在德语等 locale 下输出逗号小数点导致解析异常。
    return String.format(java.util.Locale.US, "%.2f %s", v, units[i])
}

private inline fun <reified T> Gson.fromJsonSafe(json: String?, clazz: Class<T>): T? {
    if (json.isNullOrBlank()) return null
    return try { fromJson(json, clazz) } catch (e: JsonSyntaxException) { null }
}
