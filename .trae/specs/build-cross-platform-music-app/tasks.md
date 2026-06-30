# Tasks

## 阶段一：架构与脚手架
- [x] Task 1: 搭建共享 Rust 核心骨架
  - [x] 1.1 创建 `core/` workspace（sources/metadata/search/cache/protocols/feiniu/ffi 模块）
  - [x] 1.2 定义核心数据模型（Song/Album/Artist/Lyric/SearchResult/Playlist）
  - [x] 1.3 定义 FFI 接口（错误处理、字符串/二进制传递规范）
- [x] Task 2: 搭建四套原生前端脚手架
  - [x] 2.1 桌面 Tauri 2 + Web UI（覆盖 Linux/Windows/macOS）
  - [x] 2.2 iOS SwiftUI 工程
  - [x] 2.3 Android Compose 工程
  - [x] 2.4 HarmonyOS ArkUI 工程
- [x] Task 3: 各端接入 Rust 核心 FFI 绑定
  - [x] 3.1 桌面（Tauri Rust sidecar / 插件）
  - [x] 3.2 iOS（Swift 绑定 via UniFFI 或 cbindgen）
  - [x] 3.3 Android（JNI/Kotlin 绑定）
  - [x] 3.4 HarmonyOS（NAPI 绑定）
- [x] Task 4: 各端接入原生音频播放组件
  - [x] 4.1 桌面系统音频组件
  - [x] 4.2 iOS AVPlayer
  - [x] 4.3 Android ExoPlayer/Media3
  - [x] 4.4 HarmonyOS AVPlayer

## 阶段二：音源系统
- [x] Task 5: 定义标准音源 JSON Schema
  - [x] 5.1 编写 `schemas/sound-source.schema.json`（基本信息/搜索/元数据/播放 URL/歌词/鉴权/字段映射）
  - [x] 5.2 在 Rust 核心实现 Schema 校验器
- [x] Task 6: 音源引擎（Rust）
  - [x] 6.1 音源加载/启用/禁用/优先级管理
  - [x] 6.2 元数据 API 客户端（HTTP 请求/解析/错误处理）
  - [x] 6.3 播放数据定位与获取
  - [x] 6.4 歌词获取
- [x] Task 7: 社区音源迁移/适配方案
  - [x] 7.1 实现社区格式适配层（导入时转换）
  - [x] 7.2 迁移结果提示与失败回退
- [x] Task 8: 设置内音源导入 UI（四端）
  - [x] 8.1 桌面导入入口
  - [x] 8.2 iOS 导入入口
  - [x] 8.3 Android 导入入口
  - [x] 8.4 HarmonyOS 导入入口

## 阶段三：核心功能
- [x] Task 9: 聚合搜索
  - [x] 9.1 Rust 核心跨音源聚合搜索（分类/分页）
  - [x] 9.2 四端搜索 UI（结果展示音源来源）
- [x] Task 10: 播放组件
  - [x] 10.1 播放/暂停/上一首/下一首/进度/音量/模式
  - [x] 10.2 播放状态持久化
- [x] Task 11: 播放列表
  - [x] 11.1 列表数据结构与持久化
  - [x] 11.2 添加/移除/清空/拖动排序（四端）
- [x] Task 12: 收藏夹
  - [x] 12.1 多分组数据结构与持久化
  - [x] 12.2 添加/移除/导入/导出（四端）
- [x] Task 13: 歌词滚动
  - [x] 13.1 LRC 解析与时间轴同步（Rust 核心）
  - [x] 13.2 歌词滚动/高亮/点击跳转 UI（四端）
- [x] Task 14: 排行榜
  - [x] 14.1 排行榜数据获取（Rust 核心）
  - [x] 14.2 排行榜展示与播放 UI（四端）
