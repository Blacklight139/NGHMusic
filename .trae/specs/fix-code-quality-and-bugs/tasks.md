# Tasks

## 阶段一：Rust 核心修复
- [x] Task 1: 修复缓存键碰撞与 FFI 安全问题
  - [x] 1.1 修复 `core/src/cache.rs` sanitize_key 使用哈希替代简单字符替换，防止缓存键碰撞
  - [x] 1.2 修复 `core/src/ffi.rs` FTP 端口截断：校验 port <= 65535，超范围返回错误
  - [x] 1.3 修复 `core/src/ffi.rs` 序列号递增使用 wrapping_add 防止溢出
  - [x] 1.4 修复 `core/src/library.rs` 整数转换添加范围校验（i64→u64 检查 >= 0，u64→i64 检查 <= i64::MAX）
- [x] Task 2: 修复 FFI 并发与错误处理问题
  - [x] 2.1 修复 `core/src/ffi.rs` 本地扫描持有锁阻塞进度查询：锁内 Arc::clone 后释放锁，锁外执行扫描
  - [x] 2.2 修复 `core/src/ffi.rs` 飞牛登录锁中毒时返回错误而非静默吞错
  - [x] 2.3 修复 `core/src/sources/mod.rs` 持久化失败不再静默忽略，至少 log::error! 记录
  - [x] 2.4 修复 `core/src/sources/local.rs` 扫描时单个文件索引失败添加 log::warn!
- [x] Task 3: 修复核心逻辑与性能问题
  - [x] 3.1 修复 `core/src/sources/local.rs` 搜索使用 SQL LIMIT/OFFSET 分页替代全量加载
  - [x] 3.2 修复 `core/src/sources/local.rs` 艺术家名分隔符使用多字符分隔符或 JSON 数组存储，防止 roundtrip 损坏
  - [x] 3.3 修复 `core/src/ffi.rs` music_core_get_play_url 集成 CacheManager 检查缓存命中
  - [x] 3.4 修复 `core/src/ffi.rs` music_core_search 使用并发查询（join_all）替代顺序执行

## 阶段二：Android 端修复
- [x] Task 4: 修复播放器核心功能 bug
  - [x] 4.1 修复 `PlayerManager.kt` 自动切歌：在 onPlaybackStateChanged 中处理 STATE_ENDED
  - [x] 4.2 修复 `PlayerManager.kt` 本地文件 URI 构造：对本地路径使用 Uri.fromFile 或添加 file:// 前缀
  - [x] 4.3 修复 `PlayerManager.kt` play/applyMediaItem 与 onCleared 竞态：在锁内重新读取 player 引用
  - [x] 4.4 修复 `PlayerManager.kt` URL 为空时不设置 currentSong 或设置后恢复 isPlaying
  - [x] 4.5 修复 `NasScreen.kt` 播放时传入 queue = listOf(song)
- [x] Task 5: 修复线程安全与并发问题
  - [x] 5.1 修复 `MusicCoreBridge.kt` placeholderSources 改用 CopyOnWriteArrayList
  - [x] 5.2 修复 `MusicCoreBridge.kt` placeholderSeq 改用 AtomicInteger
  - [x] 5.3 修复 `SearchScreen.kt` 搜索使用 Job 变量取消旧请求
  - [x] 5.4 修复 `SettingsScreen.kt` 音源操作串行化
- [x] Task 6: 修复空安全与错误处理
  - [x] 6.1 修复 `MusicRepository.kt` SongOriginTypeAdapter.read 的 else 分支返回默认值而非 null
  - [x] 6.2 修复 `MusicRepository.kt` parse/parseList 捕获 Exception 而非仅 JsonSyntaxException
  - [x] 6.3 修复 `SettingsScreen.kt` 删除音源添加 try-catch，失败时 snackbar 提示
  - [x] 6.4 修复 `SettingsScreen.kt` 文件读取使用 withContext(Dispatchers.IO)
  - [x] 6.5 修复 `MusicRepository.kt` inferElementType 使用 JsonParser 解析首元素而非 String.contains
