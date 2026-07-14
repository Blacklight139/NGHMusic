# 五端原生重构 Spec（推翻 Web 桌面框架，逐端原生重开发）

## Why
当前桌面端采用 Tauri 2 + Web UI 单一前端覆盖 Linux/Windows/macOS，违背「鸿蒙、安卓、Windows、macOS、iOS 各端采用各自不同的组件库与前端开发语言」的要求；Web 套壳无法充分调用各端原生组件能力，且与移动端原生体验割裂。本次**推翻原有代码框架并重新开发**：移除 Tauri/Web 桌面端，改为 Windows（WinUI 3 / C#）、macOS（SwiftUI / Swift）、iOS（SwiftUI / Swift）、Android（Jetpack Compose / Kotlin）、HarmonyOS（ArkUI / ArkTS）五端各自原生技术栈，共享 Rust 核心（`music-core`）作为业务逻辑层。docs 文件夹内的功能规范与音源规范为唯一功能来源，**不作修改**；五端 UI 可不统一，但功能必须一致并与 docs 描述对齐。最终进行代码审查，寻找潜在漏洞。

## 架构决策
- **共享核心（Rust，`core/`）**：音源引擎、元数据 API 客户端、聚合搜索、播放缓存、协议客户端（SMB/WebDAV/FTP/DLNA/NFS）、飞牛 API 客户端、本地音乐源、FFI 暴露。其对外契约（数据结构、错误类型、接口语义）与 docs 描述保持一致，docs 不修改。
- **Windows**：WinUI 3（Windows App SDK）+ C# / .NET 8，原生音频走 Windows Media Foundation / MediaPlayer，FFI 经 C# P/Invoke（或 uniffi-cs）调用 Rust 核心。
- **macOS**：SwiftUI + Swift（AppKit 互操作），原生音频走 AVPlayer，FFI 经 Swift 绑定（uniffi-swift）调用 Rust 核心。
- **iOS**：SwiftUI + Swift，原生音频走 AVPlayer，FFI 经 Swift 绑定（uniffi-swift）调用 Rust 核心。
- **Android**：Jetpack Compose + Kotlin，原生音频走 Media3/ExoPlayer，FFI 经 JNI/Kotlin 绑定调用 Rust 核心。
- **HarmonyOS**：ArkUI + ArkTS，原生音频走系统 AVPlayer，FFI 经 NAPI 绑定调用 Rust 核心。
- **UI 可不统一**：各端采用各自原生设计语言（WinUI Fluent、macOS/iOS Human Interface、Material 3、HarmonyOS Design），但功能集合与交互流程一致。
- **功能一致性基准**：以 `docs/` 为准——`docs/api/sound-source-api.md`（音源管理/搜索/元数据/播放URL/歌词/排行榜）、`docs/api/feiniu-api.md`（飞牛 NAS）、`docs/api/protocol-api.md`（网络协议）、`docs/sound-source-development.md`（音源开发规范与标准 JSON Schema）、`docs/bug-report.md`（已知问题与质量基线）。

## What Changes
- **BREAKING**：移除 `desktop/`（Tauri 2 + Web UI）整体代码框架，不再以 Web 前端覆盖任何桌面端
- **BREAKING**：移除 Linux 端支持（本次仅覆盖 Windows、macOS、iOS、Android、HarmonyOS 五端）
- 新建 Windows 原生客户端：`windows/`（WinUI 3 / C# / .NET 8 + Windows App SDK）
- 新建 macOS 原生客户端：`macos/`（SwiftUI / Swift + AppKit 互操作）
- 重建 iOS 原生客户端：`ios/`（SwiftUI / Swift + AVPlayer），对齐新框架
- 重建 Android 原生客户端：`android/`（Jetpack Compose / Kotlin + Media3），对齐新框架
- 重建 HarmonyOS 原生客户端：`harmonyos/`（ArkUI / ArkTS + 系统组件），对齐新框架
- 保留并按需重构共享 Rust 核心 `core/`，使其对外契约与 docs 一致；新增/调整 FFI 绑定以支持 C#（Windows）与 Swift（macOS/iOS）调用方
- 各端 UI 采用各自原生设计语言，不强求视觉统一；但功能集合、交互流程、数据模型与 docs 对齐
- 音源导入统一改为文件上传（.json 文件选择器），校验/适配后导入，音源管理仿 LXMusic 列表化（开关/排序/编辑/删除）
- 各端接入原生音频播放组件，实现播放/暂停/上一首/下一首/进度/音量/播放模式 + 状态持久化
- 实现核心功能：聚合搜索、播放列表、收藏夹、歌词滚动、排行榜、本地音乐、播放缓存、飞牛 NAS、网络协议（SMB/WebDAV/FTP/DLNA/NFS）、社区音源迁移
- 全端禁用 emoji，统一使用各端开源/系统图标库
- 最终代码审查：覆盖未处理异常、内存泄漏、状态不一致、异步竞态、边界值、兼容性、性能瓶颈、FFI 安全、鉴权与凭据处理等潜在漏洞