- [x] Task 15: 本地音乐播放
  - [x] 15.1 Rust 核心 `core/sources/local/`：目录递归扫描、扩展名过滤（mp3/flac/m4a/ape/ogg/wav/aac）
  - [x] 15.2 元数据解析（lofty：ID3v1/v2、FLAC、MP4、APE 标签），缺失字段以文件名回退
  - [x] 15.3 本地音乐库持久化索引（SQLite，路径/标题/艺术家/专辑/封面/时长）
  - [x] 15.4 文件夹监听增量更新（notify：新增/删除/移动 → 增量同步，不全量重扫）
  - [x] 15.5 本地音乐作为内置源接入聚合搜索/播放列表/收藏夹，结果标注「本地」来源
  - [x] 15.6 本地目录管理 UI（四端：添加/移除目录、扫描进度、重新扫描）
  - [x] 15.7 本地音乐库浏览 UI（四端：按歌曲/专辑/艺术家/文件夹浏览并播放）

## 阶段四：UI/UX
- [x] Task 16: 简约风格设计系统
  - [x] 16.1 色彩/字体/间距/图标规范文档
  - [x] 16.2 四端共享设计 token 落地
- [x] Task 17: UI 布局统一与视觉协调
  - [x] 17.1 各功能页布局对齐设计稿
  - [x] 17.2 响应式适配（桌面窗口/移动竖屏）
- [x] Task 18: 平滑页面切换
  - [x] 18.1 四端页面过渡动画
  - [x] 18.2 切换性能优化（无闪烁/卡顿）

## 阶段五：缓存与 NAS/协议
- [x] Task 19: 播放缓存层
  - [x] 19.1 Rust 核心缓存策略（LRU/容量上限/命中优先）
  - [x] 19.2 缓存管理 UI（容量/清理）
- [x] Task 20: 飞牛 API 集成
  - [x] 20.1 飞牛 API 客户端（Rust 核心，鉴权/列目录/取流）
  - [x] 20.2 NAS 配置与浏览播放 UI（四端）
  - [x] 20.3 异常提示与重试
- [x] Task 21: SMB 协议
- [x] Task 22: WebDAV 协议
- [x] Task 23: FTP 协议
- [x] Task 24: DLNA 协议
- [x] Task 25: NFS 协议
- [x] Task 26: 协议源管理 UI（四端，统一添加/浏览/播放）

## 阶段六：文档
- [x] Task 27: 音源开发文档
  - [x] 27.1 标准 JSON Schema 说明与示例
  - [x] 27.2 元数据 API / 搜索定位 / 缓存播放对接说明
  - [x] 27.3 社区音源迁移方案说明
- [x] Task 28: API 文档（仿 Apifox 规格，Markdown）
  - [x] 28.1 音源接口文档
  - [x] 28.2 飞牛接口文档
  - [x] 28.3 协议接口文档（含请求/响应示例与错误码）

## 阶段七：代码质量与 Bug 排查
- [x] Task 29: 配置静态分析工具链
  - [x] 29.1 Rust（clippy/rustfmt/miri）
  - [x] 29.2 桌面 Web（ESLint/TS/Prettier）
  - [x] 29.3 iOS（SwiftLint）、Android（Lint/Detekt）、HarmonyOS（代码扫描）
- [x] Task 30: 边界条件与错误处理审查
- [x] Task 31: 内存泄漏排查（Rust 生命周期/循环引用；各端资源释放）
- [x] Task 32: 状态管理与异步竞态排查
- [x] Task 33: 兼容性与性能瓶颈测试（六端真机/模拟）
- [x] Task 34: 编写 Bug 报告（复现步骤/预期/实际/优先级）并分类
- [x] Task 35: Bug 修复与回归验证

# Task Dependencies
- Task 2 依赖 Task 1
- Task 3 依赖 Task 1
- Task 4 依赖 Task 2
- Task 5/6 依赖 Task 1
- Task 7 依赖 Task 5、Task 6
- Task 8 依赖 Task 2、Task 6
- Task 9~14 依赖 Task 6、Task 8
- Task 15（本地音乐）依赖 Task 1、Task 4
- Task 16~18 与 Task 9~15 可部分并行
- Task 19 依赖 Task 6
- Task 20~25 依赖 Task 1、Task 4
- Task 26 依赖 Task 20~25
- Task 27、28 依赖 Task 5、Task 6、Task 15、Task 20
- Task 29~35 依赖各功能任务完成
