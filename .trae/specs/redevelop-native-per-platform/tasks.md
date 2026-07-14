# Tasks

## 阶段一：框架推翻与共享核心重构
- [x] Task 1: 移除旧 Web 桌面框架并确立新五端原生骨架
  - [x] 1.1 删除 `desktop/`（Tauri 2 + Web UI）整体目录，移除 Linux 端支持
  - [x] 1.2 新建顶层目录 `windows/`、`macos/`，保留并清理 `ios/`、`android/`、`harmonyos/`、`core/`、`docs/`、`schemas/`
  - [x] 1.3 核对 `docs/` 不被修改，确认其为功能与音源规范唯一来源
- [x] Task 2: 共享 Rust 核心 `core/` 对齐 docs 契约并扩展 FFI
  - [x] 2.1 核对 `core/` 对外数据结构/错误类型/接口语义与 docs（sound-source-api / feiniu-api / protocol-api / sound-source-development）一致
  - [x] 2.2 新增 C# FFI 绑定（uniffi-cs 或 P/Invoke 头文件）供 Windows 调用
  - [x] 2.3 新增/维护 Swift FFI 绑定（uniffi-swift）供 macOS/iOS 调用
  - [x] 2.4 维护 JNI 绑定（Android）与 NAPI 绑定（HarmonyOS）
  - [x] 2.5 FFI 安全整改：裸指针解引用函数标 `unsafe` + `# Safety`；FFI 入口不 panic（`unwrap_or_default` 回退）
- [x] Task 3: 配置各端静态分析工具链
  - [x] 3.1 Rust（clippy/rustfmt，延续 `core/clippy.toml`、`core/rustfmt.toml`）
  - [x] 3.2 Windows（C# analyzers / .editorconfig）
  - [x] 3.3 macOS/iOS（SwiftLint，延续 `.swiftlint.yml`）
  - [x] 3.4 Android（Lint/Detekt，延续 `android/detekt.yml`）
  - [x] 3.5 HarmonyOS（代码扫描，延续 `harmonyos/code-scan.yml`）

## 阶段二：五端原生脚手架与音频接入
- [x] Task 4: Windows 原生客户端脚手架（WinUI 3 / C# / .NET 8）
  - [x] 4.1 创建 WinUI 3 工程（Windows App SDK），窗口标题显示品牌名
  - [x] 4.2 接入 Rust 核心 FFI（P/Invoke 或 uniffi-cs）
  - [x] 4.3 接入 Windows 原生媒体组件（MediaPlayer/Media Foundation）播放
- [x] Task 5: macOS 原生客户端脚手架（SwiftUI / Swift）
  - [x] 5.1 创建 macOS SwiftUI 工程（AppKit 互操作），窗口标题显示品牌名
  - [x] 5.2 接入 Rust 核心 FFI（uniffi-swift 绑定）
  - [x] 5.3 接入 AVPlayer 播放
- [x] Task 6: iOS 原生客户端脚手架（SwiftUI / Swift）重建
  - [x] 6.1 重建 iOS SwiftUI 工程，对齐新框架
  - [x] 6.2 接入 Rust 核心 FFI（uniffi-swift）
  - [x] 6.3 接入 AVPlayer 播放
- [x] Task 7: Android 原生客户端脚手架（Jetpack Compose / Kotlin）重建
  - [x] 7.1 重建 Android Compose 工程，对齐新框架
  - [x] 7.2 接入 Rust 核心 FFI（JNI/Kotlin）
  - [x] 7.3 接入 Media3/ExoPlayer 播放
- [x] Task 8: HarmonyOS 原生客户端脚手架（ArkUI / ArkTS）重建
  - [x] 8.1 重建 HarmonyOS ArkUI 工程，对齐新框架
  - [x] 8.2 接入 Rust 核心 FFI（NAPI）
  - [x] 8.3 接入系统 AVPlayer 播放

## 阶段三：音源系统（五端 + 核心）
- [x] Task 9: 核心音源引擎对齐 docs
  - [x] 9.1 核对 `schemas/sound-source.schema.json` 与 docs 一致；Schema 校验器可用
  - [x] 9.2 音源加载/启用/禁用/优先级/删除（`local` 受保护不可删）
  - [x] 9.3 元数据/播放URL/歌词/排行榜 API 客户端，错误映射与 docs 错误码一致
  - [x] 9.4 社区音源适配层（标准 / community-a / community-b）+ 迁移报告 warnings
- [x] Task 10: 五端音源导入与管理 UI（文件上传 + LXMusic 列表）
  - [x] 10.1 Windows 音源导入（文件选择 .json）+ 列表管理（开关/排序/编辑/删除）
  - [x] 10.2 macOS 音源导入与列表管理
  - [x] 10.3 iOS 音源导入与列表管理
  - [x] 10.4 Android 音源导入与列表管理
  - [x] 10.5 HarmonyOS 音源导入与列表管理

## 阶段四：核心功能（五端功能一致，对齐 docs）
- [x] Task 11: 聚合搜索（五端）
  - [x] 11.1 核心跨音源聚合搜索（分类/分页，单源失败跳过）
  - [x] 11.2 五端搜索 UI（结果展示音源来源，行为一致）
