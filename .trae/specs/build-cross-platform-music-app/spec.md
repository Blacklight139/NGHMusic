# 跨平台音乐播放器 Spec

## Why
当前缺乏一款同时覆盖 Linux、Windows、macOS、HarmonyOS、iOS、Android 六端、支持开发者通过 JSON 自定义音源、并接入 NAS（飞牛 API）与多种网络协议（SMB/WebDAV/FTP/DLNA/NFS）的统一音乐播放器。本项目旨在交付一款简约风格、功能完整、且具备完善开发者生态（音源开发文档 + 仿 Apifox 规格 API 文档）的跨平台音乐软件。

## 架构决策（已确认）
- **代码组织**：全原生多后端 + 共享 Rust 核心。
  - 共享核心（Rust，经 FFI 暴露）：音源引擎、元数据 API 客户端、聚合搜索、播放缓存、协议客户端（SMB/WebDAV/FTP/DLNA/NFS）、飞牛 API 客户端。
  - 桌面端（Linux/Windows/macOS）：Tauri 2 + Web UI（单一前端，覆盖三端），音频走 Tauri 原生插件/系统播放组件。
  - iOS：SwiftUI + AVPlayer。
  - Android：Jetpack Compose + ExoPlayer/Media3。
  - HarmonyOS：ArkUI + AVPlayer（系统组件）。
  - 各端调用各自原生音频播放组件与 UI 组件库，六端功能要求一致。
- **音源格式**：我方定义标准 JSON Schema，并在其上提供社区音源迁移/适配方案（导入时自动转换/兼容层）。

## What Changes
- 新建跨平台音乐播放器，覆盖 Linux、Windows、macOS、HarmonyOS、iOS、Android 六端
- 共享 Rust 核心 + 四套原生前端（桌面 Tauri/iOS/Android/HarmonyOS），各端使用各自原生音频与 UI 组件库
- 实现设置内 JSON 音源导入，支持校验、启用/禁用、切换、优先级；附社区音源迁移方案
- 制定面向开发者的音源开发文档：通过 API 获取主流音乐软件元数据；搜索结果点击后在音源内定位数据并缓存播放
- 提供 Markdown 编写、仿 Apifox 规格的 API 文档（音源接口、飞牛接口、协议接口）
- 实现核心功能：收藏夹、播放列表、歌词滚动、排行榜、搜索、播放组件
- 简约风格 UI：视觉协调统一，平滑稳定页面切换
- 接入飞牛 API，实现通过 NAS 设备播放歌曲
- 支持 SMB、WebDAV、FTP、DLNA、NFS 网络协议访问与管理音乐资源
- 全面代码质量与潜在 Bug 排查（边界条件、错误处理、内存泄漏、状态管理、竞态条件、兼容性、性能瓶颈）
- 输出 Bug 报告（复现步骤、预期行为、实际行为、优先级），分类并修复验证

## Impact
- Affected specs: 全新项目，无既有 spec 受影响
- Affected code: 全新代码库
  - `core/`（Rust 共享核心）
  - `desktop/`（Tauri 2 + Web，覆盖 Linux/Windows/macOS）
  - `ios/`（SwiftUI）、`android/`（Compose）、`harmonyos/`（ArkUI）
  - `docs/`（音源开发文档 + 仿 Apifox API 文档）
  - `schemas/sound-source.schema.json`（标准音源 Schema）

## ADDED Requirements

### Requirement: 跨平台架构与原生音频后端
系统 SHALL 在 Linux、Windows、macOS、HarmonyOS、iOS、Android 六端运行；各端调用各自原生音频播放组件（桌面系统组件/AVPlayer/ExoPlayer/HarmonyOS AVPlayer）与原生 UI 组件库；业务逻辑、音源引擎、协议客户端、缓存层在共享 Rust 核心中实现，六端功能要求一致。

#### Scenario: 各端音频播放使用原生组件
- **WHEN** 用户在任一端播放歌曲
- **THEN** 系统通过该端原生音频组件解码与播放，播放控制行为与其它端一致

#### Scenario: 共享核心跨端一致
- **WHEN** 同一音源配置在六端导入
- **THEN** 搜索、元数据获取、缓存行为一致（同一 Rust 核心，仅 FFI 绑定不同）

### Requirement: JSON 音源导入与迁移
系统 SHALL 在设置内提供 JSON 音源导入入口，基于我方标准 JSON Schema 校验，支持启用/禁用、切换、优先级设置；并提供社区音源迁移/适配方案（导入时自动转换为标准格式或经兼容层加载）。

