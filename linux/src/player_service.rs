//! 播放器服务：基于 GStreamer 的音频播放封装。
//!
//! 本模块面向 NGHMusic Linux 客户端（GTK4 + Rust），以 GStreamer `playbin`
//! 元素为后端，提供播放/暂停/上一首/下一首/进度跳转/音量控制/播放模式等
//! 能力，并将播放状态持久化到 `~/.local/share/nghmusic/player_state.json`，
//! 重启后可恢复音量与播放模式。
//!
//! ## 设计要点
//! - 持有 GStreamer `playbin` 元素与 `Arc<Mutex<PlayerStateInner>>` 共享状态，
//!   所有公开方法均为 `&self`，通过内部可变性驱动播放。
//! - 通过 `Bus::add_watch_local` 监听总线消息：收到 EOS 时按播放模式自动
//!   推进下一首，收到 Error 时记录日志。
//! - 通过 `glib::source::timeout_add_local` 注册 500ms 周期任务，刷新当前
//!   播放位置与总时长，供进度条实时更新。
//! - `BusWatchGuard` 必须保活，否则总线监听会被立即移除；故存于结构字段。

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dirs::data_dir;
use gstreamer::bus::BusWatchGuard;
use gstreamer::glib::{self, ControlFlow, SourceId};
use gstreamer::prelude::*;
use gstreamer::{ClockTime, Element, ElementFactory, MessageView, SeekFlags, State};
use music_core::{PlayMode, PlayState, Song};

/// 播放器内部可变状态。
///
/// 以 `Arc<Mutex<>>` 包裹存于 [`PlayerService`]，所有读写均经锁保护。
/// `current_index` 为 -1 表示尚未选中任何曲目。
#[derive(Clone, Debug)]
struct PlayerStateInner {
    /// 当前播放曲目
    current_song: Option<Song>,
    /// 是否正在播放
    is_playing: bool,
    /// 当前播放位置（秒）
    current_time: f64,
    /// 总时长（秒）
    duration: f64,
    /// 音量（0.0 ~ 1.0）
    volume: f32,
    /// 播放模式
    mode: PlayMode,
    /// 播放队列
    queue: Vec<Song>,
    /// 当前曲目在队列中的索引，-1 表示未选中
    current_index: i32,
    /// 状态脏标记：seek 时置 true，由周期定时器延迟写盘（防抖），
    /// 避免拖动进度条时高频写盘。
    dirty: bool,
}

impl Default for PlayerStateInner {
    fn default() -> Self {
        Self {
            current_song: None,
            is_playing: false,
            current_time: 0.0,
            duration: 0.0,
            volume: 1.0,
            mode: PlayMode::Sequential,
            queue: Vec::new(),
            current_index: -1,
            dirty: false,
        }
    }
}

/// 播放器服务：GStreamer `playbin` 的线程安全封装。
///
/// 通过 [`PlayerService::new`] 构造，自动初始化 GStreamer、注册总线监听
/// 与位置更新定时器。所有方法均为 `&self`，可在 GTK 主线程直接调用。
pub struct PlayerService {
    /// GStreamer `playbin` 元素
    playbin: Element,
    /// 共享可变状态
    state: Arc<Mutex<PlayerStateInner>>,
    /// 状态持久化文件路径
    state_file: PathBuf,
    /// 总线监听保活句柄（drop 即移除监听）
    _bus_watch_guard: Option<BusWatchGuard>,
    /// 位置更新定时器 SourceId（保活）
    _position_source_id: Option<SourceId>,
}

