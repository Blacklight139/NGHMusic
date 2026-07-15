# 代码质量审查与 Bug 修复 Spec

## Why
对全平台（Rust 核心 + Windows/macOS/iOS/Android/HarmonyOS/Linux 六端）代码库进行系统性代码质量审查后，发现了大量严重 bug、竞态条件、资源泄漏、错误处理缺陷和逻辑错误。这些问题涵盖了播放器核心功能失效、运行时崩溃、数据损坏等，严重影响应用稳定性和用户体验。本次变更旨在逐一修复审查发现的问题，将代码质量提升至生产可用水平。

## What Changes
- 修复 Rust 核心 `cache.rs` 缓存键碰撞导致数据串台的问题
- 修复 Rust 核心 FFI 本地扫描期间进度查询被阻塞的问题
- 修复 Rust 核心 `ffi.rs` 飞牛登录锁失败静默吞错的问题
- 修复 Rust 核心 `local.rs` 搜索全量加载内存分页的性能问题
- 修复 Rust 核心 `local.rs` 艺术家名 roundtrip 数据损坏
- 修复 Rust 核心 `ffi.rs` 获取播放 URL 未使用缓存的问题
- 修复 Rust 核心 FTP 端口截断无校验的问题
- 修复 Rust 核心 `ffi.rs` 序列号溢出风险
- 修复 Rust 核心 `library.rs` 整数转换溢出风险
- 修复 Rust 核心 `sources/mod.rs` 持久化失败静默吞错的问题
- 修复 Android `PlayerManager` 自动切歌失效（STATE_ENDED 未处理）
- 修复 Android `PlayerManager` 本地文件 URI 构造错误
- 修复 Android `PlayerManager` play/onCleared 竞态条件
- 修复 Android `SettingsScreen` 删除音源无 try-catch
- 修复 Android `NasScreen` 播放歌曲未加入队列
- 修复 Android `MusicCoreBridge` placeholderSources 线程安全问题
- 修复 Android `SearchScreen` 搜索未取消旧请求
- 修复 Android `MusicRepository` SongOrigin 解析返回 null 致 NPE
- 修复 Android `MusicRepository` inferElementType 误判类型
- 修复 Android `LyricsScreen` null timeMs 高亮错乱
- 修复 Android `Theme.kt` pointerInput(Unit) 回调过期
- 修复 Android 列表缺少 key 导致重组效率低
- 修复 Android `NasScreen` Column+forEach 渲染长列表
- 修复 Android `SettingsScreen` I/O 在主线程执行
- 修复 Android 多处加载失败静默回退占位数据
- 修复 iOS `PlayerManager` play(song:in:) 队列索引计算错误
- 修复 iOS `PlayerManager` 加载失败仍 isPlaying=true
- 修复 iOS `LyricsView` 无时间戳歌词行高亮错乱
- 修复 iOS `NasView` 异步修改 @State 未切主线程
- 修复 iOS `PlayerManager` 未配置 AVAudioSession
- 修复 iOS `PlayerManager` volume 未同步到 player
- 修复 iOS `SearchView` 搜索 Task 无取消
- 修复 iOS `NasView` 重试中 try? 吞取消语义
- 修复 macOS `PlayerService` play(at:) 失败后仍 isPlaying=true
- 修复 macOS `Models` SourceImportResult CodingKey 不匹配
- 修复 macOS `Models` NasFile CodingKeys isDir 未映射
- 修复 macOS `LocalMusicView` scanDirs 与核心状态脱节
- 修复 macOS `PlayerService` volume didSet 高频写盘
- 修复 macOS `SearchView` 搜索 Task 无取消
- 修复 Windows P/Invoke 字符串编码导致中文乱码
- 修复 Windows MediaPlayer 无限循环跳曲
- 修复 Windows SettingsPage JSON 注入问题
- 修复 HarmonyOS AVPlayer 回调泄漏
- 修复 HarmonyOS ensurePlayer/release 竞态
- 修复 HarmonyOS stateWaiter 单回调覆盖
- 修复 HarmonyOS NasPage busy-wait UI 阻塞
- 修复 Linux `favorites.rs` RefCell 重入 panic
- 修复 Linux `local_music.rs` 使用聚合搜索混入在线歌曲
- 修复 Linux `settings.rs` 删除音源不刷新 UI
- 修复 Linux `core_service.rs` feiniu_login 锁中毒静默丢失状态
- 修复 Linux `player_service.rs` 定时器泄漏
- 修复 Linux `main.rs` 无日志初始化
- 修复 Linux `playback_bar.rs` 等多处定时器 SourceId 未存储
- 修复 Linux `leaderboard.rs` O(n²) 内存问题
- 修复 Linux `player_service.rs` save_state 每次 seek 写盘
- 修复 Linux `player_service.rs` 播放状态保存后大部分不恢复
- 修复 Linux `core_service.rs` 本地扫描持有锁阻塞进度查询
- 修复 Linux `core_service.rs` FTP 端口截断无校验
- 修复 Linux `player_service.rs` 随机播放 PRNG 质量差
- 修复 Linux CSS 路径依赖工作目录
- 修复 Linux 多处 set_state 错误被忽略
- 修复 Linux 歌词高亮 O(n) 全量遍历性能问题
- 修复 Linux 重复代码抽取公共工具函数

