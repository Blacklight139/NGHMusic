# Checklist

## Linux 工程脚手架
- [ ] `linux/Cargo.toml` 正确配置 music-core path 依赖、gtk4、libadwaita、gstreamer 依赖
- [ ] `linux/src/main.rs` GTK Application 可正常初始化并显示窗口
- [ ] 窗口标题与侧边栏品牌位显示「逆光音乐」
- [ ] 构建脚本正确链接 GTK4 + libadwaita

## 核心集成
- [x] `linux/src/core_service.rs` 直接调用 `music_core` Rust API（非 FFI）
- [x] 音源管理（import/validate/list/enable/disable/delete）功能完整可用
- [x] 搜索/元数据/播放URL/歌词/排行榜功能完整可用
- [x] 飞牛 NAS（login/list_files/stream/health）功能完整可用
- [x] 协议源（add/list/delete/list_files/read/stream）功能完整可用
- [x] 本地音乐（init/add_dir/rescan/progress）与缓存（init/stats/clear）功能完整可用

## GStreamer 播放器
- [ ] GStreamer pipeline 正确初始化
- [ ] 播放/暂停/上一首/下一首功能正常
- [ ] 进度跳转（seek）功能正常
- [ ] 音量控制功能正常
- [ ] 播放模式（顺序/单曲循环/随机）功能正常
- [ ] 播放结束自动下一首（EOS 信号处理）
- [ ] 播放状态持久化到 `~/.local/share/nghmusic/player_state.json`，重启可恢复

## 设计 Token 与主题
- [ ] `linux/src/theme.rs` 颜色/圆角/间距 Token 与 macOS AppTheme.swift 对齐
- [ ] CSS 样式正确应用到 GTK4 组件
- [ ] libadwaita 浅色主题作为基底
- [ ] 禁用 emoji，使用 GTK Symbolic Icons

## 主窗口布局
- [ ] 侧边栏导航包含全部功能页入口
- [ ] 侧边栏品牌头部 + 功能页列表 + 底部信息条
- [ ] 页面切换正常，无闪烁/卡顿
- [ ] 底部播放控制栏正确显示

## 功能页面
- [ ] 搜索页面：搜索框/结果列表/当前播放高亮
- [ ] 播放列表页面：队列展示/点击播放/清空/当前高亮
- [ ] 收藏夹页面：分组列表/添加移除
- [ ] 歌词页面：LRC 解析/同步滚动/高亮/点击跳转
- [ ] 排行榜页面：音源选择/榜单列表/播放
- [ ] 本地音乐页面：目录管理/扫描进度/库浏览
- [ ] NAS 页面：飞牛登录/健康检查/文件浏览/播放 + 协议源浏览/播放
- [ ] 设置页面：音源导入/列表管理/协议源管理/缓存管理/关于

## 播放控制栏
- [ ] 当前歌曲信息（封面占位 + 标题 + 艺术家）
- [ ] 播放控制按钮（上一首/播放暂停/下一首/播放模式）
- [ ] 进度条 + 时间显示
- [ ] 音量控制

## 全端代码审查
- [ ] core/ FFI 安全：裸指针/panic 跨边界/字符串释放配对
- [ ] core/ 并发竞态：缓存写/淘汰、文件监听、锁安全
- [ ] core/ 错误处理：无残留 unwrap/expect，错误传播完整
- [ ] macOS：KVO/Observer 正确释放，AVPlayer 无竞态
- [ ] Windows：P/Invoke 字符串释放/内存泄漏，MediaPlayer 资源释放
- [ ] iOS：AVPlayer/Swift 并发安全
- [ ] Android：ExoPlayer 生命周期/Compose 状态管理
- [ ] HarmonyOS：AVPlayer/NAPI 绑定安全
- [ ] 审查发现的问题已全部修复并回归验证

## UI 视觉改进
- [ ] macOS 端：视觉层级优化、留白/对齐改进、不必要卡片装饰移除
- [ ] Windows 端：间距 Token 统一、列表行视觉层级改进
- [ ] iOS 端：线性列表风格优化、当前播放高亮
- [ ] Android 端：搜索/列表行视觉统一
- [ ] HarmonyOS 端：列表行风格统一
- [ ] Linux 端：与改进后各端视觉一致
- [ ] 全端品牌存在感突出（第一屏幕即可识别「逆光音乐」）
- [ ] 全端交互反馈一致（按压/悬停/选中状态）
