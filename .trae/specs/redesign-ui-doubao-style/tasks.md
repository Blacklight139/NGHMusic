# Tasks

## 阶段一：品牌与设计系统重做
- [x] Task 1: 品牌命名与图标库接入
  - [x] 1.1 四端窗口标题/品牌位统一显示「逆光音乐」/「NGHMusic」
  - [x] 1.2 桌面端引入 Lucide/Heroicons SVG 图标，移除所有 emoji
  - [x] 1.3 iOS 接入 SF Symbols，移除 emoji
  - [x] 1.4 Android 接入 Material Icons，移除 emoji
  - [x] 1.5 HarmonyOS 接入系统图标/开源 SVG，移除 emoji
- [x] Task 2: 定义豆包风格设计 token
  - [x] 2.1 确定配色（柔和浅色背景、低饱和强调色、文本/边框/阴影色阶）
  - [x] 2.2 确定圆角（≥12px）、间距、字体层级、阴影规范
  - [x] 2.3 桌面端重写 `desktop/src/styles.css` 设计 token 与全局样式
  - [x] 2.4 iOS 重写 `AppTheme.swift` 设计 token
  - [x] 2.5 Android 重写 `ui/theme/Color.kt`、`Theme.kt`、`Type.kt`
  - [x] 2.6 HarmonyOS 重写 `color.json` 与页面样式 token

## 阶段二：音源管理（LXMusic 风格 + 文件上传）
- [x] Task 3: 后端微调——音源列表与优先级接口
  - [x] 3.1 `core/src/sources/mod.rs` SourceManager 暴露有序列表与更新优先级方法（如已具备则确认）
  - [x] 3.2 桌面 `src-tauri/src/lib.rs` 新增/调整 command：list_sources、update_source_priority、delete_source、import_source_file
- [x] Task 4: 桌面端音源管理 UI
  - [x] 4.1 设置页音源列表（名称、开关、来源、操作）
  - [x] 4.2 文件上传导入入口（input type=file accept=.json）
  - [x] 4.3 开关启停、拖动/按钮排序、删除确认交互
- [x] Task 5: iOS 音源管理 UI
  - [x] 5.1 SettingsView 音源列表与文件选择导入（UIDocumentPicker）
  - [x] 5.2 开关、排序、删除交互
- [x] Task 6: Android 音源管理 UI
  - [x] 6.1 SettingsScreen 音源列表与文件选择导入（ActivityResultContracts.GetContent）
  - [x] 6.2 开关、排序、删除交互
- [x] Task 7: HarmonyOS 音源管理 UI
  - [x] 7.1 Settings.ets 音源列表与文件选择导入（picker）
  - [x] 7.2 开关、排序、删除交互

## 阶段三：功能页卡片化重做
- [x] Task 8: 桌面端各功能页豆包风格重做
  - [x] 8.1 index.html 布局调整（浅色侧栏、卡片内容区、品牌位）
  - [x] 8.2 search.js / playlist.js / favorites.js / leaderboard.js / local.js / lyrics.js 卡片化
  - [x] 8.3 main.js 播放组件栏浮动卡片样式
- [x] Task 9: iOS 各功能页豆包风格重做
  - [x] 9.1 ContentView 侧栏/Tab 浅色化与品牌位
  - [x] 9.2 各 View 卡片化与播放栏浮动样式
- [x] Task 10: Android 各功能页豆包风格重做
  - [x] 10.1 MainScreen 导航浅色化与品牌位
  - [x] 10.2 各 Screen 卡片化与 MiniPlayer 浮动样式
- [x] Task 11: HarmonyOS 各功能页豆包风格重做
  - [x] 11.1 Index.ets Tabs 浅色化与品牌位
  - [x] 11.2 各 page 卡片化与播放组件浮动样式

## 阶段四：交互与过渡优化
- [x] Task 12: 四端页面切换与微交互动画优化（柔和过渡、卡片 hover/press 反馈）

# Task Dependencies
- Task 3 依赖现有 SourceManager（已具备）
- Task 4/5/6/7 依赖 Task 3
- Task 8/9/10/11 依赖 Task 1、Task 2
- Task 12 依赖 Task 8~11