- [x] Task 7: 修复 Compose 与 UI 问题
  - [x] 7.1 修复 `Theme.kt` nghClickableScale 的 pointerInput 使用 onClick 作为 key
  - [x] 7.2 修复各列表添加 key 参数（SearchScreen/PlaylistScreen/LocalMusicScreen/FavoritesScreen/SettingsScreen）
  - [x] 7.3 修复 `NasScreen.kt` 文件列表改用 LazyColumn
  - [x] 7.4 修复 `LyricsScreen.kt` null timeMs 行跳过高亮
  - [x] 7.5 修复 `LeaderboardScreen.kt`/`LocalMusicScreen.kt` 加载失败显示错误状态而非静默回退占位数据
  - [x] 7.6 修复 `NasScreen.kt` Surface onClick 与 nghClickableScale 点击冲突
  - [x] 7.7 修复 `NasScreen.kt` formatBytes 使用 Locale.US

## 阶段三：iOS 端修复
- [x] Task 8: 修复播放器核心功能 bug
  - [x] 8.1 修复 `PlayerManager.swift` play(song:in:) 队列索引：song 不在队列时设 queue = [song], index = 0
  - [x] 8.2 修复 `PlayerManager.swift` KVO 处理 .failed 状态时置 isPlaying = false
  - [x] 8.3 修复 `PlayerManager.swift` 初始化时配置 AVAudioSession
  - [x] 8.4 修复 `PlayerManager.swift` volume 添加 didSet 同步到 player.volume
  - [x] 8.5 修复 `PlayerManager.swift` KVO 校验 observedItem === player.currentItem
- [x] Task 9: 修复 UI 与并发问题
  - [x] 9.1 修复 `LyricsView.swift` 无时间戳行跳过高亮（参考 macOS guard let 写法）
  - [x] 9.2 修复 `NasView.swift` 异步修改 @State 包裹 MainActor.run
  - [x] 9.3 修复 `SearchView.swift` 搜索 Task 保存句柄并在新搜索前 cancel
  - [x] 9.4 修复 `NasView.swift` retry 中 try? Task.sleep 改为 try Task.sleep 保留取消语义
  - [x] 9.5 修复 `NasView.swift` Song id 拼入完整路径防止同名冲突
  - [x] 9.6 修复 `PlayerManager.swift` toNext() 随机模式排除当前索引
  - [x] 9.7 修复 `SettingsView.swift` reorderSources no-op 时不做乐观 UI 更新

## 阶段四：macOS 端修复
- [x] Task 10: 修复播放器与数据模型问题
  - [x] 10.1 修复 `PlayerService.swift` play(at:) 校验 load 成功后再置 isPlaying = true
  - [x] 10.2 修复 `PlayerService.swift` volume didSet 添加防抖减少写盘频率
  - [x] 10.3 修复 `Models.swift` SourceImportResult CodingKey sourceFormat 改为 "source_format"
  - [x] 10.4 修复 `Models.swift` NasFile CodingKeys isDir 改为 "is_dir"
- [x] Task 11: 修复 UI 与状态问题
  - [x] 11.1 修复 `LocalMusicView.swift` removeDirectory 调用核心接口或标记不可用，loadInitial 从核心回填
  - [x] 11.2 修复 `SearchView.swift` 搜索 Task 保存句柄并在新搜索前 cancel
  - [x] 11.3 修复 `LyricsView.swift` activeLineIndex 缓存为 @State 避免 O(n²) 渲染
  - [x] 11.4 修复 `PlayerService.swift` restoreState 实现完整状态恢复或移除无用字段