impl PlayerService {
    /// 创建播放器服务：初始化 GStreamer、构造 playbin、注册总线监听与
    /// 500ms 位置更新定时器，并从磁盘恢复音量与播放模式。
    pub fn new() -> Self {
        let _ = gstreamer::init();
        let playbin = ElementFactory::make("playbin")
            .build()
            .expect("创建 GStreamer playbin 元素失败");
        let state = Arc::new(Mutex::new(PlayerStateInner::default()));
        let state_file = data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("nghmusic")
            .join("player_state.json");

        // 从磁盘恢复音量与模式（队列需由上层重新装载）。
        // 注意：不恢复 current_index，因为此时队列为空，恢复的索引无意义，
        // 且会导致后续 load_queue 之前的 current_index() 报告陈旧值。
        if let Some(persisted) = Self::load_state_from_file(&state_file) {
            if let Ok(mut s) = state.lock() {
                s.volume = persisted.volume;
                s.mode = persisted.mode.clone();
            }
            playbin.set_property("volume", f64::from(persisted.volume));
        }

        // 总线监听：处理 EOS 自动下一首与错误日志
        let state_for_bus = Arc::clone(&state);
        let playbin_for_bus = playbin.clone();
        let state_file_for_bus = state_file.clone();
        let bus = playbin.bus().expect("获取 playbin 总线失败");
        let bus_watch_guard = bus
            .add_watch_local(move |_bus, msg| {
                match msg.view() {
                    MessageView::Eos(_) => {
                        Self::advance_on_eos(&playbin_for_bus, &state_for_bus, &state_file_for_bus);
                    }
                    MessageView::Error(err) => {
                        log::error!("GStreamer 播放错误: {}", err.error());
                    }
                    _ => {}
                }
                ControlFlow::Continue
            })
            .expect("注册 playbin 总线监听失败");

        // 500ms 周期：刷新当前播放位置与总时长，并在脏标记置位时延迟写盘（防抖）
        let state_for_tick = Arc::clone(&state);
        let playbin_for_tick = playbin.clone();
        let state_file_for_tick = state_file.clone();
        let position_source_id = glib::source::timeout_add_local(
            Duration::from_millis(500),
            move || {
                let mut dirty = false;
                if let Ok(mut s) = state_for_tick.lock() {
                    if s.is_playing {
                        if let Some(pos) = playbin_for_tick.query_position::<ClockTime>() {
                            s.current_time = pos.seconds_f64();
                        }
                        if let Some(dur) = playbin_for_tick.query_duration::<ClockTime>() {
                            s.duration = dur.seconds_f64();
                        }
                    }
                    if s.dirty {
                        s.dirty = false;
                        dirty = true;
                    }
                }
                // 锁已释放后再写盘，避免持锁 I/O
                if dirty {
                    Self::save_state(&state_for_tick, &state_file_for_tick);
                }
                ControlFlow::Continue
            },
        );

        Self {
            playbin,
            state,
            state_file,
            _bus_watch_guard: Some(bus_watch_guard),
            _position_source_id: Some(position_source_id),
        }
    }

    // =================================================================
    // 队列与播放控制
    // =================================================================

    /// 装载播放队列并从 `start_index` 开始播放。
    ///
    /// 队列为空时停止播放并复位状态。索引越界或曲目无可播放 URL 时
    /// 记录日志并跳过（`load_and_play` 内部处理）。
    pub fn load_queue(&self, songs: Vec<Song>, start_index: usize) {
        if songs.is_empty() {
            // 空队列：停止 playbin 并复位状态
            Self::set_playbin_state(&self.playbin, State::Null, "load_queue 空队列");
            if let Ok(mut s) = self.state.lock() {
                s.current_index = -1;
                s.current_song = None;
                s.current_time = 0.0;
                s.duration = 0.0;
                s.is_playing = false;
                s.queue = songs;
            }
            Self::save_state(&self.state, &self.state_file);
            return;
        }
        // 非空队列：先写入队列，再从指定索引播放
        {
            if let Ok(mut s) = self.state.lock() {
                s.queue = songs;
            }
        }
        Self::load_and_play(&self.playbin, &self.state, &self.state_file, start_index);
    }

    /// 播放队列中指定索引的曲目。
    ///
    /// 索引越界或曲目无可播放 URL 时记录日志并跳过。
    pub fn play_at(&self, index: usize) {
        Self::load_and_play(&self.playbin, &self.state, &self.state_file, index);
    }

    /// 切换播放/暂停状态。
    pub fn toggle_play_pause(&self) {
        if self.is_playing() {
            self.pause();
        } else {
            self.resume();
        }
    }

    /// 暂停播放。
    pub fn pause(&self) {
        Self::set_playbin_state(&self.playbin, State::Paused, "pause");
        if let Ok(mut s) = self.state.lock() {
            s.is_playing = false;
        }
        Self::save_state(&self.state, &self.state_file);
    }

