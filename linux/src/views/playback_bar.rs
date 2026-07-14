//! 底部播放控制栏视图。
//!
//! 提供当前歌曲信息展示、播放控制（上一首/播放暂停/下一首/模式切换）、
//! 进度条与音量控制，与 macOS 端 `PlaybackBar.swift` 对齐。
//!
//! 设计要点：
//! - 通过 `glib::timeout_add_local` 每 500ms 轮询 `PlayerService` 状态，更新
//!   歌曲信息、播放按钮图标、进度条位置、时间标签与音量。
//! - 进度条与音量条使用 `Scale`（`change-value` 信号处理用户拖拽，
//!   程序化 `set_value` 不触发该信号，避免反馈循环）。
//! - 播放模式图标随 `PlayMode` 切换：顺序、单曲循环、随机。

use std::sync::Arc;
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::*;
use music_core::models::*;

use crate::player_service::PlayerService;
use crate::theme;

/// 轮询间隔。
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// 播放栏高度（px）。
const BAR_HEIGHT: i32 = 72;

/// 创建底部播放控制栏组件。
///
/// 布局：左侧歌曲信息（封面占位 + 标题 + 艺术家）、
/// 中间播放控制（上一首/播放暂停/下一首/模式 + 进度条 + 时间标签）、
/// 右侧音量控制（图标 + 音量条）。
///
/// # 参数
/// - `player`：共享的播放器服务，用于读取与控制播放状态。
pub fn create_playback_bar(player: Arc<PlayerService>) -> gtk4::Widget {
    let bar = Box::new(Orientation::Horizontal, theme::SPACING_S3);
    bar.add_css_class("ngh-playback-bar");
    bar.set_height_request(BAR_HEIGHT);
    bar.set_margin_start(theme::SPACING_S4);
    bar.set_margin_end(theme::SPACING_S4);
    bar.set_margin_top(theme::SPACING_S2);
    bar.set_margin_bottom(theme::SPACING_S2);

    // ===================================================================
    // 左侧：封面占位 + 歌曲信息
    // ===================================================================
    let left_box = Box::new(Orientation::Horizontal, theme::SPACING_S2);
    left_box.set_halign(Align::Start);

    let cover = Box::new(Orientation::Vertical, 0);
    cover.add_css_class("ngh-cover-placeholder");
    cover.set_size_request(48, 48);
    let cover_icon = Image::from_icon_name("audio-x-generic");
    cover_icon.set_pixel_size(24);
    cover.append(&cover_icon);

    let info = Box::new(Orientation::Vertical, 2);
    info.set_valign(Align::Center);
    let title_label = Label::new(Some("未在播放"));
    title_label.add_css_class("ngh-song-title");
    title_label.set_halign(Align::Start);
    title_label.set_ellipsize(EllipsizeMode::End);
    title_label.set_max_width_chars(25);
    let artist_label = Label::new(Some("—"));
    artist_label.add_css_class("ngh-song-artist");
    artist_label.set_halign(Align::Start);
    artist_label.set_ellipsize(EllipsizeMode::End);
    artist_label.set_max_width_chars(25);
    info.append(&title_label);
    info.append(&artist_label);

    left_box.append(&cover);
    left_box.append(&info);
    bar.append(&left_box);

    // ===================================================================
    // 中间：播放控制 + 进度条
    // ===================================================================
    let center_box = Box::new(Orientation::Vertical, theme::SPACING_S1);
    center_box.set_hexpand(true);
    center_box.set_halign(Align::Center);

    // 按钮行
    let controls = Box::new(Orientation::Horizontal, theme::SPACING_S2);
    controls.set_halign(Align::Center);

    let prev_button = Button::from_icon_name("media-skip-backward-symbolic");
    prev_button.add_css_class("ngh-ghost-button");

    let play_icon = Image::from_icon_name("media-playback-start-symbolic");
    play_icon.set_pixel_size(20);
    let play_button = Button::new();
    play_button.set_child(Some(&play_icon));
    play_button.add_css_class("ngh-primary-button");

    let next_button = Button::from_icon_name("media-skip-forward-symbolic");
    next_button.add_css_class("ngh-ghost-button");

    let mode_icon = Image::from_icon_name("media-playlist-consecutive-symbolic");
    mode_icon.set_pixel_size(18);
    let mode_button = Button::new();
    mode_button.set_child(Some(&mode_icon));
    mode_button.add_css_class("ngh-ghost-button");
    mode_button.set_tooltip_text(Some("切换播放模式"));

    controls.append(&prev_button);
    controls.append(&play_button);
    controls.append(&next_button);
    controls.append(&mode_button);

    // 进度行：当前时间 + 进度条 + 总时长
    let progress_row = Box::new(Orientation::Horizontal, theme::SPACING_S2);

    let current_time_label = Label::new(Some("0:00"));
    current_time_label.add_css_class("ngh-song-duration");
    current_time_label.set_valign(Align::Center);

    let progress_scale = Scale::new_with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    progress_scale.add_css_class("ngh-progress-bar");
    progress_scale.set_hexpand(true);
    progress_scale.set_valign(Align::Center);
    progress_scale.set_draw_value(false);

    let duration_label = Label::new(Some("0:00"));
    duration_label.add_css_class("ngh-song-duration");
    duration_label.set_valign(Align::Center);

    progress_row.append(&current_time_label);
    progress_row.append(&progress_scale);
    progress_row.append(&duration_label);

    center_box.append(&controls);
    center_box.append(&progress_row);
    bar.append(&center_box);

    // ===================================================================
    // 右侧：音量控制
    // ===================================================================
    let right_box = Box::new(Orientation::Horizontal, theme::SPACING_S1);
    right_box.set_halign(Align::End);
    right_box.set_valign(Align::Center);

    let volume_icon = Image::from_icon_name("audio-volume-high-symbolic");
    volume_icon.set_pixel_size(18);

    let volume_scale = Scale::new_with_range(Orientation::Horizontal, 0.0, 1.0, 0.01);
    volume_scale.add_css_class("ngh-volume");
    volume_scale.set_size_request(100, -1);
    volume_scale.set_draw_value(false);
    volume_scale.set_value(f64::from(player.volume()));

    right_box.append(&volume_icon);
    right_box.append(&volume_scale);
    bar.append(&right_box);

    // ===================================================================
    // 信号连接
    // ===================================================================

    // --- 播放/暂停按钮 ---
    {
        let player = Arc::clone(&player);
        play_button.connect_clicked(move |_| {
            player.toggle_play_pause();
        });
    }

    // --- 上一首按钮 ---
    {
        let player = Arc::clone(&player);
        prev_button.connect_clicked(move |_| {
            player.previous();
        });
    }

    // --- 下一首按钮 ---
    {
        let player = Arc::clone(&player);
        next_button.connect_clicked(move |_| {
            player.next();
        });
    }

    // --- 模式切换按钮 ---
    {
        let player = Arc::clone(&player);
        mode_button.connect_clicked(move |_| {
            player.toggle_mode();
        });
    }

    // --- 进度条拖拽：跳转播放进度 ---
    // change-value 仅由用户交互触发，程序化 set_value 不会触发，无反馈循环。
    {
        let player = Arc::clone(&player);
        progress_scale.connect_change_value(move |_, _, value| {
            player.seek(value);
            glib::Propagation::Stop
        });
    }

    // --- 音量条拖拽：设置音量 ---
    {
        let player = Arc::clone(&player);
        volume_scale.connect_change_value(move |_, _, value| {
            player.set_volume(value as f32);
            glib::Propagation::Stop
        });
    }

    // ===================================================================
    // 定时轮询：每 500ms 更新所有状态
    // ===================================================================
    {
        let player = Arc::clone(&player);
        let title_label = title_label.clone();
        let artist_label = artist_label.clone();
        let play_icon = play_icon.clone();
        let mode_icon = mode_icon.clone();
        let current_time_label = current_time_label.clone();
        let duration_label = duration_label.clone();
        let progress_scale = progress_scale.clone();
        let volume_scale = volume_scale.clone();
        let volume_icon = volume_icon.clone();

        glib::timeout_add_local(POLL_INTERVAL, move || {
            // 更新歌曲信息
            match player.current_song() {
                Some(song) => {
                    title_label.set_text(&song.title);
                    artist_label.set_text(&format_artists(&song.artists));
                }
                None => {
                    title_label.set_text("未在播放");
                    artist_label.set_text("—");
                }
            }

            // 更新播放/暂停按钮图标
            let play_icon_name = if player.is_playing() {
                "media-playback-pause-symbolic"
            } else {
                "media-playback-start-symbolic"
            };
            play_icon.set_from_icon_name(Some(play_icon_name));

            // 更新播放模式图标
            let mode_icon_name = mode_icon_name(&player.mode());
            mode_icon.set_from_icon_name(Some(mode_icon_name));

            // 更新进度条
            let duration = player.duration();
            let current = player.current_time();
            current_time_label.set_text(&player.current_time_display());
            duration_label.set_text(&player.duration_display());

            // 设置进度条范围与位置（set_value 不触发 change-value）
            let adj = progress_scale.adjustment();
            adj.set_upper(duration.max(0.1));
            if current <= adj.upper() {
                progress_scale.set_value(current);
            }

            // 更新音量条
            let vol = f64::from(player.volume());
            if (volume_scale.value() - vol).abs() > 0.005 {
                volume_scale.set_value(vol);
            }

            // 更新音量图标
            let vol_icon_name = if vol <= 0.0 {
                "audio-volume-muted-symbolic"
            } else if vol < 0.33 {
                "audio-volume-low-symbolic"
            } else if vol < 0.67 {
                "audio-volume-medium-symbolic"
            } else {
                "audio-volume-high-symbolic"
            };
            volume_icon.set_from_icon_name(Some(vol_icon_name));

            glib::ControlFlow::Continue
        });
    }

    bar.upcast::<Widget>()
}

/// 根据播放模式返回对应的 GTK 图标名。
fn mode_icon_name(mode: &PlayMode) -> &'static str {
    match mode {
        PlayMode::Sequential => "media-playlist-consecutive-symbolic",
        PlayMode::SingleLoop => "media-playlist-repeat-song-symbolic",
        PlayMode::Random => "media-playlist-shuffle-symbolic",
    }
}

/// 格式化艺术家列表为「A / B / C」形式。
fn format_artists(artists: &[String]) -> String {
    artists.join(" / ")
}
