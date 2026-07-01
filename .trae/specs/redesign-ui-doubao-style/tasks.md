# Tasks

## 阶段一：设计系统重做
- [ ] Task 1: 定义豆包风格设计 token
  - [ ] 1.1 确定配色（柔和浅色背景、低饱和强调色、文本/边框/阴影色阶）
  - [ ] 1.2 确定圆角（≥12px）、间距、字体层级、阴影规范
  - [ ] 1.3 桌面端重写 `desktop/src/styles.css` 设计 token 与全局样式
  - [ ] 1.4 iOS 重写 `AppTheme.swift` 设计 token
  - [ ] 1.5 Android 重写 `ui/theme/Color.kt`、`Theme.kt`、`Type.kt`
  - [ ] 1.6 HarmonyOS 重写 `color.json` 与页面样式 token

## 阶段二：音源管理（LXMusic 风格 + 文件上传）
- [ ] Task 2: 后端微调——音源列表与优先级接口
  - [ ] 2.1 `core/src/sources/mod.rs` SourceManager 暴露有序列表与更新优先级方法（如已具备则确认）
  - [ ] 2.2 桌面 `src-tauri/src/lib.rs` 新增/调整 command：list_sources、update_source_priority、delete_source、import_source_file
- [ ] Task 3: 桌面端音源管理 UI
  - [ ] 3.1 设置页音源列表（名称、开关、来源、操作）
  - [ ] 3.2 文件上传导入入口（input type=file accept=.json）
  - [ ] 3.3 开关启停、拖动/按钮排序、删除确认交互
- [ ] Task 4: iOS 音源管理 UI
  - [ ] 4.1 SettingsView 音源列表与文件选择导入（UIDocumentPicker）
  - [ ] 4.2 开关、排序、删除交互
- [ ] Task 5: Android 音源管理 UI
  - [ ] 5.1 SettingsScreen 音源列表与文件选择导入（ActivityResultContracts.GetContent）
  - [ ] 5.2 开关、排序、删除交互
- [ ] Task 6: HarmonyOS 音源管理 UI
  - [ ] 6.1 Settings.ets 音源列表与文件选择导入（picker）
  - [ ] 6.2 开关、排序、删除交互

## 阶段三：功能页卡片化重做
- [ ] Task 7: 桌面端各功能页豆包风格重做
  - [ ] 7.1 index.html 布局调整（浅色侧栏、卡片内容区）
  - [ ] 7.2 search.js / playlist.js / favorites.js / leaderboard.js / local.js / lyrics.js 卡片化
  - [ ] 7.3 main.js 播放组件栏浮动卡片样式
- [ ] Task 8: iOS 各功能页豆包风格重做
  - [ ] 8.1 ContentView 侧栏/Tab 浅色化
  - [ ] 8.2 各 View 卡片化与播放栏浮动样式
- [ ] Task 9: Android 各功能页豆包风格重做
  - [ ] 9.1 MainScreen 导航浅色化
  - [ ] 9.2 各 Screen 卡片化与 MiniPlayer 浮动样式
- [ ] Task 10: HarmonyOS 各功能页豆包风格重做
  - [ ] 10.1 Index.ets Tabs 浅色化
  - [ ] 10.2 各 page 卡片化与播放组件浮动样式

## 阶段四：交互与过渡优化
- [ ] Task 11: 四端页面切换与微交互动画优化（柔和过渡、卡片 hover/press 反馈）

# Task Dependencies
- Task 2 依赖现有 SourceManager（已具备）
- Task 3/4/5/6 依赖 Task 2
- Task 7/8/9/10 依赖 Task 1
- Task 11 依赖 Task 7~10
