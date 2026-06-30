# Bug 报告 — 跨平台音乐播放器代码质量排查

> 范围：`/workspace/core`（Rust 核心）的代码质量与潜在 Bug 排查；移动端（iOS / Android / HarmonyOS）做静态审查记录。
> 排查工具：`cargo clippy --all-targets -- -W clippy::all -W clippy::pedantic` + 人工审查。
> 基线测试：修复前 `cargo test` 81 passed / 0 failed。

---

## 一、Clippy 警告统计

运行 `cargo clippy --all-targets -- -W clippy::all -W clippy::pedantic` 共产生 **257 条** 诊断（lib 219 + lib test 去重后约 38 条独有）。按 lint 类别（去重）的主要分布：

| 类别 | 数量 | 说明 |
| --- | --- | --- |
| `doc_markdown` | 44 | 文档中技术名词未用反引号包裹 |
| `missing_errors_doc` | 40 | 返回 `Result` 的公开函数缺 `# Errors` 文档段 |
| `uninlined_format_args` | 36 | `format!("x={}", v)` 应改为 `format!("x={v}")` |
| `redundant_closure_for_method_calls` | 21 | `.map(\|s\| s.foo())` 应直接 `.map(Type::foo)` |
| `missing_panics_doc` | 17 | 含 `unwrap/expect/panic!` 的公开函数缺 `# Panics` 文档 |
| `must_use_candidate` | 14 | 返回新建 `Vec`/`Self` 的方法建议加 `#[must_use]` |
| `cast_possible_truncation` | 11 | `as` 转换可能截断（如 `usize`→`u32`） |
| `manual_let_else` | 10 | 可用 `let ... else` 简化 |
| `cast_possible_wrap` / `cast_sign_loss` | 17 | 有符号/无符号转换潜在风险 |
| `map_unwrap_or` / `derivable_impls` / `unreadable_literal` 等 | 14 | 杂项风格 |
| **`not_unsafe_ptr_arg_deref`** | **1** | **FFI 安全性**：`music_core_free_string` 解引用裸指针但未标 `unsafe` |
| **`unused_async`** | **1** | `FeiniuClient::get_stream_url` 标了 `async` 但无 `await` |
| **`wildcard_imports`** | **1** | `library.rs` 中 `use crate::models::*;` |

其中标粗的 3 条与「未处理异常 / 兼容性 / 性能」直接相关，已在 Bug 列表中追踪。

---

## 二、Bug 列表

> 严重级别：Critical > High > Medium > Low
> 分类：未处理异常 / 内存泄漏 / 状态不一致 / 竞态 / 边界值 / 兼容性 / 性能

---

### BUG-001

- **标题**：`LocalSource::add_directory` 在 `start_watch` 或前序加锁失败时不重置 `scanning` 标志
- **严重级别**：High
- **分类**：状态不一致
- **复现步骤**：
  1. 构造一个 `LocalSource`（临时目录）。
  2. 调用 `add_directory(dir)`，其中 `dir` 是一个 notify 无法监听的路径（例如不存在的目录、或权限受限路径），使 `start_watch` 返回 `Err`。
  3. 调用 `scan_progress()`。
- **预期行为**：扫描失败后 `scanning` 应回到 `false`，`scan_progress` 返回 `scanning=false`。
- **实际行为**：`start_watch(&dir)?` 通过 `?` 提前返回，`scanning.store(false, ...)` 永不执行，`scan_progress` 报告 `scanning=true` 直至进程结束。同理 `root_dirs.lock()` 与 `db.lock()` 失败也会卡死状态。`rescan` 存在相同问题。
- **根因分析**：函数用 `?` 提前返回但未保证 `scanning` 标志回退；缺少 RAII 守卫或 `finally` 语义。
- **修复方案**：将主体抽出为内部函数，外层统一 `store(true)` → 执行 → `store(false)` → 返回内部结果，保证任意错误路径都复位标志。`rescan` 存在相同问题，一并按同模式抽出 `rescan_inner` 修复。
- **修复状态**：已修复（见 `core/src/sources/local.rs`，`add_directory`/`add_directory_inner` 与 `rescan`/`rescan_inner`）

---

### BUG-002

- **标题**：`Library` 所有方法使用 `self.db.lock().unwrap()`，Mutex 中毒时级联 panic
- **严重级别**：High
- **分类**：未处理异常 / 状态不一致
- **复现步骤**：
  1. 任一持有 `Library` 内部 `Mutex<Connection>` 的线程在持锁期间 panic（例如 `rusqlite` 触发底层 panic）。
  2. 任何后续调用 `create_playlist` / `add_to_playlist` / `save_play_state` 等 16 处方法的请求。
