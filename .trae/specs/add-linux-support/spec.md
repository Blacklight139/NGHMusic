# 新增 Linux 系统支持 Spec

## Why
当前项目覆盖 Windows、macOS、iOS、Android、HarmonyOS 五端，唯独缺少 Linux 桌面端支持。Linux 桌面用户群体（Ubuntu/Fedora/Arch 等）无法使用本应用。本次新增 Linux 原生客户端，采用 GTK4 + libadwaita + Rust（gtk4-rs）技术栈——这是 Linux 桌面适应性最好的组件库与前端语言组合：GTK4 是 Linux 桌面原生工具包（GNOME/KDE/XFCE 均兼容），libadwaita 提供现代自适应 UI 组件，Rust 与共享核心 `core/` 同语言可直接调用无需 FFI，GStreamer（gstreamer-rs）提供原生音频播放。完成后进行全端代码审查与 UI 改进。

## 技术决策
- **UI 框架**：GTK4 + libadwaita（gtk4-rs Rust 绑定），Linux 桌面原生工具包，适应性最佳
- **前端语言**：Rust（与共享核心 `core/` 同语言，直接依赖 `music-core` crate，无需 FFI 跨语言调用）
- **音频播放**：GStreamer（gstreamer-rs），Linux 原生多媒体框架
- **核心集成**：直接 `use music_core::*`，Rust crate 级依赖，无 FFI 开销
- **UI 设计语言**：豆包风格设计 Token（与 macOS/Windows 等端对齐），以 libadwaita 浅色主题为基底
- **图标**：使用 GTK 内置图标主题（Symbolic Icons），禁用 emoji
- **状态持久化**：`~/.local/share/nghmusic/` 目录（XDG Base Directory 规范）

## What Changes
- 新建 `linux/` 顶层目录，包含完整 GTK4 + Rust 客户端工程
- 新建 `linux/Cargo.toml`：依赖 `music-core`（path 依赖 `../core`）、gtk4、libadwaita、gstreamer
- 新建 `linux/src/main.rs`：应用入口，GTK Application 初始化
- 新建 `linux/src/models.rs`：镜像 `core/src/models.rs` 的数据模型（直接复用 core 类型，无需 FFI JSON 序列化）
- 新建 `linux/src/core_service.rs`：封装 `music-core` 全部能力，直接调用 Rust API（async tokio）
- 新建 `linux/src/player_service.rs`：GStreamer 播放器封装（play/pause/next/prev/seek/volume/mode + 状态持久化）
- 新建 `linux/src/theme.rs`：豆包风格设计 Token（颜色/圆角/间距），与 macOS AppTheme.swift 对齐
- 新建 `linux/src/views/`：全部功能页面（搜索/播放列表/收藏/歌词/排行榜/本地音乐/NAS/设置）
- 新建 `linux/src/views/playback_bar.rs`：底部播放控制栏
- 新建 `linux/src/views/sidebar.rs`：侧边栏导航
- 新建 `linux/resources/`：应用图标、CSS 样式资源
- 更新 `core/Cargo.toml`：无改动（Linux 直接依赖 core crate）
- 全端代码审查：检查现有五端代码中的潜在 bug、内存泄漏、竞态条件
- UI 改进：应用 frontend-skill 原则，改进各端 UI 视觉层级与交互体验

## Impact
- Affected specs:
  - `redevelop-native-per-platform`（原"移除 Linux 端支持"决策被本次推翻，恢复六端覆盖）
  - `redesign-ui-doubao-style`（Linux 端同步落地豆包风格设计 Token）
- Affected code:
  - **新建**：`linux/`（完整 GTK4 + Rust 客户端）
  - **审查/修复**：`macos/`、`windows/`、`ios/`、`android/`、`harmonyos/`、`core/`（全端代码审查）
  - **改进**：各端 UI 文件（frontend-skill 原则指导的视觉改进）

## ADDED Requirements

### Requirement: Linux 原生客户端（GTK4 + Rust）
系统 SHALL 提供 Linux 原生客户端，基于 GTK4 + libadwaita（gtk4-rs Rust 绑定），音频播放使用 GStreamer（gstreamer-rs），直接依赖共享 Rust 核心 `music-core` crate（无 FFI 跨语言调用开销）。

