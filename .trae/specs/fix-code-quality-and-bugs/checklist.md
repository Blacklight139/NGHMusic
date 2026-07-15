# Checklist

## Rust 核心修复
- [x] `core/src/cache.rs` sanitize_key 不再产生碰撞，不同 song_id 映射到不同缓存文件
- [x] `core/src/ffi.rs` FTP 端口超过 65535 时返回错误而非静默截断
- [x] `core/src/ffi.rs` 序列号递增使用 wrapping_add，debug 模式不 panic
- [x] `core/src/library.rs` i64→u64 转换校验 >= 0，u64→i64 转换校验 <= i64::MAX
- [x] `core/src/ffi.rs` 本地扫描期间进度查询不被阻塞
- [x] `core/src/ffi.rs` 飞牛登录锁中毒时返回错误码
- [x] `core/src/sources/mod.rs` 持久化失败有 log::error! 记录
- [x] `core/src/sources/local.rs` 搜索使用 SQL LIMIT/OFFSET 分页
- [x] `core/src/sources/local.rs` 艺术家名 roundtrip 不再损坏
- [x] `core/src/ffi.rs` music_core_get_play_url 检查缓存命中
- [x] `core/src/ffi.rs` music_core_search 使用并发查询

## Android 端修复
- [x] `PlayerManager.kt` 当前曲目播放完毕后自动播放下一首（STATE_ENDED 处理）
- [x] `PlayerManager.kt` 本地文件路径正确构造为 file:// URI
- [x] `PlayerManager.kt` play/applyMediaItem 不再与 onCleared 竞态崩溃
- [x] `PlayerManager.kt` URL 为空时不误导性设置 currentSong
- [x] `NasScreen.kt` 播放歌曲时传入 queue
- [x] `MusicCoreBridge.kt` placeholderSources 线程安全（CopyOnWriteArrayList）
- [x] `MusicCoreBridge.kt` placeholderSeq 原子递增（AtomicInteger）
- [x] `SearchScreen.kt` 快速连续搜索结果不乱序
- [x] `SettingsScreen.kt` 删除音源失败不崩溃，有 snackbar 提示
- [x] `SettingsScreen.kt` 文件 I/O 在 IO 线程执行
- [x] `MusicRepository.kt` SongOrigin 解析不返回 null
- [x] `MusicRepository.kt` parse/parseList 捕获所有异常
- [x] `MusicRepository.kt` inferElementType 不误判类型
- [x] `Theme.kt` nghClickableScale 回调不过期
- [x] 各列表有 key 参数
- [x] `NasScreen.kt` 文件列表使用 LazyColumn
- [x] `LyricsScreen.kt` null timeMs 行不被高亮
- [x] `LeaderboardScreen.kt`/`LocalMusicScreen.kt` 加载失败显示错误状态
- [x] `NasScreen.kt` Surface onClick 与 nghClickableScale 不冲突
- [x] `NasScreen.kt` formatBytes 使用 Locale.US

## iOS 端修复
- [x] `PlayerManager.swift` play(song:in:) 队列索引正确，song 不在队列时设为单曲队列
- [x] `PlayerManager.swift` 加载失败时 isPlaying = false
- [x] `PlayerManager.swift` 已配置 AVAudioSession
- [x] `PlayerManager.swift` volume 同步到 player.volume
- [x] `PlayerManager.swift` KVO 校验 observedItem === currentItem
- [x] `LyricsView.swift` 无时间戳行不被高亮
- [x] `NasView.swift` 异步修改 @State 在主线程执行
- [x] `SearchView.swift` 搜索 Task 可取消
- [x] `NasView.swift` retry 保留取消语义
- [x] `NasView.swift` Song id 包含完整路径
- [x] `PlayerManager.swift` 随机模式不重复选当前曲
- [x] `SettingsView.swift` 排序 no-op 时不做乐观 UI 更新

## macOS 端修复
- [x] `PlayerService.swift` play(at:) 加载失败不置 isPlaying = true
- [x] `PlayerService.swift` volume didSet 有防抖
- [x] `Models.swift` SourceImportResult CodingKey 使用 snake_case
- [x] `Models.swift` NasFile CodingKeys isDir 映射为 "is_dir"
- [x] `LocalMusicView.swift` 目录管理与核心状态一致
- [x] `SearchView.swift` 搜索 Task 可取消
- [x] `LyricsView.swift` activeLineIndex 缓存避免 O(n²)
- [x] `PlayerService.swift` restoreState 完整实现或无用字段已移除

## Windows 端修复
- [x] `MusicCoreNative.cs` P/Invoke 使用 UTF-8 编码，中文不乱码
- [x] `PlayerService.cs` MediaPlayer 播放结束不无限循环
- [x] `SettingsPage.xaml.cs` 音源导入无 JSON 注入风险

## HarmonyOS 端修复
- [x] `PlayerManager.ets` AVPlayer 回调在销毁时正确注销
- [x] `PlayerManager.ets` ensurePlayer 与 release 无竞态
- [x] `PlayerManager.ets` stateWaiter 支持多回调
- [x] `NasPage.ets` 不阻塞 UI 线程

## Linux 端修复
- [x] `favorites.rs` 添加收藏/新建分组不触发 RefCell panic
- [x] `local_music.rs` 仅显示本地音源歌曲
- [x] `settings.rs` 删除音源后 UI 刷新
- [x] `core_service.rs` feiniu_login 锁中毒返回错误
- [x] `player_service.rs` Drop 时移除定时器
- [x] `playback_bar.rs`/`local_music.rs`/`lyrics.rs` 定时器 SourceId 已存储可移除
- [x] `leaderboard.rs` 无 O(n²) 内存问题
- [x] `lyrics.rs` 歌词高亮仅更新两行
- [x] `player_service.rs` seek 不立即写盘
- [x] `main.rs` 已初始化日志后端
- [x] `Cargo.toml` 添加 env_logger 依赖，tokio 精简 features
- [x] `player_service.rs` set_state 错误有日志记录
- [x] `core_service.rs` 本地扫描不阻塞进度查询
- [x] `core_service.rs` FTP 端口有校验
- [x] `player_service.rs` 随机播放使用更好的 PRNG
- [x] `theme.rs` CSS 路径不依赖工作目录
- [x] `player_service.rs` 播放状态恢复完整或无用字段已移除
- [x] format_artists/format_duration 抽取到公共模块
- [x] `lyrics.rs` load_lyric 闭包无重复
- [x] `settings.rs`/`nas.rs` 协议源行构造无重复
- [x] format_size/format_bytes 精度统一