- **预期行为**：Mutex 中毒后应返回 `Err`，由上层决定是否重建库。
- **实际行为**：`self.db.lock().unwrap()` 直接 panic，引发整条调用链级联崩溃，进程不可恢复。
- **根因分析**：`std::sync::Mutex::lock()` 返回 `Result`，`.unwrap()` 在 `PoisonError` 时 panic；项目其他模块（`cache.rs`、`local.rs`）已用 `map_err` 转换，唯独 `library.rs` 全部使用 `unwrap()`。
- **修复方案**：统一替换为 `self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?`。
- **修复状态**：已修复（见 `core/src/library.rs`）

---

### BUG-003

- **标题**：`ffi::music_core_free_string` 接收并解引用裸指针但未标记 `unsafe`
- **严重级别**：High
- **分类**：兼容性 / 未定义行为
- **复现步骤**：
  1. C/移动端宿主传入一个非 `CString::into_raw` 产生的指针（如栈地址、已释放的指针、错配 `free` 的指针）。
  2. 调用 `music_core_free_string(ptr)`。
- **预期行为**：函数应标记为 `unsafe extern "C" fn`，强制调用方在 `unsafe` 块内显式承诺指针来源合法；同时 `# Safety` 文档说明前置条件。
- **实际行为**：函数签名是 `pub extern "C" fn music_core_free_string(ptr: *mut c_char)`，调用方无需 `unsafe` 即可调用；clippy 报 `not_unsafe_ptr_arg_deref`。错误指针会触发未定义行为（UB），且编译器无法在调用点提示风险。
- **根因分析**：FFI 边界未将「解引用裸指针」的不变性显式表达为 `unsafe`。
- **修复方案**：将签名改为 `pub unsafe extern "C" fn music_core_free_string(ptr: *mut c_char)`，并在 `# Safety` 段说明 `ptr` 必须为 NULL 或 `CString::into_raw` 产物。
- **修复状态**：已修复（见 `core/src/ffi.rs`）

---

### BUG-004

- **标题**：`ffi::music_core_version` 用 `expect` 在版本号含 NUL 时 panic，跨 FFI 边界 panic 行为未定义
- **严重级别**：Medium
- **分类**：未处理异常
- **复现步骤**：
  1. 设想 `Cargo.toml` 的 `version` 字段被误填入含 NUL 的字符串（虽然 Cargo 校验通常阻止，但防御性编程应处理）。
  2. 调用 `music_core_version()`。
- **预期行为**：FFI 入口对调用方（C/移动端）不应触发 Rust panic（unwinding 跨 FFI 是 UB）；异常输入应回退到空串或占位值。
- **实际行为**：`CString::new(version).expect("crate 版本号不应包含 NUL")` 触发 panic，在 C 栈帧上行为未定义。
- **根因分析**：FFI 入口对不可能但「理论上可能」的输入采用 panic 而非优雅降级。
- **修复方案**：改为 `CString::new(version).unwrap_or_default()`，遇 NUL 时返回空 `CString`（即仅 NUL 终止符的 C 空串），保证不 panic。
- **修复状态**：已修复（见 `core/src/ffi.rs`）

---

### BUG-005

- **标题**：`CacheManager::get_or_fetch` 未命中 → `fetcher().await` → 写文件路径上存在并发竞态
- **严重级别**：Medium
- **分类**：竞态
- **复现步骤**：
  1. 同一 `CacheManager` 实例，对同一 `key` 并发调用两次 `get_or_fetch(key, fetcher)`。
  2. 两个调用同时通过「缓存命中」检查（均未命中）。
- **预期行为**：仅一次下载、一次写文件、一次插入索引；两个调用返回相同路径。
- **实际行为**：两个调用各自 `fetcher().await` 下载；两个并发 `fs::write` 写同一 `file_path`，可能产生损坏的缓存文件；两次 `index.insert` 与 `current_bytes` 增减虽最终一致（第二次 insert 替换第一次并减去旧 size），但磁盘文件内容不确定（最后写者胜出，过程可能交错）。
- **根因分析**：检查-下载-写入非原子；缺少 per-key 锁或写入后的二次命中检查。
- **修复方案**：在 `fetcher().await` 返回后、写文件前，再次获取索引锁检查是否已被其他并发请求填充；若已存在则直接返回已有路径并跳过写入。该改动显著缩小竞态窗口（仅极快的 fetcher 仍可能竞态），且不引入 per-key 锁的开销；残余竞态在文档中标注。
- **修复状态**：已修复（部分缓解；见 `core/src/cache.rs`）