## 阶段五：Windows 端修复
- [x] Task 12: 修复 P/Invoke 与播放器问题
  - [x] 12.1 修复 `MusicCoreNative.cs` P/Invoke 字符串编码改用 UTF-8 替代 ANSI，防止中文乱码
  - [x] 12.2 修复 `PlayerService.cs` MediaPlayer 播放结束自动切歌逻辑（防止无限循环）
  - [x] 12.3 修复 `SettingsPage.xaml.cs` 音源导入 JSON 注入风险（转义或使用结构化解析）

## 阶段六：HarmonyOS 端修复
- [x] Task 13: 修复 AVPlayer 与并发问题
  - [x] 13.1 修复 `PlayerManager.ets` AVPlayer 回调在销毁时正确注销，防止泄漏
  - [x] 13.2 修复 `PlayerManager.ets` ensurePlayer 与 release 竞态条件
  - [x] 13.3 修复 `PlayerManager.ets` stateWaiter 支持多回调或使用 Promise 队列
  - [x] 13.4 修复 `NasPage.ets` busy-wait 改用异步等待，不阻塞 UI 线程

## 阶段七：Linux 端修复
- [x] Task 14: 修复致命崩溃与核心功能 bug
  - [x] 14.1 修复 `favorites.rs` RefCell 重入 panic：克隆数据后释放借用再调用 select_row
  - [x] 14.2 修复 `local_music.rs` 使用本地专用查询接口替代聚合搜索
  - [x] 14.3 修复 `settings.rs` 删除音源后刷新列表
  - [x] 14.4 修复 `core_service.rs` feiniu_login 锁中毒时返回错误
- [x] Task 15: 修复资源泄漏与性能问题
  - [x] 15.1 修复 `player_service.rs` Drop 实现中移除 position 定时器
  - [x] 15.2 修复 `playback_bar.rs`/`local_music.rs`/`lyrics.rs` 存储 SourceId 以便移除定时器
  - [x] 15.3 修复 `leaderboard.rs` 使用 Arc<Vec<Song>> 共享引用替代 O(n²) clone
  - [x] 15.4 修复 `lyrics.rs` 歌词高亮仅更新上一行和当前行
  - [x] 15.5 修复 `player_service.rs` save_state 添加防抖，seek 时不立即写盘
- [x] Task 16: 修复错误处理与配置问题
  - [x] 16.1 修复 `main.rs` 初始化日志后端（env_logger 或 env_logger + log）
  - [x] 16.2 修复 `Cargo.toml` 添加 env_logger 依赖，tokio 改用 rt-multi-thread feature
  - [x] 16.3 修复 `player_service.rs` set_state 错误添加 log::warn! 记录
  - [x] 16.4 修复 `core_service.rs` 本地扫描持有锁阻塞进度查询（锁内 clone 后释放）
  - [x] 16.5 修复 `core_service.rs` FTP 端口截断添加校验
  - [x] 16.6 修复 `player_service.rs` 随机播放使用 rand crate 或更好的 PRNG
  - [x] 16.7 修复 `theme.rs` CSS 路径使用绝对路径或 GResource 嵌入
  - [x] 16.8 修复 `player_service.rs` 实现完整播放状态恢复或移除无用持久化字段
- [x] Task 17: 修复代码重复与一致性问题
  - [x] 17.1 抽取 format_artists/format_duration 到公共 util 模块，统一时间格式
  - [x] 17.2 修复 `lyrics.rs` load_lyric 闭包重复代码
  - [x] 17.3 修复 `settings.rs`/`nas.rs` 协议源行构造重复代码
  - [x] 17.4 统一 format_size/format_bytes 精度

# Task Dependencies
- Task 1~3 可并行（Rust 核心不同模块）
- Task 4~7 可并行（Android 不同模块）
- Task 8~9 有依赖（iOS 播放器修复后 UI 修复）
- Task 10~11 有依赖（macOS 播放器修复后 UI 修复）
- Task 12 独立（Windows）
- Task 13 独立（HarmonyOS）
- Task 14~17 有依赖（Linux 致命 bug 先修，再修资源/性能，最后修重复代码）
- 各端任务之间无依赖，可全部并行
