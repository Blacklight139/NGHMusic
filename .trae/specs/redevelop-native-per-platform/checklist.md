# Checklist

## 框架推翻与共享核心
- [x] `desktop/`（Tauri 2 + Web UI）目录已删除，Linux 端支持已移除
- [x] `windows/`、`macos/` 顶层目录已新建；`ios/`、`android/`、`harmonyos/` 已重建对齐新框架
- [x] `docs/` 文件夹内容未被修改（与重开发前逐文件一致）
- [x] 共享 Rust 核心 `core/` 对外契约（数据结构/错误类型/接口语义）与 docs 一致
- [x] C# FFI 绑定（Windows）、Swift FFI 绑定（macOS/iOS）、JNI（Android）、NAPI（HarmonyOS）均已接入
- [x] FFI 裸指针解引用函数标记 `unsafe` 并附 `# Safety` 文档
- [x] FFI 入口不触发 Rust panic（无 `unwrap`/`expect`/`panic!` 跨边界）

## 五端原生架构
- [x] Windows 端使用 WinUI 3（C#/.NET 8），无 Web 套壳
- [x] macOS 端使用 SwiftUI（Swift），无 Web 套壳
- [x] iOS 端使用 SwiftUI（Swift），无 Web 套壳
- [x] Android 端使用 Jetpack Compose（Kotlin），无 Web 套壳
- [x] HarmonyOS 端使用 ArkUI（ArkTS），无 Web 套壳
- [x] 五端各自接入原生音频播放组件（Windows 媒体组件/AVPlayer/AVPlayer/Media3/系统 AVPlayer）
- [x] 五端窗口标题/品牌位显示统一品牌名

## 功能一致性（对齐 docs）
- [x] 音源管理：导入（文件上传 .json）/校验/启用/禁用/删除/列表，五端行为一致且对齐 sound-source-api.md
- [x] 聚合搜索：跨音源、分类（歌曲/专辑/艺术家）、分页，单源失败跳过，五端一致
- [x] 歌曲元数据/播放URL/歌词获取，错误码与 docs 一致，五端一致
- [x] 排行榜：展示与播放，本地源返回空数组，五端一致
- [x] 播放组件：播放/暂停/上一首/下一首/进度/音量/模式（顺序/单曲循环/随机）+ 状态持久化，五端一致
- [x] 播放列表：添加/移除/清空/拖动排序 + 持久化，五端一致
- [x] 收藏夹：多分组/添加/移除/导入/导出 + 持久化，五端一致
- [x] 歌词滚动：LRC 解析/同步/高亮/点击跳转/翻译，五端一致
- [x] 本地音乐：扫描/元数据解析/SQLite 索引/文件夹监听增量更新/作为内置源接入，五端一致
- [x] 播放缓存：LRU/容量上限/命中优先/清理，五端一致
- [x] 飞牛 NAS：登录/列目录/流URL/健康检查，错误码与 feiniu-api.md 一致，五端一致
- [x] 网络协议：SMB/WebDAV/FTP/DLNA/NFS 源管理与浏览播放，占位协议错误信息与 protocol-api.md 一致，五端一致
- [x] 社区音源迁移：标准/community-a/community-b 适配 + warnings 报告，五端一致

## 编写指南落地
- [x] 目录结构：五端各自顶层目录 + `core/` + `docs/`（只读）+ `schemas/`
- [x] 命名规范：Rust/C#/Swift/Kotlin/ArkTS 各自规范已执行
- [x] 功能对齐基准：以 docs 为唯一来源，新增/修改功能前核对 docs
- [x] 音源规范：JSON 符合 `schemas/sound-source.schema.json`，导入走文件上传 + 校验/适配
- [x] 错误处理：跨 FFI 不 panic，CoreError 映射为 Result，UI 层提示 + 重试
- [x] 异步与并发：阻塞 IO 经 spawn_blocking，缓存/监听不持锁做 IO，竞态路径二次检查
- [x] 资源释放：播放器/watcher/连接显式 release/dispose，FFI 分配内存配对释放
- [x] 图标规范：全端禁用 emoji，使用 Fluent/SF Symbols/Material/系统或 SVG 图标
- [x] 状态持久化：播放/列表/收藏/音源/协议源/本地目录/缓存索引均可重启恢复
- [x] 各端静态分析工具链已配置并可运行

## 代码审查与潜在漏洞
- [x] 各端静态分析全量运行通过（clippy/rustfmt、C# analyzers、SwiftLint、Lint/Detekt、HarmonyOS 扫描）
- [x] FFI 安全审查：裸指针/panic 跨边界、字符串/字节释放配对，无 UB
- [x] 鉴权与凭据审查：token/密码不明文持久化、传输保护、飞牛/协议源凭据保护
- [x] 路径遍历审查：本地音乐目录、协议源路径校验，无越权访问
- [x] SSRF 审查：音源 URL、协议源 URL 校验，无内网探测风险
- [x] 并发竞态审查：缓存写/淘汰、文件监听、播放器状态无数据竞争
- [x] 资源释放审查：播放器/watcher/连接无泄漏
- [x] 状态一致性与边界值审查：空 playUrl 处理、分页边界、超时、空关键字
- [x] 漏洞报告已产出（复现步骤/预期/实际/严重级别/分类）
- [x] 漏洞已分类修复并回归验证通过