---

### BUG-006

- **标题**：`CacheManager::evict_if_needed` 在持有索引锁时调用 `fs::remove_file` 同步 IO
- **严重级别**：Medium
- **分类**：性能
- **复现步骤**：
  1. 缓存接近 `max_bytes`，连续插入触发 `evict_if_needed`。
  2. 此时另一个线程调用 `get_or_fetch` / `get_path` / `has`。
- **预期行为**：索引锁仅保护内存数据结构，文件 IO 应在锁外执行，避免阻塞所有并发缓存查询。
- **实际行为**：`evict_if_needed` 在 `for entry in entries` 循环中持有 `index` 锁并调用 `fs::remove_file(&entry.file_path)`；大批量淘汰时所有缓存操作被阻塞。
- **根因分析**：淘汰循环未将「从索引移除」与「磁盘文件删除」解耦。
- **修复方案**：在锁内收集待淘汰条目（含 `file_path`）并从 `index` 移除、更新 `current_bytes`；释放锁后再循环 `fs::remove_file`。索引已无这些条目，并发 `get_or_fetch` 不会返回它们。
- **修复状态**：已修复（见 `core/src/cache.rs`）

---

### BUG-007

- **标题**：FTP 客户端无连接超时，且失败路径不调用 `quit()` 释放连接
- **严重级别**：Medium
- **分类**：性能 / 内存泄漏
- **复现步骤**：
  1. 配置一个不可达的 FTP 服务器地址。
  2. 调用 `FtpClient::list("/music")`。
- **预期行为**：在合理超时（如 10s）内返回 `Err`；不长时间占用 `spawn_blocking` 线程；连接资源被释放。
- **实际行为**：`FtpStream::connect(addr)` 阻塞等待 OS TCP SYN 重试（Linux 默认约 75s+），耗尽 tokio 阻塞线程池；登录或 LIST 失败时 `?` 提前返回，`ftp.quit()` 不会被执行，TCP 连接泄漏直至 drop。
- **根因分析**：未使用 suppaftp 提供的 `connect_timeout`；缺少类似 `defer` / `Drop` 的清理逻辑。
- **修复方案**：使用 `suppaftp::FtpStream::connect_timeout(socket_addr, Duration::from_secs(10))`（先解析地址），并将 `quit()` 抽到清理闭包中确保错误路径也执行。
- **修复状态**：已修复（见 `core/src/protocols/ftp.rs`）

---

### BUG-008

- **标题**：`JsonSource::request_leaderboards` 使用 `expect` 在 leaderboards 端点缺失时 panic
- **严重级别**：Medium
- **分类**：未处理异常
- **复现步骤**：
  1. 构造一个 `SoundSourceConfig`，其 `endpoints.leaderboards = None`。
  2. 由于 `get_leaderboards` 已校验 `is_none()` 直接返回空数组，正常路径不会触发；但若调用方直接调用 `request_leaderboards`（例如未来重构改动），则触发 panic。
- **预期行为**：返回 `Err(CoreError::Source(...))`，由调用方处理。
- **实际行为**：`expect("调用方应保证 leaderboards 存在")` panic。
- **根因分析**：将「调用方契约」用 panic 而非 `Result` 强制。
- **修复方案**：改为 `ok_or_else(|| CoreError::Source("leaderboards 端点未配置".into()))?`。
- **修复状态**：已修复（见 `core/src/sources/json.rs`）

---

### BUG-009

- **标题**：`LocalSource::search` 先全表加载所有匹配歌曲到 `Vec<Song>` 再做内存分页
- **严重级别**：Low
- **分类**：性能
- **复现步骤**：
  1. 本地库索引 10 万首歌曲。
  2. 调用 `search("", 1, 20)`（空关键字匹配全部）。
- **预期行为**：仅从 DB 读取当前页的 20 条记录。
- **实际行为**：`search_db` 执行 `SELECT ... ORDER BY title`（无 LIMIT）将所有匹配行加载到内存，再 `skip(offset).take(page_size)`；大库下内存峰值高、延迟大。
- **根因分析**：分页在 Rust 侧而非 SQL 侧完成。
- **修复方案**：在 `search_db` 中增加 `LIMIT page_size OFFSET offset` 参数（保留 `total` 的全量计数查询或单独 `COUNT(*)`）。该改动较大，**待修复**（避免破坏现有测试与排序语义）。
- **修复状态**：待修复