    /// 继续播放（从暂停恢复）。
    pub fn resume(&self) {
        Self::set_playbin_state(&self.playbin, State::Playing, "resume");
        if let Ok(mut s) = self.state.lock() {
            s.is_playing = true;
        }
        Self::save_state(&self.state, &self.state_file);
    }

    /// 停止播放并复位位置。
    pub fn stop(&self) {
        Self::set_playbin_state(&self.playbin, State::Null, "stop");
        if let Ok(mut s) = self.state.lock() {
            s.is_playing = false;
            s.current_time = 0.0;
        }
        Self::save_state(&self.state, &self.state_file);
    }

    /// 下一首：按播放模式计算目标索引并播放。
    pub fn next(&self) {
        let (len, cur, mode) = match self.state.lock() {
            Ok(s) => (s.queue.len(), s.current_index, s.mode.clone()),
            Err(_) => return,
        };
        if let Some(idx) = Self::next_index(cur, len, &mode, false) {
            Self::load_and_play(&self.playbin, &self.state, &self.state_file, idx);
        }
    }

    /// 上一首：按播放模式计算目标索引并播放。
    pub fn previous(&self) {
        let (len, cur, mode) = match self.state.lock() {
            Ok(s) => (s.queue.len(), s.current_index, s.mode.clone()),
            Err(_) => return,
        };
        if let Some(idx) = Self::prev_index(cur, len, &mode) {
            Self::load_and_play(&self.playbin, &self.state, &self.state_file, idx);
        }
    }

    /// 跳转到指定秒数位置。
    ///
    /// 不立即写盘：仅标记脏，由 500ms 周期定时器延迟持久化（防抖），
    /// 避免拖动进度条时高频磁盘 I/O。
    pub fn seek(&self, position_seconds: f64) {
        let pos = position_seconds.max(0.0);
        if let Err(e) = self
            .playbin
            .seek_simple(SeekFlags::FLUSH, ClockTime::from_seconds_f64(pos))
        {
            log::warn!("seek 失败: {e}");
        }
        if let Ok(mut s) = self.state.lock() {
            s.current_time = pos;
            s.dirty = true;
        }
    }

    /// 按进度比例（0.0 ~ 1.0）跳转。
    pub fn seek_to_fraction(&self, fraction: f32) {
        let frac = fraction.clamp(0.0, 1.0) as f64;
        let dur = self.duration();
        if dur > 0.0 {
            self.seek(frac * dur);
        }
    }

    // =================================================================
    // 播放模式与音量
    // =================================================================

    /// 循环切换播放模式：顺序 → 单曲循环 → 随机 → 顺序。
    pub fn toggle_mode(&self) {
        let new_mode = match self.mode() {
            PlayMode::Sequential => PlayMode::SingleLoop,
            PlayMode::SingleLoop => PlayMode::Random,
            PlayMode::Random => PlayMode::Sequential,
        };
        self.set_mode(new_mode);
    }

    /// 设置播放模式并持久化。
    pub fn set_mode(&self, mode: PlayMode) {
        if let Ok(mut s) = self.state.lock() {
            s.mode = mode;
        }
        Self::save_state(&self.state, &self.state_file);
    }

    /// 设置音量（0.0 ~ 1.0，自动钳制），同步到 playbin 并持久化。
    pub fn set_volume(&self, volume: f32) {
        let v = volume.clamp(0.0, 1.0);
        self.playbin.set_property("volume", f64::from(v));
        if let Ok(mut s) = self.state.lock() {
            s.volume = v;
        }
        Self::save_state(&self.state, &self.state_file);
    }

    // =================================================================
    // 状态查询
    // =================================================================

    /// 当前播放曲目。
    pub fn current_song(&self) -> Option<Song> {
        self.state.lock().ok().and_then(|s| s.current_song.clone())
    }

    /// 是否正在播放。
    pub fn is_playing(&self) -> bool {
        self.state.lock().map(|s| s.is_playing).unwrap_or(false)
    }

    /// 当前播放位置（秒）。
    pub fn current_time(&self) -> f64 {
        self.state.lock().map(|s| s.current_time).unwrap_or(0.0)
    }