## Impact
- Affected specs:
  - `add-linux-support`（其全端代码审查发现的问题本次修复）
  - `redevelop-native-per-platform`（其代码审查与漏洞排查发现的问题本次修复）
  - `build-cross-platform-music-app`（代码质量与 Bug 排查需求的具体落地）
- Affected code:
  - `core/src/cache.rs`、`core/src/ffi.rs`、`core/src/library.rs`、`core/src/sources/mod.rs`、`core/src/sources/local.rs`
  - `android/app/src/main/java/com/musicplayer/app/` 下多个文件
  - `ios/MusicPlayerApp/` 下多个文件
  - `macos/` 下多个文件
  - `windows/MusicPlayerApp/` 下多个文件
  - `harmonyos/entry/src/main/ets/` 下多个文件
  - `linux/src/` 下多个文件

## ADDED Requirements

### Requirement: 缓存键唯一性保证
系统 SHALL 确保缓存键不会因 sanitize 导致不同 song_id 映射到同一缓存文件，防止缓存数据串台。

#### Scenario: 缓存键碰撞
- **WHEN** 两个不同 song_id 经过 sanitize 后产生相同的简化键
- **THEN** 系统使用哈希或保留原始字符的方式确保缓存文件路径唯一，不发生数据串台

### Requirement: 播放器自动切歌功能
各端播放器 SHALL 在当前曲目播放结束时（STATE_ENDED / didPlayToEndTime / EOS）自动播放下一首，而非停止不动。

#### Scenario: 顺序播放自动切歌
- **WHEN** 当前曲目播放完毕
- **THEN** 播放器自动切换到队列中的下一首歌曲并开始播放

### Requirement: 播放器状态一致性
各端播放器 SHALL 在播放加载失败时正确设置 isPlaying 为 false，而非误导性地显示播放中。

#### Scenario: 播放加载失败
- **WHEN** 播放器加载歌曲 URL 失败（网络错误、无效 URL 等）
- **THEN** isPlaying 设为 false，UI 显示错误状态而非播放中

### Requirement: 队列索引正确性
各端播放器 SHALL 在播放单曲时正确设置队列和索引，确保后续切歌操作（上一首/下一首）行为正确。

#### Scenario: 播放不在队列中的歌曲
- **WHEN** 用户从 NAS/协议源播放一首不在当前队列中的歌曲
- **THEN** 系统将该歌曲设为单曲队列，索引为 0，确保切歌操作不会跳到不相关的歌曲

### Requirement: 本地音乐页仅查询本地源
Linux 端本地音乐页 SHALL 仅查询本地音源数据，不混入在线音源搜索结果。