- [x] Task 12: 播放组件与状态持久化（五端）
  - [x] 12.1 播放/暂停/上一首/下一首/进度/音量/模式（顺序/单曲循环/随机）
  - [x] 12.2 播放状态持久化，重启可恢复
- [x] Task 13: 播放列表（五端）
  - [x] 13.1 列表数据结构与持久化
  - [x] 13.2 添加/移除/清空/拖动排序（五端行为一致）
- [x] Task 14: 收藏夹（五端）
  - [x] 14.1 多分组数据结构与持久化
  - [x] 14.2 添加/移除/导入/导出（五端行为一致）
- [x] Task 15: 歌词滚动（五端）
  - [x] 15.1 核心 LRC 解析与时间轴同步
  - [x] 15.2 歌词滚动/高亮/点击跳转/翻译 UI（五端行为一致）
- [x] Task 16: 排行榜（五端）
  - [x] 16.1 核心排行榜数据获取（本地源返回空数组）
  - [x] 16.2 排行榜展示与播放 UI（五端行为一致）
- [x] Task 17: 本地音乐（五端）
  - [x] 17.1 核心本地源：递归扫描、扩展名过滤、lofty 元数据解析、SQLite 索引、notify 增量监听
  - [x] 17.2 本地目录管理 UI（五端：添加/移除/扫描进度/重扫）
  - [x] 17.3 本地音乐库浏览 UI（五端：歌曲/专辑/艺术家/文件夹）
  - [x] 17.4 本地音乐作为内置源接入搜索/播放列表/收藏夹，标注「本地」来源
- [x] Task 18: 播放缓存（五端）
  - [x] 18.1 核心 LRU 缓存（命中优先、容量上限、淘汰不持锁做 IO、写前二次检查防竞态）
  - [x] 18.2 缓存管理 UI（容量/清理，五端行为一致）

## 阶段五：NAS 与网络协议（五端）
- [x] Task 19: 飞牛 NAS API（五端）
  - [x] 19.1 核心飞牛客户端（登录/列目录/流URL/健康检查），错误码与 docs 一致
  - [x] 19.2 NAS 配置与浏览播放 UI（五端），异常提示与重试
- [x] Task 20: 网络协议 SMB/WebDAV/FTP/DLNA/NFS（五端）
  - [x] 20.1 核心协议客户端（WebDAV/FTP 完整实现，SMB/DLNA/NFS 占位与错误信息对齐 docs）
  - [x] 20.2 协议源管理 UI（五端：添加/浏览/播放，行为一致）

## 阶段六：UI/UX 与编写指南落地
- [x] Task 21: 各端原生设计语言落地（UI 可不统一）
  - [x] 21.1 Windows Fluent / macOS+iOS HIG / Android Material 3 / HarmonyOS Design 各自落地
  - [x] 21.2 页面切换平滑稳定（无闪烁/卡顿）
- [x] Task 22: 编写指南落地
  - [x] 22.1 五端命名规范、目录结构对齐 spec 编写指南
  - [x] 22.2 全端禁用 emoji，接入各端图标库（Fluent/SF Symbols/Material/系统或 SVG）
  - [x] 22.3 状态持久化覆盖（播放/列表/收藏/音源/协议源/本地目录/缓存索引）

## 阶段七：代码审查与潜在漏洞排查
- [x] Task 23: 各端静态分析全量运行并修复
  - [x] 23.1 Rust clippy/rustfmt、C# analyzers、SwiftLint、Android Lint/Detekt、HarmonyOS 代码扫描
- [x] Task 24: 系统性代码审查寻找潜在漏洞
  - [x] 24.1 FFI 安全：裸指针/panic 跨边界、字符串/字节释放配对
  - [x] 24.2 鉴权与凭据：token/密码明文存储与传输、飞牛/协议源凭据保护
  - [x] 24.3 路径遍历：本地音乐目录、协议源路径校验
  - [x] 24.4 SSRF：音源 URL、协议源 URL 校验
  - [x] 24.5 并发竞态：缓存写/淘汰、文件监听、播放器状态
  - [x] 24.6 资源释放：各端播放器/watcher/连接显式释放
  - [x] 24.7 状态一致性与边界值：播放器空 URL 处理、分页边界、超时
- [x] Task 25: 产出漏洞报告并修复回归
  - [x] 25.1 记录漏洞报告（复现步骤/预期/实际/严重级别/分类）
  - [x] 25.2 分类修复并回归验证

# Task Dependencies
- Task 2 依赖 Task 1
- Task 3 与 Task 2 可部分并行
- Task 4~8（五端脚手架）依赖 Task 2
- Task 9 依赖 Task 2；Task 10 依赖 Task 4~8、Task 9
- Task 11~18 依赖 Task 9、Task 10（五端可并行实现各自功能）
- Task 19~20 依赖 Task 2、Task 4~8
- Task 21~22 与 Task 11~20 可部分并行
- Task 23~25 依赖各功能任务完成