#### Scenario: 导入合法标准音源
- **WHEN** 用户在设置内导入一份符合标准 Schema 的 JSON 音源
- **THEN** 系统校验通过后启用该音源并可在搜索/播放中使用

#### Scenario: 导入社区音源
- **WHEN** 用户导入一份社区格式音源
- **THEN** 系统经适配层转换为标准格式后加载，并提示迁移结果

#### Scenario: 导入非法音源
- **WHEN** 用户导入格式不合法或无法适配的 JSON
- **THEN** 系统提示具体错误原因且不加载该音源

### Requirement: 音源元数据获取
音源 SHALL 通过 API 获取主流音乐软件的元数据（歌曲、专辑、艺术家、封面、歌词、时长等），返回结构化数据供 UI 展示。

#### Scenario: 获取歌曲元数据
- **WHEN** 系统依据音源 API 请求某歌曲元数据
- **THEN** 返回包含标题/艺术家/专辑/封面/时长/歌词地址的结构化元数据

### Requirement: 搜索定位与缓存播放
系统 SHALL 在搜索结果点击后，定位音源内相关播放数据并缓存播放；缓存命中时优先本地播放以降低带宽与延迟。

#### Scenario: 搜索点击播放
- **WHEN** 用户搜索某歌曲并点击
- **THEN** 系统在音源内定位播放数据，缓存后开始播放

#### Scenario: 缓存命中
- **WHEN** 用户再次播放已缓存歌曲
- **THEN** 系统从本地缓存读取播放，无需重新请求音源

### Requirement: 播放组件与播放控制
系统 SHALL 提供播放/暂停、上一首/下一首、进度拖动、音量、播放模式（顺序/单曲循环/随机）的播放组件，状态在重启后持久化。

### Requirement: 播放列表
系统 SHALL 维护当前播放列表，支持添加、移除、清空、拖动排序，并与播放组件状态联动。

### Requirement: 收藏夹
系统 SHALL 提供收藏夹，支持多分组、添加/移除、导入/导出。

### Requirement: 歌词滚动
系统 SHALL 解析并同步显示歌词，支持滚动高亮、点击跳转、可选翻译（如音源提供）。

### Requirement: 排行榜
系统 SHALL 展示音源提供的排行榜，支持点击进入详情并播放整榜或单曲。

### Requirement: 搜索
系统 SHALL 提供跨音源聚合搜索，支持歌曲/专辑/艺术家分类与分页，结果展示音源来源。

### Requirement: 简约 UI 与交互
系统 SHALL 采用简约风格，视觉协调统一；页面切换平滑稳定，无闪烁/卡顿。

#### Scenario: 页面切换
- **WHEN** 用户在不同功能页间切换
- **THEN** 过渡动画平滑稳定

### Requirement: 飞牛 API 集成
系统 SHALL 接入飞牛 API，实现通过 NAS 设备播放歌曲，调用稳定可靠；网络异常时给出明确提示并支持重试。

#### Scenario: NAS 播放
- **WHEN** 用户配置飞牛 NAS 并选择歌曲
- **THEN** 系统通过飞牛 API 获取并播放

#### Scenario: NAS 异常
- **WHEN** 飞牛 API 调用失败
- **THEN** 系统提示错误并支持重试，不崩溃

### Requirement: 网络协议支持
系统 SHALL 支持 SMB、WebDAV、FTP、DLNA、NFS 协议访问与管理音乐资源，支持浏览/搜索/加入播放列表。

#### Scenario: 协议浏览
- **WHEN** 用户添加某协议源并浏览
- **THEN** 系统列出可播放音乐文件并可加入播放

### Requirement: 开发者音源开发文档
系统 SHALL 附带面向开发者的音源开发文档，说明标准 JSON Schema、元数据 API 对接、搜索定位与缓存播放对接、社区音源迁移方案。

### Requirement: API 文档（仿 Apifox 规格）
系统 SHALL 提供 Markdown 编写、仿 Apifox 规格的 API 文档，覆盖音源接口、飞牛接口、协议接口，含请求/响应示例与错误码。

### Requirement: 代码质量与 Bug 排查
项目 SHALL 进行全面潜在 Bug 排查，覆盖：未处理异常、内存泄漏、状态管理不一致、异步竞态、边界值、兼容性、性能瓶颈；使用静态分析 + 动态调试 + 场景测试结合，产出 Bug 报告（复现步骤、预期、实际、优先级），分类后修复并验证。

#### Scenario: Bug 修复闭环
- **WHEN** 排查发现某 Bug
- **THEN** 记录报告、分类优先级、修复后回归验证通过