## Impact
- Affected specs:
  - `build-cross-platform-music-app`（架构决策被本次推翻：桌面端由 Tauri/Web 改为各端原生；Linux 端移除）
  - `redesign-ui-doubao-style`（UI 不再强求四端统一豆包风格；本次允许各端原生设计语言，但音源管理交互（文件上传 + LXMusic 列表）与禁用 emoji 规范保留）
- Affected code:
  - **删除**：`desktop/`（Tauri 2 + Web UI 全部）
  - **新建**：`windows/`（WinUI 3 / C#）、`macos/`（SwiftUI / Swift）
  - **重建**：`ios/`、`android/`、`harmonyos/`（对齐新框架与功能一致性）
  - **保留/重构**：`core/`（Rust 共享核心，FFI 绑定扩展）、`schemas/sound-source.schema.json`、`docs/`（不动）
- docs 不作修改，作为功能与音源规范的唯一来源

## ADDED Requirements

### Requirement: 五端原生架构（各自组件库与前端语言）
系统 SHALL 在 Windows、macOS、iOS、Android、HarmonyOS 五端运行，各端采用各自原生组件库与前端开发语言：Windows 用 WinUI 3（C#/.NET）、macOS 用 SwiftUI（Swift）、iOS 用 SwiftUI（Swift）、Android 用 Jetpack Compose（Kotlin）、HarmonyOS 用 ArkUI（ArkTS）；业务逻辑在共享 Rust 核心中实现，各端经各自 FFI 机制调用；UI 可不统一，但五端功能集合与交互流程一致。

#### Scenario: 各端使用原生组件库
- **WHEN** 用户在任一端打开应用
- **THEN** 界面由该端原生组件库渲染（Windows WinUI / macOS+iOS SwiftUI / Android Compose / HarmonyOS ArkUI），不存在 Web 套壳

#### Scenario: 共享核心跨端一致
- **WHEN** 同一音源配置在五端导入
- **THEN** 搜索、元数据获取、缓存、协议访问行为一致（同一 Rust 核心，仅 FFI 绑定不同），与 docs 描述对齐

### Requirement: Windows 原生客户端（WinUI 3 / C#）
系统 SHALL 提供 Windows 原生客户端，基于 WinUI 3（Windows App SDK）+ C# / .NET 8，音频播放使用 Windows 原生媒体组件，经 P/Invoke 或 uniffi-cs 调用 Rust 核心 FFI。

#### Scenario: Windows 端播放
- **WHEN** 用户在 Windows 端播放歌曲
- **THEN** 系统通过 Windows 原生媒体组件解码播放，控制行为与其它端一致

### Requirement: macOS 原生客户端（SwiftUI / Swift）
系统 SHALL 提供 macOS 原生客户端，基于 SwiftUI + Swift（AppKit 互操作），音频播放使用 AVPlayer，经 Swift 绑定（uniffi-swift）调用 Rust 核心 FFI。

#### Scenario: macOS 端播放
- **WHEN** 用户在 macOS 端播放歌曲
- **THEN** 系统通过 AVPlayer 解码播放，控制行为与其它端一致

### Requirement: iOS 原生客户端（SwiftUI / Swift）
系统 SHALL 提供 iOS 原生客户端，基于 SwiftUI + Swift，音频播放使用 AVPlayer，经 Swift 绑定调用 Rust 核心 FFI。

### Requirement: Android 原生客户端（Jetpack Compose / Kotlin）
系统 SHALL 提供 Android 原生客户端，基于 Jetpack Compose + Kotlin，音频播放使用 Media3/ExoPlayer，经 JNI/Kotlin 绑定调用 Rust 核心 FFI。

### Requirement: HarmonyOS 原生客户端（ArkUI / ArkTS）
系统 SHALL 提供 HarmonyOS 原生客户端，基于 ArkUI + ArkTS，音频播放使用系统 AVPlayer，经 NAPI 绑定调用 Rust 核心 FFI。

### Requirement: 功能一致性（对齐 docs）
五端 SHALL 实现与 docs 描述一致的功能集合：音源管理（导入/校验/启用/禁用/删除/列表）、聚合搜索（跨音源、分类、分页）、歌曲元数据/播放URL/歌词获取、排行榜、播放组件（播放/暂停/上一首/下一首/进度/音量/模式 + 持久化）、播放列表（增删/清空/排序）、收藏夹（多分组/导入导出）、歌词滚动（LRC 解析/同步/高亮/跳转/翻译）、本地音乐（扫描/元数据解析/库索引/文件夹监听/增量更新）、播放缓存（LRU/容量/清理）、飞牛 NAS（登录/列目录/流URL/健康检查）、网络协议（SMB/WebDAV/FTP/DLNA/NFS）、社区音源迁移。UI 可不统一，但任一功能在五端均可完成且行为一致。

#### Scenario: 跨端功能对等
- **WHEN** 用户在任一端使用任一 docs 描述的功能
- **THEN** 该功能在该端可正常完成，行为与其它端及 docs 描述一致

