# Tasks

## 阶段一：Linux 工程脚手架与核心集成
- [x] Task 1: 创建 Linux GTK4 + Rust 工程骨架
  - [x] 1.1 新建 `linux/Cargo.toml`，配置依赖（music-core path 依赖、gtk4、libadwaita、gstreamer）
  - [x] 1.2 新建 `linux/src/main.rs`，GTK Application 初始化，窗口标题显示「逆光音乐」
  - [x] 1.3 新建 `linux/resources/` 目录，放置应用图标与 CSS 样式
  - [x] 1.4 配置构建脚本（build.rs / gresources），确保 GTK4 + libadwaita 正确链接
- [x] Task 2: CoreService 封装（直接调用 music-core Rust API）
  - [x] 2.1 新建 `linux/src/core_service.rs`，直接 `use music_core::*` 调用核心 API
  - [x] 2.2 封装音源管理（import/validate/list/enable/disable/delete）
  - [x] 2.3 封装搜索/元数据/播放URL/歌词/排行榜
  - [x] 2.4 封装飞牛 NAS（login/list_files/stream/health）
  - [x] 2.5 封装协议源（add/list/delete/list_files/read/stream）
  - [x] 2.6 封装本地音乐（init/add_dir/rescan/progress）与缓存（init/stats/clear）
- [x] Task 3: GStreamer 播放器服务
  - [x] 3.1 新建 `linux/src/player_service.rs`，初始化 GStreamer pipeline
  - [x] 3.2 实现播放/暂停/上一首/下一首/进度跳转/音量控制/播放模式
  - [x] 3.3 播放状态持久化到 `~/.local/share/nghmusic/player_state.json`
  - [x] 3.4 播放结束自动下一首（GStreamer EOS 信号处理）

## 阶段二：Linux UI 框架与设计 Token
- [x] Task 4: 豆包风格设计 Token 与主题
  - [x] 4.1 新建 `linux/src/theme.rs`，定义颜色/圆角/间距 Token（与 macOS AppTheme.swift 对齐）
  - [x] 4.2 编写 CSS 样式文件（`linux/resources/style.css`），应用豆包风格到 GTK4 组件
  - [x] 4.3 加载 libadwaita 浅色主题作为基底
- [x] Task 5: 主窗口布局（侧边栏 + 详情区 + 播放栏）
  - [x] 5.1 新建 `linux/src/views/sidebar.rs`，侧边栏导航（品牌头部 + 功能页列表 + 底部信息）
  - [x] 5.2 实现主窗口布局（OverlaySplitView / PanedWindow，侧边栏 + 详情区 + 底部播放栏）
  - [x] 5.3 页面切换逻辑（GtkStack / 页面路由）

## 阶段三：Linux 功能页面实现
- [x] Task 6: 搜索页面
  - [x] 6.1 搜索框 + 搜索按钮 + 加载状态
  - [x] 6.2 搜索结果列表（歌曲行：序号 + 封面占位 + 标题 + 艺术家 + 来源标签）
  - [x] 6.3 点击歌曲播放，当前播放高亮
- [x] Task 7: 播放列表页面
  - [x] 7.1 播放队列列表展示
  - [x] 7.2 点击播放/清空队列
  - [x] 7.3 当前播放曲目高亮
- [x] Task 8: 收藏夹页面
  - [x] 8.1 收藏分组列表
  - [x] 8.2 添加/移除收藏歌曲
- [x] Task 9: 歌词页面
  - [x] 9.1 LRC 歌词解析与时间轴同步滚动
  - [x] 9.2 当前行高亮，点击跳转
- [x] Task 10: 排行榜页面
  - [x] 10.1 音源选择 + 榜单列表
  - [x] 10.2 榜单歌曲展示与播放
- [x] Task 11: 本地音乐页面
  - [x] 11.1 扫描目录管理（添加/移除/重扫/进度）
  - [x] 11.2 本地音乐库浏览（歌曲列表）
- [x] Task 12: NAS 页面
  - [x] 12.1 飞牛 NAS 登录/健康检查/文件浏览/播放
  - [x] 12.2 协议源浏览/播放
- [x] Task 13: 设置页面
  - [x] 13.1 音源导入（文件选择 .json）+ LXMusic 风格列表管理
  - [x] 13.2 协议源管理（添加/删除）
  - [x] 13.3 缓存管理（统计/清理）
  - [x] 13.4 关于页面（版本信息）

## 阶段四：Linux 播放控制栏
- [x] Task 14: 底部播放控制栏
  - [x] 14.1 当前歌曲信息（封面占位 + 标题 + 艺术家）
  - [x] 14.2 播放控制按钮（上一首/播放暂停/下一首/播放模式）
  - [x] 14.3 进度条 + 时间显示
  - [x] 14.4 音量控制

## 阶段五：全端代码审查
- [x] Task 15: 共享核心（core/）代码审查
  - [x] 15.1 FFI 安全：裸指针解引用/panic 跨边界/字符串释放配对
  - [x] 15.2 并发竞态：缓存写/淘汰、文件监听、SourceManager 锁
  - [x] 15.3 错误处理：unwrap/expect 残留、错误传播完整性
  - [x] 15.4 资源释放：连接/watcher/文件句柄
- [x] Task 16: macOS 端代码审查
  - [x] 16.1 KVO/Observer 释放（statusObserver/endObserver/timeObserverToken）
  - [x] 16.2 AVPlayer 状态管理与竞态
  - [x] 16.3 CoreService actor 并发安全
- [x] Task 17: Windows 端代码审查
  - [x] 17.1 P/Invoke 调用安全（字符串释放/内存泄漏）
  - [x] 17.2 MediaPlayer 资源释放
  - [x] 17.3 异步调用与 UI 线程
- [x] Task 18: iOS/Android/HarmonyOS 端代码审查
  - [x] 18.1 iOS：AVPlayer/Swift 并发安全
  - [x] 18.2 Android：ExoPlayer 生命周期/Compose 状态
  - [x] 18.3 HarmonyOS：AVPlayer/NAPI 绑定安全
- [x] Task 19: 审查问题修复
  - [x] 19.1 记录审查发现的问题清单
  - [x] 19.2 按严重级别分类修复
  - [x] 19.3 回归验证

## 阶段六：UI 视觉改进（frontend-skill 原则）
- [x] Task 20: 各端 UI 视觉改进
  - [x] 20.1 macOS 端：优化视觉层级、留白、对齐，移除不必要卡片装饰
  - [x] 20.2 Windows 端：统一间距 Token、改进列表行视觉层级
  - [x] 20.3 iOS 端：优化线性列表风格、当前播放高亮
  - [x] 20.4 Android 端：统一搜索/列表行视觉
  - [x] 20.5 HarmonyOS 端：统一列表行风格
  - [x] 20.6 Linux 端：确保与改进后的各端视觉一致

# Task Dependencies
- Task 2 依赖 Task 1
- Task 3 依赖 Task 1
- Task 4 与 Task 2/3 可并行
- Task 5 依赖 Task 4
- Task 6~14 依赖 Task 2、Task 3、Task 5
- Task 14 可与 Task 6~13 部分并行
- Task 15~18 可并行（各端独立审查）
- Task 19 依赖 Task 15~18
- Task 20 依赖 Task 6~14、Task 19
