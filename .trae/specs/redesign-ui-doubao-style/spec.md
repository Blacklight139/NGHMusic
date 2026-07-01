# UI 重设计（豆包风格 + LXMusic 音源管理）Spec

## Why
当前六端 UI 采用 Spotify 风格（绿色主色 + 深色侧栏），视觉偏重且与主流 AI 助手类应用的简约柔和趋势不符；音源导入仅提供 JSON 文本框，操作门槛高且导入后缺乏可视化管理。本次重设计将 UI 全面改为豆包风格（极简、柔和、圆角卡片、充足留白），并将音源导入改为文件上传、音源管理仿照 LXMusic（列表化、开关、排序、编辑、删除），提升整体观感与音源管理体验。

## 范围说明
- **本次仅更改 UI 设计**，四端（桌面 Tauri/iOS SwiftUI/Android Compose/HarmonyOS ArkUI）前端全面重做样式与音源管理交互。
- **后端可做微调**：仅允许为支撑新 UI/交互（如音源列表查询、排序持久化）对 Rust 核心做小幅适配，不改动音源引擎、协议、缓存等核心逻辑。

## What Changes
- 四端 UI 全面重设计为豆包风格：柔和浅色配色、圆角卡片、充足留白、柔和阴影、友好字体层级
- 桌面端侧栏由深色改为浅色柔和风格，主色由 #1db954 改为豆包风柔和色（如 #4E6EF2 蓝紫 或 #C8A064 暖金，最终以设计稿为准）
- 音源导入改为文件上传（文件选择器选 .json 文件），移除纯文本框导入方式
- 导入后音源如 LXMusic 显示在设置页音源列表中：每项含名称、启用/禁用开关、来源标识、操作（编辑/删除/排序）
- 音源管理仿 LXMusic：列表视图、开关启停、拖动或按钮调整优先级、单击查看详情、删除确认
- 各功能页（搜索/播放列表/收藏/排行榜/本地音乐/歌词/设置）统一豆包风格卡片化布局
- 播放组件栏改为柔和浮动卡片样式
- 后端微调：为音源列表排序持久化提供支持（如 SourceManager 暴露有序列表与优先级更新接口）

## Impact
- Affected specs: build-cross-platform-music-app（UI/UX 相关需求被本次重设计覆盖更新）
- Affected code:
  - 桌面：`desktop/src/styles.css`（重写设计系统）、`desktop/src/index.html`、`desktop/src/main.js`、`desktop/src/pages/*`（重写各页样式与音源管理交互）、`desktop/src-tauri/src/lib.rs`（音源列表/排序 command 微调）
  - iOS：`ios/MusicPlayerApp/Views/*`、`ios/MusicPlayerApp/Views/AppTheme.swift`（重写设计 token 与视图）
  - Android：`android/.../ui/theme/*`、`android/.../ui/screens/*`（重写主题与屏幕）
  - HarmonyOS：`harmonyos/.../pages/*`、`harmonyos/.../resources/base/element/color.json`（重写页面与色彩）
  - 后端微调：`core/src/sources/mod.rs`（SourceManager 暴露有序列表与优先级更新）

## ADDED Requirements

### Requirement: 豆包风格设计系统
系统 SHALL 采用豆包风格视觉语言：柔和浅色背景、圆角卡片（radius ≥ 12px）、充足留白、柔和阴影、友好字体层级、低饱和强调色；四端共享统一设计 token 并各自落地。

#### Scenario: 浅色柔和基调
- **WHEN** 用户打开应用任一页面
- **THEN** 背景为柔和浅色，内容以圆角卡片承载，无深色侧栏或高饱和色块

#### Scenario: 四端设计一致
- **WHEN** 同一界面在四端展示
- **THEN** 配色、圆角、间距、字体层级一致（共享设计 token）

### Requirement: JSON 音源文件上传导入
系统 SHALL 提供文件上传方式导入 JSON 音源：用户通过文件选择器选取 .json 文件，系统读取内容并校验/适配后导入，移除纯文本框导入入口。

#### Scenario: 上传合法音源文件
- **WHEN** 用户点击「导入音源」并选择一个合法的 .json 音源文件
- **THEN** 系统读取文件、校验/适配通过后导入，并在音源列表中显示

#### Scenario: 上传非法文件
- **WHEN** 用户选择格式不合法或无法适配的文件
- **THEN** 系统提示具体错误原因，不导入

### Requirement: LXMusic 风格音源管理
系统 SHALL 在设置页以列表形式管理音源（仿 LXMusic）：每项显示音源名称、启用/禁用开关、来源标识；支持拖动或按钮调整优先级、单击查看详情、删除（带确认）；导入后立即出现在列表。

#### Scenario: 音源列表展示
- **WHEN** 用户进入设置页音源管理
- **THEN** 以列表展示所有已导入音源，每项含名称、开关、来源标识、操作入口

#### Scenario: 启停音源
- **WHEN** 用户切换某音源的启用开关
- **THEN** 该音源立即在搜索/播放中生效或失效，状态持久化

#### Scenario: 调整优先级
- **WHEN** 用户拖动或点击上下按钮调整音源顺序
- **THEN** 聚合搜索按新顺序执行，顺序持久化

#### Scenario: 删除音源
- **WHEN** 用户删除某音源并确认
- **THEN** 该音源从列表移除且不再参与搜索/播放

### Requirement: 功能页卡片化布局
各功能页（搜索/播放列表/收藏/排行榜/本地音乐/歌词/设置）SHALL 采用豆包风格卡片化布局：内容以圆角卡片分组承载，列表项卡片化，留白充足。

### Requirement: 播放组件栏柔和浮动样式
底部播放组件栏 SHALL 改为柔和浮动卡片样式：圆角、柔和阴影、与内容区有间距，控制按钮采用豆包风格图标与配色。

## MODIFIED Requirements

### Requirement: 简约 UI 与交互（更新）
系统 SHALL 采用豆包风格简约设计（原「简约风格」更新为「豆包风格」），视觉协调统一；页面切换平滑稳定，无闪烁/卡顿；音源导入改为文件上传，音源管理仿 LXMusic。

## REMOVED Requirements

### Requirement: 深色侧栏 + 高饱和绿色主题
**Reason**: 与豆包风格柔和浅色基调冲突
**Migration**: 桌面端侧栏改为浅色柔和风格，主色改为低饱和强调色，四端同步