### Requirement: 共享 Rust 核心与 FFI 绑定扩展
系统 SHALL 保留共享 Rust 核心 `core/`，其对外契约（数据结构、错误类型、接口语义）与 docs 一致；并扩展 FFI 绑定以支持 C#（Windows P/Invoke 或 uniffi-cs）与 Swift（macOS/iOS uniffi-swift）调用方，同时维护现有 JNI（Android）与 NAPI（HarmonyOS）绑定。

#### Scenario: FFI 安全
- **WHEN** 任一端经 FFI 调用 Rust 核心
- **THEN** 跨 FFI 边界的裸指针解引用函数标记为 `unsafe` 并附 `# Safety` 文档；FFI 入口不触发 Rust panic（跨 FFI panic 为 UB）

### Requirement: 编写指南（开发规范）
项目 SHALL 遵循以下编写指南，五端统一执行：

1. **目录结构**：每端一个顶层目录（`windows/`、`macos/`、`ios/`、`android/`、`harmonyos/`），共享核心在 `core/`，文档在 `docs/`（只读），音源 Schema 在 `schemas/`。
2. **命名规范**：
   - Rust：snake_case 函数/变量、PascalCase 类型、SCREAMING_SNAKE_CASE 常量。
   - C#：PascalCase 公共成员、camelCase 局部变量。
   - Swift：PascalCase 类型、camelCase 方法/属性。
   - Kotlin：PascalCase 类、camelCase 函数/属性、lowercase 包名。
   - ArkTS：PascalCase 类型/组件、camelCase 方法/变量。
3. **功能对齐基准**：以 `docs/` 为唯一功能来源；新增/修改功能前先核对 docs，docs 不修改；五端功能集合必须对齐 docs 描述。
4. **音源规范**：音源 JSON 必须符合 `schemas/sound-source.schema.json`；导入走文件上传 + 校验/适配；社区音源经适配层转换并返回迁移报告（warnings）。
5. **错误处理**：跨 FFI 边界不 panic；网络/IO 错误经 `CoreError` 映射并向上传播为 `Result`；各端 UI 层捕获错误并给出明确提示与重试入口。
6. **异步与并发**：阻塞 IO 经 `spawn_blocking` 包装；缓存写/淘汰、文件监听避免持锁做同步 IO；竞态敏感路径加二次检查或 per-key 锁。
7. **资源释放**：各端原生播放器与 watcher 显式 release/dispose；FFI 分配的字符串/字节经对应 free 函数释放。
8. **图标规范**：全端禁用 emoji；Windows 用 Fluent Icons / SVG、macOS/iOS 用 SF Symbols、Android 用 Material Icons、HarmonyOS 用系统图标或开源 SVG。
9. **状态持久化**：播放状态、播放列表、收藏夹、音源配置、协议源、本地目录、缓存索引均持久化，重启可恢复。
10. **代码审查**：合并前各端跑各自静态分析（Rust clippy/rustfmt、C# analyzers、SwiftLint、Android Lint/Detekt、HarmonyOS 代码扫描）；FFI/鉴权/缓存/并发路径人工复审。

### Requirement: 代码审查与潜在漏洞排查
项目 SHALL 在五端实现完成后进行系统性代码审查，寻找潜在漏洞，覆盖：未处理异常、内存泄漏、状态不一致、异步竞态、边界值、兼容性、性能瓶颈、FFI 安全（裸指针/panic 跨边界）、鉴权与凭据处理（token/密码明文存储与传输）、路径遍历（本地音乐/协议源路径）、SSRF（音源 URL/协议源）、缓存竞态与损坏、资源未释放。产出漏洞报告（复现步骤/预期/实际/严重级别/分类），分类后修复并回归验证。

#### Scenario: 漏洞修复闭环
- **WHEN** 审查发现某潜在漏洞
- **THEN** 记录报告、分类严重级别、修复后回归验证通过

## MODIFIED Requirements

### Requirement: 跨平台架构（更新）
系统 SHALL 在 Windows、macOS、iOS、Android、HarmonyOS 五端运行（原六端移除 Linux）；各端采用各自原生组件库与前端开发语言（原「桌面 Tauri/Web」推翻为各端原生）；业务逻辑在共享 Rust 核心实现；UI 可不统一，五端功能一致并对齐 docs。

### Requirement: 简约 UI 与交互（更新）
系统 SHALL 采用各端原生设计语言（不强求五端视觉统一）；页面切换平滑稳定，无闪烁/卡顿；音源导入改为文件上传，音源管理仿 LXMusic；全端禁用 emoji，使用各端开源/系统图标库。

## REMOVED Requirements

### Requirement: Tauri 2 + Web UI 桌面端
**Reason**: 违背「各端采用各自不同组件库与前端语言」要求；Web 套壳无法充分调用各端原生组件能力。
**Migration**: Windows 改为 WinUI 3（C#），macOS 改为 SwiftUI（Swift）；删除 `desktop/` 目录。

### Requirement: Linux 端支持
**Reason**: 本次仅覆盖 Windows、macOS、iOS、Android、HarmonyOS 五端；Linux 不在范围。
**Migration**: 无（Linux 端原本依赖 Tauri/Web，随 `desktop/` 一并移除）。