---

### BUG-010

- **标题**：`LocalSource::remove_directory` 的 LIKE 模式硬编码 `/`，在 Windows 上无法匹配反斜杠路径
- **严重级别**：Medium
- **分类**：兼容性
- **复现步骤**：
  1. 在 Windows 上构造 `LocalSource`，`add_directory("C:\\Users\\foo\\music")`。
  2. 调用 `remove_directory("C:\\Users\\foo\\music")`。
- **预期行为**：删除库中所有 `C:\Users\foo\music\` 下的记录。
- **实际行为**：`format!("{}/%", dir.to_string_lossy())` 生成 `C:\Users\foo\music/%`，而存储的路径为 `C:\Users\foo\music\song.mp3`（反斜杠分隔），LIKE 不匹配，记录残留。
- **根因分析**：LIKE 模式分隔符与 `Path` 序列化所用的 OS 原生分隔符不一致。
- **修复方案**：使用 `std::path::MAIN_SEPARATOR` 拼接模式，使分隔符与存储路径一致。
- **修复状态**：已修复（见 `core/src/sources/local.rs`）

---

### BUG-011

- **标题**：`LocalSource::start_watch` 在持 `watchers` 锁期间 drop 旧 watcher，可能阻塞
- **严重级别**：Low
- **分类**：竞态 / 性能
- **复现步骤**：
  1. 已对目录 `dir` 启动 watcher。
  2. 再次调用 `add_directory(dir)` 或 `start_watch(dir)`，触发 `watchers.insert(dir, new)`，旧 watcher 被 drop。
- **预期行为**：drop 旧 watcher 时其内部 notify 线程被 join；若该线程正在执行回调并等待 `db` 锁，join 会阻塞当前线程，同时持有 `watchers` 锁阻塞其他 `add_directory` / `remove_directory`。
- **实际行为**：非死锁（`db` 锁最终会释放），但可能造成较长阻塞。
- **根因分析**：watcher drop 与索引锁耦合。
- **修复方案**：将旧 watcher 取出后在锁外 drop（先 `remove`，释放锁，再让旧 watcher 离开作用域）。**待修复**（影响较小）。
- **修复状态**：待修复

---

### BUG-012

- **标题**：`LocalSource::search` 返回的 `total` 是过滤后行数而非音源真实总数
- **严重级别**：Low
- **分类**：边界值
- **复现步骤**：
  1. 调用 `search("hello", 1, 10)`，匹配 5 条。
  2. 查看 `result.total`。
- **预期行为**：`total` 反映音源对该关键字的总匹配数。
- **实际行为**：`total = all.len()`，即加载到内存后的过滤后数量；语义上正确但与「全表 COUNT」存在细微差异（当前实现下二者等价，因 `search_db` 已经过滤）。
- **根因分析**：实现与 `JsonSource` 的 `total` 语义对齐方式不同。
- **修复方案**：保持现状（语义一致），仅在文档中说明 `total` 含义。**不修复**。
- **修复状态**：不修复（语义已一致）

---

## 三、移动端静态审查发现（无法编译，需在对应 IDE 验证）

### iOS（Swift）

- **IOS-001**（Medium，状态不一致）：`PlayerManager.play(song:in:)`（`Player/PlayerManager.swift:72`）当 `song.playUrl` 为 `nil` 或 URL 解析失败时 `guard ... else { return }` 静默返回，但 `currentSong` 已被设置、`isPlaying` 未变更，UI 显示「正在播放」但实际无音频。**需在 Xcode 验证**。
- **IOS-002**（Low，兼容性）：`PlayerManager.toggleMode()` 使用 `PlayMode.allCases`，但 `PlayerManager` 中 `mode` 的类型 `PlayMode` 未见 `CaseIterable` 声明（依赖 `Models/Song.swift` 中的定义）。若 `PlayMode` 枚举未加 `: CaseIterable`，编译失败。**需在 Xcode 验证**。
- **IOS-003**（Low，资源释放）：`deinit` 中 `player.removeTimeObserver(t)` 在 `player` 本身即将释放时调用，顺序依赖 ARC；若 `timeObserver` 闭包捕获 `self` 形成强引用环，`deinit` 不会被触发。当前用 `[weak self]`，应无环。**需在 Xcode 验证**。

### Android（Kotlin）

- **AND-001**（Medium，竞态）：`PlayerManager.startProgressLoop()`（`player/PlayerManager.kt:62`）中 `while (true) { player?.let { ... }; delay(500) }`，`onCleared` 调用 `player?.release()` 将 `player` 置 `null` 与循环中 `player?.let` 之间存在竞争；`viewModelScope` 取消协程可缓解，但 release 与 `currentPosition` 读取之间无同步。**需在 Android Studio 验证**。
- **AND-002**（Low，逻辑）：`PlayerManager.moveToNext(auto=true)` 在 `RANDOM` / `SEQUENTIAL` 分支递归调用 `play(queue[index])`，而 `play` 内部会重新计算 `index = queue.indexOfFirst{...}`，可能导致刚切到的歌曲被重新定位回原 `index`，行为不符合预期。**需在 Android Studio 验证**。
- **AND-003**（Low，资源）：`PlayerManager.attach` 注册的 `Player.Listener` 未在 `onCleared` 显式移除（依赖 `player.release()` 隐式清理）。**需在 Android Studio 验证**。

### HarmonyOS（ArkTS）

- **HAR-001**（Medium，状态不一致）：`PlayerManager.playAt`（`player/PlayerManager.ets:158`）当 `song.playUrl` 为空时仅更新标题不调用 `load`，与 IOS-001 同样问题。**需在 DevEco Studio 验证**。
- **HAR-002**（Medium，未处理异常）：`PlayerManager.ensurePlayer`（`player/PlayerManager.ets:44`）中 `await media.createAVPlayer()` 若失败抛异常，调用方 `load`/`play` 未 `try/catch`，异常会冒泡到 ArkTS 事件循环变为未处理 rejection。**需在 DevEco Studio 验证**。
- **HAR-003**（Medium，兼容性）：`MusicCoreBridge.ets` 的 `SongOrigin` 类型定义中 `Nas` 变体使用 `protocol_name` 字段，而 Rust 端 `models.rs::SongOrigin::Nas` 序列化为 `protocol`（无 `_name` 后缀），跨端反序列化会错位。**需在 DevEco Studio 验证**。
- **HAR-004**（Low，资源）：`PlayerManager` 未提供 `release()` 方法显式释放 `avPlayer`，依赖 GC；AVPlayer 状态机要求显式 `release()` 进入 `released` 态。**需在 DevEco Studio 验证**。

---

## 四、修复与验证汇总

| Bug ID | 严重级别 | 分类 | 修复状态 |
| --- | --- | --- | --- |
| BUG-001 | High | 状态不一致 | 已修复 |
| BUG-002 | High | 未处理异常 | 已修复 |
| BUG-003 | High | 兼容性 / UB | 已修复 |
| BUG-004 | Medium | 未处理异常 | 已修复 |
| BUG-005 | Medium | 竞态 | 已修复（部分缓解） |
| BUG-006 | Medium | 性能 | 已修复 |
| BUG-007 | Medium | 性能 / 内存泄漏 | 已修复 |
| BUG-008 | Medium | 未处理异常 | 已修复 |
| BUG-009 | Low | 性能 | 待修复 |
| BUG-010 | Medium | 兼容性 | 已修复 |
| BUG-011 | Low | 竞态 / 性能 | 待修复 |
| BUG-012 | Low | 边界值 | 不修复（语义一致） |

**严重级别分布**：High 3 / Medium 7 / Low 3（含不修复 1 项）。
**分类分布**：未处理异常 3 / 状态不一致 2 / 竞态 2 / 兼容性 2 / 性能 3 / 边界值 1 / 内存泄漏 1（部分 Bug 跨多分类）。

**已修复 9 项**，**待修复 2 项**（BUG-009 涉及较大重构、BUG-011 影响较小），**不修复 1 项**（BUG-012 语义已一致）。

### 验证结果

修复后执行 `cargo build` 与 `cargo test`：

```
cargo build          → Compiling music-core ... Finished
cargo test           → test result: ok. 81 passed; 0 failed; 0 ignored
```

所有原有 81 个单测保持通过，未删除任何功能或测试。

### 移动端概要

iOS / Android / HarmonyOS 三端因无法在当前环境编译，仅做静态审查，共发现 **10 项** 潜在问题（IOS-001~003、AND-001~003、HAR-001~004），均标注「需在对应 IDE 验证」，不在本次修复范围内强制修复。