    /// 总时长（秒）。
    pub fn duration(&self) -> f64 {
        self.state.lock().map(|s| s.duration).unwrap_or(0.0)
    }

    /// 音量（0.0 ~ 1.0）。
    pub fn volume(&self) -> f32 {
        self.state.lock().map(|s| s.volume).unwrap_or(1.0)
    }

    /// 播放模式。
    pub fn mode(&self) -> PlayMode {
        self.state
            .lock()
            .map(|s| s.mode.clone())
            .unwrap_or(PlayMode::Sequential)
    }

    /// 播放队列副本。
    pub fn queue(&self) -> Vec<Song> {
        self.state.lock().map(|s| s.queue.clone()).unwrap_or_default()
    }

    /// 当前曲目索引（-1 表示未选中）。
    pub fn current_index(&self) -> i32 {
        self.state.lock().map(|s| s.current_index).unwrap_or(-1)
    }

    /// 播放进度比例（0.0 ~ 1.0）。
    pub fn progress(&self) -> f32 {
        let (cur, dur) = self
            .state
            .lock()
            .map(|s| (s.current_time, s.duration))
            .unwrap_or((0.0, 0.0));
        if dur > 0.0 {
            (cur / dur) as f32
        } else {
            0.0
        }
    }

    /// 当前播放位置的可读时间（`M:SS`）。
    pub fn current_time_display(&self) -> String {
        Self::format_time(self.current_time())
    }

    /// 总时长的可读时间（`M:SS`）。
    pub fn duration_display(&self) -> String {
        Self::format_time(self.duration())
    }

    // =================================================================
    // 内部辅助
    // =================================================================

    /// 设置 playbin 状态，失败时记录日志而非静默忽略。
    fn set_playbin_state(playbin: &Element, state: State, context: &str) {
        if let Err(e) = playbin.set_state(state) {
            log::warn!("设置 playbin 状态失败（{context}）: {e}");
        }
    }

    /// 装载指定索引曲目并开始播放。
    ///
    /// URL 解析规则：优先使用 `local_path` 转为 `file://` URI，
    /// 否则回退到 `play_url`。无可用 URL 时记录日志并跳过。
    fn load_and_play(
        playbin: &Element,
        state: &Arc<Mutex<PlayerStateInner>>,
        state_file: &Path,
        index: usize,
    ) {
        let (song, volume) = match state.lock() {
            Ok(s) if index < s.queue.len() => (s.queue[index].clone(), s.volume),
            _ => {
                log::warn!("play_at 索引越界或队列未就绪: {index}");
                return;
            }
        };

        let uri = song
            .local_path
            .as_ref()
            .map(|p| format!("file://{}", p.to_string_lossy()))
            .or_else(|| song.play_url.clone());
        let Some(uri) = uri else {
            log::warn!("曲目无可播放 URL，已跳过: {}", song.title);
            return;
        };

        let duration = song.duration_ms.map(|ms| ms as f64 / 1000.0).unwrap_or(0.0);

        Self::set_playbin_state(playbin, State::Null, "load_and_play 复位");
        playbin.set_property("uri", uri);
        playbin.set_property("volume", f64::from(volume));

        if let Ok(mut s) = state.lock() {
            s.current_index = index as i32;
            s.current_song = Some(song);
            s.is_playing = true;
            s.current_time = 0.0;
            s.duration = duration;
        }
        Self::set_playbin_state(playbin, State::Playing, "load_and_play 播放");
        Self::save_state(state, state_file);
    }

    /// EOS 触发的自动推进：按播放模式计算下一曲。
    ///
    /// 顺序模式到达队尾时停止播放；单曲循环重播当前曲；随机模式随机选曲。
    fn advance_on_eos(
        playbin: &Element,
        state: &Arc<Mutex<PlayerStateInner>>,
        state_file: &Path,
    ) {
        let (len, cur, mode) = match state.lock() {
            Ok(s) => (s.queue.len(), s.current_index, s.mode.clone()),
            Err(_) => return,
        };
        match Self::next_index(cur, len, &mode, true) {
            Some(idx) => Self::load_and_play(playbin, state, state_file, idx),
            None => {
                // 顺序模式播放完毕：停止并复位
                Self::set_playbin_state(playbin, State::Null, "advance_on_eos 停止");
                if let Ok(mut s) = state.lock() {
                    s.is_playing = false;
                    s.current_time = 0.0;
                }
                Self::save_state(state, state_file);
            }
        }
    }