#### Scenario: 本地音乐浏览
- **WHEN** 用户打开本地音乐页浏览歌曲
- **THEN** 仅显示本地音乐库中的歌曲，不包含在线音源结果

### Requirement: RefCell 借用安全
Linux 端 GTK4 代码 SHALL 避免在持有 RefCell 不可变借用时触发需要可变借用的信号回调，防止运行时 panic。

#### Scenario: 收藏页操作不崩溃
- **WHEN** 用户在收藏页点击"添加当前歌曲"或"新建分组"
- **THEN** 操作正常完成，不触发 RefCell 重入 panic

### Requirement: 进度查询不被扫描阻塞
系统 SHALL 确保本地音乐扫描期间进度查询不被阻塞，UI 能实时获取扫描进度。

#### Scenario: 扫描期间查询进度
- **WHEN** 本地音乐正在扫描
- **THEN** UI 线程能立即获取当前扫描进度，不被扫描操作阻塞

### Requirement: 线程安全的数据访问
Android 端 SHALL 使用线程安全的数据结构访问共享可变状态（如音源列表），防止并发修改异常。

#### Scenario: 并发访问音源列表
- **WHEN** 多个协程同时读取和修改音源列表
- **THEN** 不发生 ConcurrentModificationException，数据保持一致

### Requirement: 搜索结果时序一致性
各端搜索功能 SHALL 取消前一次搜索请求，确保最终显示的搜索结果与当前输入的关键词匹配。

#### Scenario: 快速连续搜索
- **WHEN** 用户快速连续输入多个关键词搜索
- **THEN** 最终显示的结果对应最后一次输入的关键词，不被旧请求的迟到结果覆盖

### Requirement: 错误处理完整性
各端 SHALL 在关键操作（删除音源、文件读取、网络请求等）中添加 try-catch 错误处理，失败时给出用户反馈而非崩溃。

#### Scenario: 删除音源失败
- **WHEN** 删除音源操作因网络或核心错误失败
- **THEN** 应用不崩溃，通过 snackbar/提示信息告知用户操作失败

### Requirement: 歌词高亮正确性
各端歌词功能 SHALL 正确处理无时间戳的歌词行，不将其误高亮为当前行。

#### Scenario: 无时间戳歌词行
- **WHEN** 歌词中包含无时间戳的纯文本行
- **THEN** 这些行不被高亮为当前行，高亮仅跟随有时间戳的行

### Requirement: 资源释放完整性
各端 SHALL 正确释放定时器、观察者、播放器等资源，防止内存泄漏。

#### Scenario: 页面销毁后资源释放
- **WHEN** 用户离开某页面或应用退出
- **THEN** 该页面关联的定时器被移除、观察者被注销、播放器被释放

### Requirement: 日志初始化
Linux 端 SHALL 初始化日志后端，确保 log::warn!/error!/info! 输出可见，便于生产环境排查问题。

#### Scenario: 日志输出
- **WHEN** 应用运行中发生警告或错误
- **THEN** 日志输出到 stderr 或文件，可通过环境变量控制日志级别

### Requirement: FFI 安全性增强
Rust 核心 FFI 层 SHALL 对整数转换进行范围校验，对锁中毒返回错误而非静默忽略，确保跨 FFI 边界不产生未定义行为。

#### Scenario: FTP 端口超范围
- **WHEN** 用户配置 FTP 端口超过 65535
- **THEN** 系统返回错误而非静默截断

#### Scenario: 锁中毒时返回错误
- **WHEN** FFI 全局状态的 Mutex 中毒
- **THEN** 对应操作返回错误码而非静默忽略

## MODIFIED Requirements

### Requirement: 代码质量与 Bug 排查（更新）
项目 SHALL 修复全平台代码审查发现的所有严重和高优先级 bug，包括：播放器功能缺陷、竞态条件、资源泄漏、错误处理缺失、数据损坏、线程安全、UI 状态不一致等问题，确保应用稳定运行。

## REMOVED Requirements
无