#### Scenario: Linux 端启动
- **WHEN** 用户在 Linux 桌面启动应用
- **THEN** GTK4 窗口正常显示，品牌名「逆光音乐」出现在标题栏与侧边栏

#### Scenario: 直接调用核心
- **WHEN** Linux 客户端执行搜索/播放/音源管理等操作
- **THEN** 直接调用 `music_core` Rust API（非 FFI），行为与其它端一致

### Requirement: Linux 端 GStreamer 音频播放
系统 SHALL 在 Linux 端使用 GStreamer 实现音频播放，支持播放/暂停/上一首/下一首/进度跳转/音量控制/播放模式（顺序/单曲循环/随机），播放状态持久化到 `~/.local/share/nghmusic/player_state.json`。

#### Scenario: Linux 端播放控制
- **WHEN** 用户在 Linux 端播放歌曲并执行播放控制操作
- **THEN** GStreamer 正常解码播放，控制行为与其它端一致

### Requirement: Linux 端豆包风格 UI
系统 SHALL 在 Linux 端采用豆包风格设计语言，与 macOS/Windows 等端视觉对齐：柔和浅色背景、语义化颜色 Token、圆角卡片、充足留白、禁用 emoji、使用 GTK Symbolic Icons。

#### Scenario: Linux 端视觉一致性
- **WHEN** 用户在 Linux 端打开任一页面
- **THEN** UI 风格与 macOS 端豆包风格视觉一致（配色/圆角/间距/字体层级）

### Requirement: Linux 端功能完整对齐
Linux 端 SHALL 实现与 docs 描述一致的全部功能：音源管理（导入/校验/启用/禁用/删除/列表）、聚合搜索、歌曲元数据/播放URL/歌词获取、排行榜、播放组件、播放列表、收藏夹、歌词滚动、本地音乐、播放缓存、飞牛 NAS、网络协议（SMB/WebDAV/FTP/DLNA/NFS）。

#### Scenario: Linux 端功能对等
- **WHEN** 用户在 Linux 端使用任一功能
- **THEN** 该功能可正常完成，行为与其它五端及 docs 描述一致

### Requirement: 全端代码审查与 Bug 修复
项目 SHALL 在 Linux 端开发完成后进行系统性代码审查，覆盖全部六端 + 共享核心，寻找并修复潜在 bug：未处理异常、内存泄漏、状态不一致、异步竞态、FFI 安全、资源未释放、边界值处理。

#### Scenario: 审查发现并修复 Bug
- **WHEN** 代码审查发现某潜在 bug
- **THEN** 记录问题、修复后回归验证通过

### Requirement: UI 视觉改进
项目 SHALL 应用 frontend-skill 设计原则改进各端 UI：强化视觉层级与品牌存在感、优化留白与对齐、统一交互反馈、移除不必要的卡片装饰、提升整体观感至 award-level 水准。

#### Scenario: UI 改进效果
- **WHEN** 用户查看改进后的任一端界面
- **THEN** 视觉层级清晰、品牌突出、留白充足、交互反馈一致

## MODIFIED Requirements

### Requirement: 跨平台架构（更新）
系统 SHALL 在 Windows、macOS、iOS、Android、HarmonyOS、Linux 六端运行（原五端新增 Linux）；各端采用各自原生组件库与前端开发语言；业务逻辑在共享 Rust 核心实现；Linux 端直接依赖 Rust 核心 crate（无 FFI），其它端经各自 FFI 机制调用；六端功能一致并对齐 docs。

### Requirement: 编写指南（更新）
项目编写指南新增 Linux 端规范：
1. **目录结构**：新增 `linux/` 顶层目录
2. **命名规范**：Rust snake_case 函数/变量、PascalCase 类型
3. **图标规范**：Linux 端使用 GTK Symbolic Icons，禁用 emoji
4. **状态持久化**：Linux 端使用 `~/.local/share/nghmusic/`（XDG 规范）

## REMOVED Requirements

### Requirement: Linux 端不支持
**Reason**: 原 `redevelop-native-per-platform` spec 中移除了 Linux 端支持，本次恢复。
**Migration**: 新建 `linux/` 目录，采用 GTK4 + Rust 原生实现。