    /// 计算下一曲索引。
    ///
    /// `from_eos=true` 表示由播放结束触发：顺序模式到队尾返回 `None`（停止），
    /// 单曲循环返回当前索引；`from_eos=false`（手动下一首）顺序模式到队尾
    /// 回绕到 0。
    fn next_index(current: i32, len: usize, mode: &PlayMode, from_eos: bool) -> Option<usize> {
        if len == 0 {
            return None;
        }
        let cur = (current.max(0) as usize).min(len - 1);
        match mode {
            PlayMode::Random => Some(Self::random_index(len, cur as i32)),
            PlayMode::SingleLoop if from_eos => Some(cur),
            _ => {
                if cur + 1 < len {
                    Some(cur + 1)
                } else if from_eos {
                    None
                } else {
                    Some(0)
                }
            }
        }
    }

    /// 计算上一曲索引。随机模式随机选曲；其余模式按序回绕。
    fn prev_index(current: i32, len: usize, mode: &PlayMode) -> Option<usize> {
        if len == 0 {
            return None;
        }
        let cur = (current.max(0) as usize).min(len - 1);
        match mode {
            PlayMode::Random => Some(Self::random_index(len, cur as i32)),
            _ => {
                if cur == 0 {
                    Some(len - 1)
                } else {
                    Some(cur - 1)
                }
            }
        }
    }

    /// 使用 `rand` crate 生成均匀分布的随机索引，尽量避开 `exclude`。
    fn random_index(len: usize, exclude: i32) -> usize {
        if len <= 1 {
            return 0;
        }
        let mut rng = rand::thread_rng();
        let mut idx = rand::Rng::gen_range(&mut rng, 0..len);
        if idx == exclude as usize && exclude >= 0 {
            idx = (idx + 1) % len;
        }
        idx
    }

    /// 将当前状态快照序列化写入磁盘。
    ///
    /// 注意：仅持久化音量、播放模式、当前曲目 id/索引/位置等元信息，
    /// 不持久化完整队列（队列由上层 UI 重新装载）。重启后仅恢复音量与模式，
    /// 不恢复播放位置与队列（完整状态恢复未实现）。
    fn save_state(state: &Arc<Mutex<PlayerStateInner>>, path: &Path) {
        let snap = match state.lock() {
            Ok(s) => PlayState {
                current_song_id: s.current_song.as_ref().map(|song| song.id.clone()),
                playlist_id: None,
                index: if s.current_index >= 0 {
                    Some(s.current_index as usize)
                } else {
                    None
                },
                position_ms: (s.current_time * 1000.0) as u64,
                duration_ms: (s.duration * 1000.0) as u64,
                is_playing: s.is_playing,
                volume: s.volume,
                mode: s.mode.clone(),
            },
            Err(_) => return,
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&snap) {
            let _ = std::fs::write(path, json);
        }
    }

    /// 从磁盘读取并反序列化播放状态。
    fn load_state_from_file(path: &Path) -> Option<PlayState> {
        let data = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&data).ok()
    }

    /// 将秒数格式化为 `M:SS`。
    fn format_time(seconds: f64) -> String {
        let total = if seconds.is_finite() && seconds > 0.0 {
            seconds as u64
        } else {
            0
        };
        let m = total / 60;
        let s = total % 60;
        format!("{m}:{s:02}")
    }
}

impl Default for PlayerService {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for PlayerService {
    fn drop(&mut self) {
        // 移除位置更新定时器，避免 drop 后定时器仍触发并访问已释放的 playbin。
        if let Some(id) = self._position_source_id.take() {
            id.remove();
        }
        // 释放 GStreamer 资源：将 playbin 置为 NULL 状态，
        // 停止总线监听（BusWatchGuard drop 时自动移除）。
        Self::set_playbin_state(&self.playbin, State::Null, "Drop");
    }
}
