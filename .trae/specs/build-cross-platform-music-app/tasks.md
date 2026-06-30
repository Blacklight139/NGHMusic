# Tasks

## 阶段一：架构与脚手架
- [x] Task 1: 搭建共享 Rust 核心骨架
  - [x] 1.1 创建 `core/` workspace（sources/metadata/search/cache/protocols/feiniu/ffi 模块）
  - [x] 1.2 定义核心数据模型（Song/Album/Artist/Lyric/SearchResult/Playlist）
  - [x] 1.3 定义 FFI 接口（错误处理、字符串/二进制传递规范）
- [ ] Task 2: 搭建四套原生前端脚手架
  - [ ] 2.1 桌面 Tauri 2 + Web UI（覆盖 Linux/Windows/macOS）
  - [ ] 2.2 iOS SwiftUI 工程
  - [ ] 2.3 Android Compose 工程
  - [ ] 2.4 HarmonyOS ArkUI 工程
- [ ] Task 3: 各端接入 Rust 核心 FFI 绑定
  - [ ] 3.1 桌面（Tauri Rust sidecar / 插件）
  - [ ] 3.2 iOS（Swift 绑定 via UniFFI 或 cbindgen）
  - [ ] 3.3 Android（JNI/Kotlin 绑定）
  - [ ] 3.4 HarmonyOS（NAPI 绑定）
- [ ] Task 4: 各端接入原生音频播放组件
  - [ ] 4.1 桌面系统音频组件
  - [ ] 4.2 iOS AVPlayer
  - [ ] 4.3 Android ExoPlayer/Media3
  - [ ] 4.4 HarmonyOS AVPlayer

## 阶段二：音源系统
- [ ] Task 5: 定义标准音源 JSON Schema
  - [ ] 5.1 编写 `schemas/sound-source.schema.json`（基本信息/搜索/元数据/播放 URL/歌词/鉴权/字段映射）
  - [ ] 5.2 在 Rust 核心实现 Schema 校验器
- [ ] Task 6: 音源引擎（Rust）
  - [ ] 6.1 音源加载/启用/禁用/优先级管理
  - [ ] 6.2 元数据 API 客户端（HTTP 请求/解析/错误处理）
  - [ ] 6.3 播放数据定位与获取
  - [ ] 6.4 歌词获取
- [ ] Task 7: 社区音源迁移/适配方案
  - [ ] 7.1 实现社区格式适配层（导入时转换）
  - [ ] 7.2 迁移结果提示与失败回退
- [ ] Task 8: 设置内音源导入 UI（四端）
  - [ ] 8.1 桌面导入入口
  - [ ] 8.2 iOS 导入入口
  - [ ] 8.3 Android 导入入口
  - [ ] 8.4 HarmonyOS 导入入口

## 阶段三：核心功能
- [ ] Task 9: 聚合搜索
  - [ ] 9.1 Rust 核心跨音源聚合搜索（分类/分页）
  - [ ] 9.2 四端搜索 UI（结果展示音源来源）
- [ ] Task 10: 播放组件
  - [ ] 10.1 播放/暂停/上一首/下一首/进度/音量/模式
  - [ ] 10.2 播放状态持久化
- [ ] Task 11: 播放列表
  - [ ] 11.1 列表数据结构与持久化
  - [ ] 11.2 添加/移除/清空/拖动排序（四端）
- [ ] Task 12: 收藏夹
  - [ ] 12.1 多分组数据结构与持久化
  - [ ] 12.2 添加/移除/导入/导出（四端）
- [ ] Task 13: 歌词滚动
  - [ ] 13.1 LRC 解析与时间轴同步（Rust 核心）
  - [ ] 13.2 歌词滚动/高亮/点击跳转 UI（四端）
- [ ] Task 14: 排行榜
  - [ ] 14.1 排行榜数据获取（Rust 核心）
  - [ ] 14.2 排行榜展示与播放 UI（四端）

## 阶段四：UI/UX
- [ ] Task 15: 简约风格设计系统
  - [ ] 15.1 色彩/字体/间距/图标规范文档
  - [ ] 15.2 四端共享设计 token 落地
- [ ] Task 16: UI 布局统一与视觉协调
  - [ ] 16.1 各功能页布局对齐设计稿
  - [ ] 16.2 响应式适配（桌面窗口/移动竖屏）
- [ ] Task 17: 平滑页面切换
  - [ ] 17.1 四端页面过渡动画
  - [ ] 17.2 切换性能优化（无闪烁/卡顿）

## 阶段五：缓存与 NAS/协议
- [ ] Task 18: 播放缓存层
  - [ ] 18.1 Rust 核心缓存策略（LRU/容量上限/命中优先）
  - [ ] 18.2 缓存管理 UI（容量/清理）
- [ ] Task 19: 飞牛 API 集成
  - [ ] 19.1 飞牛 API 客户端（Rust 核心，鉴权/列目录/取流）
  - [ ] 19.2 NAS 配置与浏览播放 UI（四端）
  - [ ] 19.3 异常提示与重试
- [ ] Task 20: SMB 协议
- [ ] Task 21: WebDAV 协议
- [ ] Task 22: FTP 协议
- [ ] Task 23: DLNA 协议
- [ ] Task 24: NFS 协议
- [ ] Task 25: 协议源管理 UI（四端，统一添加/浏览/播放）

## 阶段六：文档
- [ ] Task 26: 音源开发文档
  - [ ] 26.1 标准 JSON Schema 说明与示例
  - [ ] 26.2 元数据 API / 搜索定位 / 缓存播放对接说明
  - [ ] 26.3 社区音源迁移方案说明
- [ ] Task 27: API 文档（仿 Apifox 规格，Markdown）
  - [ ] 27.1 音源接口文档
  - [ ] 27.2 飞牛接口文档
  - [ ] 27.3 协议接口文档（含请求/响应示例与错误码）

## 阶段七：代码质量与 Bug 排查
- [ ] Task 28: 配置静态分析工具链
  - [ ] 28.1 Rust（clippy/rustfmt/miri）
  - [ ] 28.2 桌面 Web（ESLint/TS/Prettier）
  - [ ] 28.3 iOS（SwiftLint）、Android（Lint/Detekt）、HarmonyOS（代码扫描）
- [ ] Task 29: 边界条件与错误处理审查
- [ ] Task 30: 内存泄漏排查（Rust 生命周期/循环引用；各端资源释放）
- [ ] Task 31: 状态管理与异步竞态排查
- [ ] Task 32: 兼容性与性能瓶颈测试（六端真机/模拟）
- [ ] Task 33: 编写 Bug 报告（复现步骤/预期/实际/优先级）并分类
- [ ] Task 34: Bug 修复与回归验证

# Task Dependencies
- Task 2 依赖 Task 1
- Task 3 依赖 Task 1
- Task 4 依赖 Task 2
- Task 5/6 依赖 Task 1
- Task 7 依赖 Task 5、Task 6
- Task 8 依赖 Task 2、Task 6
- Task 9~14 依赖 Task 6、Task 8
- Task 15~17 与 Task 9~14 可部分并行
- Task 18 依赖 Task 6
- Task 19~24 依赖 Task 1、Task 4
- Task 25 依赖 Task 19~24
- Task 26、27 依赖 Task 5、Task 6、Task 19
- Task 28~34 依赖各功能任务完成
