//! 收藏页视图。
//!
//! 展示收藏分组与已收藏歌曲，与 macOS 端 `FavoritesView.swift` 对齐。
//! 简化实现：分组与歌曲存储在本地 JSON 文件（`~/.local/share/nghmusic/favorites.json`）。
//!
//! 设计要点：
//! - 支持多个收藏分组，默认包含「我喜欢的音乐」。
//! - 可添加当前播放歌曲到选中分组、新建分组。
//! - 歌曲行采用线性列表风格，与搜索/播放列表一致。

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use gtk4::prelude::*;
use serde::{Deserialize, Serialize};

use crate::player_service::PlayerService;
use crate::theme;
use music_core::models::*;

/// 收藏分组（本地持久化结构）。
#[derive(Serialize, Deserialize, Clone)]
struct FavoriteGroup {
    /// 分组唯一标识。
    id: String,
    /// 分组显示名称。
    name: String,
    /// 已收藏的歌曲列表。
    songs: Vec<Song>,
}

/// 收藏页内部状态。
struct FavoritesState {
    /// 全部分组。
    groups: Vec<FavoriteGroup>,
    /// 当前选中分组的索引。
    selected: usize,
}

/// 创建收藏页组件。
///
/// 布局自上而下：页面标题、操作栏（添加当前歌曲 + 新建分组）、
/// 分组列表、分割线、歌曲列表（或空状态占位）。
///
/// # 参数
/// - `player`：共享的播放器服务，用于读取当前播放歌曲与播放收藏歌曲。
pub fn create_favorites_page(player: Arc<PlayerService>) -> gtk4::Widget {
    let state = Rc::new(RefCell::new(FavoritesState {
        groups: load_favorites(),
        selected: 0,
    }));

    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // --- 页面标题 ---
    let title = gtk4::Label::new(Some("收藏"));
    title.add_css_class("ngh-page-title");
    title.set_halign(gtk4::Align::Start);
    title.set_margin_start(theme::SPACING_S4);
    title.set_margin_top(theme::SPACING_S4);
    title.set_margin_bottom(theme::SPACING_S2);
    container.append(&title);

    // --- 操作栏 ---
    let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S2);
    toolbar.set_margin_start(theme::SPACING_S4);
    toolbar.set_margin_end(theme::SPACING_S4);
    toolbar.set_margin_bottom(theme::SPACING_S3);

    let add_button = gtk4::Button::with_label("添加当前歌曲");
    add_button.add_css_class("ngh-primary-button");

    let new_group_button = gtk4::Button::with_label("新建分组");
    new_group_button.add_css_class("ngh-ghost-button");

    let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    toolbar.append(&add_button);
    toolbar.append(&new_group_button);
    toolbar.append(&spacer);
    container.append(&toolbar);

    // --- 分组区标题 ---
    let group_header = gtk4::Label::new(Some("分组"));
    group_header.add_css_class("ngh-label-secondary");
    group_header.set_halign(gtk4::Align::Start);
    group_header.set_margin_start(theme::SPACING_S4);
    group_header.set_margin_bottom(theme::SPACING_S1);
    container.append(&group_header);

    // --- 分组列表 ---
    let group_list = gtk4::ListBox::new();
    group_list.add_css_class("ngh-list");
    group_list.set_selection_mode(gtk4::SelectionMode::Single);
    group_list.set_margin_start(theme::SPACING_S4);
    group_list.set_margin_end(theme::SPACING_S4);
    group_list.set_margin_bottom(theme::SPACING_S3);
    container.append(&group_list);

    // --- 分割线 ---
    let divider = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    divider.set_margin_start(theme::SPACING_S4);
    divider.set_margin_end(theme::SPACING_S4);
    divider.set_margin_bottom(theme::SPACING_S3);
    container.append(&divider);

    // --- 歌曲区标题 ---
    let song_header = gtk4::Label::new(Some("歌曲"));
    song_header.add_css_class("ngh-label-secondary");
    song_header.set_halign(gtk4::Align::Start);
    song_header.set_margin_start(theme::SPACING_S4);
    song_header.set_margin_bottom(theme::SPACING_S1);
    container.append(&song_header);

    // --- 歌曲列表 / 空状态 ---
    let placeholder = gtk4::Label::new(Some("尚无收藏歌曲，点击「添加当前歌曲」收藏正在播放的曲目"));
    placeholder.add_css_class("ngh-empty-state");
    placeholder.set_vexpand(true);
    placeholder.set_valign(gtk4::Align::Center);

    let song_list = gtk4::ListBox::new();
    song_list.add_css_class("ngh-list");
    song_list.set_selection_mode(gtk4::SelectionMode::None);

    let scrolled = gtk4::ScrolledWindow::new(None::<&gtk4::Adjustment>, None::<&gtk4::Adjustment>);
    scrolled.set_vexpand(true);
    scrolled.set_child(Some(&song_list));
    scrolled.set_visible(false);

    container.append(&placeholder);
    container.append(&scrolled);

    // --- 首次构建 ---
    refresh_groups(&state, &group_list);
    refresh_songs(&state, &song_list, &placeholder, &scrolled, &player);

    // --- 分组选中变化 ---
    {
        let state = Rc::clone(&state);
        let song_list = song_list.clone();
        let placeholder = placeholder.clone();
        let scrolled = scrolled.clone();
        let player = Arc::clone(&player);
        group_list.connect_selected_rows_changed(move |listbox| {
            if let Some(row) = listbox.selected_row() {
                let idx = row.index();
                if idx >= 0 {
                    state.borrow_mut().selected = idx as usize;
                    refresh_songs(&state, &song_list, &placeholder, &scrolled, &player);
                }
            }
        });
    }

    // --- 添加当前歌曲 ---
    {
        let state = Rc::clone(&state);
        let group_list = group_list.clone();
        let song_list = song_list.clone();
        let placeholder = placeholder.clone();
        let scrolled = scrolled.clone();
        let player = Arc::clone(&player);
        add_button.connect_clicked(move |_| {
            let song = match player.current_song() {
                Some(s) => s,
                None => return,
            };
            {
                let mut s = state.borrow_mut();
                if let Some(group) = s.groups.get_mut(s.selected) {
                    // 避免重复收藏
                    let exists = group
                        .songs
                        .iter()
                        .any(|x| x.id == song.id && x.source_id == song.source_id);
                    if !exists {
                        group.songs.push(song);
                    }
                }
            }
            save_favorites(&state.borrow().groups);
            refresh_groups(&state, &group_list);
            refresh_songs(&state, &song_list, &placeholder, &scrolled, &player);
        });
    }

    // --- 新建分组 ---
    {
        let state = Rc::clone(&state);
        let group_list = group_list.clone();
        let song_list = song_list.clone();
        let placeholder = placeholder.clone();
        let scrolled = scrolled.clone();
        let player = Arc::clone(&player);
        new_group_button.connect_clicked(move |_| {
            {
                let mut s = state.borrow_mut();
                let n = s.groups.len() + 1;
                s.groups.push(FavoriteGroup {
                    id: format!("group-{n}"),
                    name: format!("分组 {n}"),
                    songs: Vec::new(),
                });
                s.selected = s.groups.len() - 1;
            }
            save_favorites(&state.borrow().groups);
            refresh_groups(&state, &group_list);
            refresh_songs(&state, &song_list, &placeholder, &scrolled, &player);
        });
    }

    // --- 歌曲行激活播放 ---
    {
        let state = Rc::clone(&state);
        let player = Arc::clone(&player);
        song_list.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx >= 0 {
                let idx = idx as usize;
                let s = state.borrow();
                if let Some(group) = s.groups.get(s.selected) {
                    if idx < group.songs.len() {
                        let songs = group.songs.clone();
                        player.load_queue(songs, idx);
                    }
                }
            }
        });
    }

    container.upcast::<gtk4::Widget>()
}

/// 刷新分组列表：清空并按 `state.groups` 重建，同步选中态。
fn refresh_groups(state: &Rc<RefCell<FavoritesState>>, group_list: &gtk4::ListBox) {
    while let Some(child) = group_list.first_child() {
        group_list.remove(&child);
    }
    let s = state.borrow();
    for (idx, group) in s.groups.iter().enumerate() {
        let row = create_group_row(&group.name, group.songs.len());
        group_list.append(&row);
        if idx == s.selected {
            if let Some(r) = group_list.row_at_index(idx as i32) {
                group_list.select_row(Some(&r));
            }
        }
    }
}

/// 刷新歌曲列表：按选中分组的歌曲重建，同步空状态可见性与当前播放高亮。
fn refresh_songs(
    state: &Rc<RefCell<FavoritesState>>,
    song_list: &gtk4::ListBox,
    placeholder: &gtk4::Label,
    scrolled: &gtk4::ScrolledWindow,
    player: &Arc<PlayerService>,
) {
    while let Some(child) = song_list.first_child() {
        song_list.remove(&child);
    }
    let s = state.borrow();
    let songs: Vec<Song> = match s.groups.get(s.selected) {
        Some(g) => g.songs.clone(),
        None => Vec::new(),
    };

    if songs.is_empty() {
        placeholder.set_visible(true);
        scrolled.set_visible(false);
        return;
    }

    placeholder.set_visible(false);
    scrolled.set_visible(true);

    let current = player.current_song();
    let current_key = current.as_ref().map(|c| (c.id.clone(), c.source_id.clone()));

    for (index, song) in songs.iter().enumerate() {
        let is_playing = current_key
            .as_ref()
            .map(|(id, sid)| id == &song.id && sid == &song.source_id)
            .unwrap_or(false);
        let row = create_favorite_song_row(song, index, is_playing);
        song_list.append(&row);
    }
}

/// 创建分组行（图标 + 名称 + 歌曲数）。
fn create_group_row(name: &str, count: usize) -> gtk4::ListBoxRow {
    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S3);
    row_box.add_css_class("ngh-sidebar-item");

    let icon = gtk4::Image::from_icon_name("emblem-favorite");
    icon.set_pixel_size(18);

    let name_label = gtk4::Label::new(Some(name));
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_hexpand(true);

    let count_label = gtk4::Label::new(Some(&format!("{count}")));
    count_label.add_css_class("ngh-label-secondary");

    row_box.append(&icon);
    row_box.append(&name_label);
    row_box.append(&count_label);

    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&row_box));
    row.set_focusable(false);
    row.set_activatable(true);
    row
}

/// 创建收藏歌曲行（序号 + 封面占位 + 标题/艺术家 + 时长）。
fn create_favorite_song_row(song: &Song, index: usize, is_playing: bool) -> gtk4::ListBoxRow {
    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S3);
    row_box.add_css_class("ngh-song-row");
    if is_playing {
        row_box.add_css_class("playing");
    }

    // 序号
    let index_label = gtk4::Label::new(Some(&format!("{}", index + 1)));
    index_label.add_css_class("ngh-song-index");
    index_label.set_valign(gtk4::Align::Center);

    // 封面占位
    let cover = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    cover.add_css_class("ngh-cover-placeholder");
    cover.set_size_request(32, 32);
    let cover_icon = gtk4::Image::from_icon_name("audio-x-generic");
    cover_icon.set_pixel_size(16);
    cover.append(&cover_icon);

    // 标题 + 艺术家
    let info = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    info.set_hexpand(true);
    info.set_halign(gtk4::Align::Start);
    let title_label = gtk4::Label::new(Some(&song.title));
    title_label.add_css_class("ngh-song-title");
    title_label.set_halign(gtk4::Align::Start);
    title_label.set_ellipsize(gtk4::EllipsizeMode::End);
    let artist_label = gtk4::Label::new(Some(&format_artists(&song.artists)));
    artist_label.add_css_class("ngh-song-artist");
    artist_label.set_halign(gtk4::Align::Start);
    artist_label.set_ellipsize(gtk4::EllipsizeMode::End);
    info.append(&title_label);
    info.append(&artist_label);

    // 时长
    let duration_label = gtk4::Label::new(Some(&format_duration(song.duration_ms)));
    duration_label.add_css_class("ngh-song-duration");
    duration_label.set_valign(gtk4::Align::Center);

    row_box.append(&index_label);
    row_box.append(&cover);
    row_box.append(&info);
    row_box.append(&duration_label);

    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&row_box));
    row.set_focusable(false);
    row.set_activatable(true);
    row
}

// ===========================================================================
// 本地持久化
// ===========================================================================

/// 收藏数据文件路径：`~/.local/share/nghmusic/favorites.json`。
fn favorites_file_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join("nghmusic").join("favorites.json")
}

/// 确保数据目录存在。
fn ensure_data_dir() {
    if let Some(base) = dirs::data_dir() {
        let _ = std::fs::create_dir_all(base.join("nghmusic"));
    }
}

/// 从本地文件加载收藏分组；文件不存在或解析失败时返回默认分组。
fn load_favorites() -> Vec<FavoriteGroup> {
    let path = favorites_file_path();
    match std::fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| default_groups()),
        Err(_) => default_groups(),
    }
}

/// 将收藏分组持久化到本地文件。
fn save_favorites(groups: &[FavoriteGroup]) {
    ensure_data_dir();
    let path = favorites_file_path();
    if let Ok(json) = serde_json::to_string_pretty(groups) {
        let _ = std::fs::write(&path, json);
    }
}

/// 默认收藏分组：包含一个空的「我喜欢的音乐」分组。
fn default_groups() -> Vec<FavoriteGroup> {
    vec![FavoriteGroup {
        id: "default".to_string(),
        name: "我喜欢的音乐".to_string(),
        songs: Vec::new(),
    }]
}

/// 格式化艺术家列表为「A / B / C」形式。
fn format_artists(artists: &[String]) -> String {
    artists.join(" / ")
}

/// 将毫秒时长格式化为 `mm:ss`。
fn format_duration(duration_ms: Option<u64>) -> String {
    match duration_ms {
        Some(ms) => {
            let total_secs = ms / 1000;
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            format!("{mins:02}:{secs:02}")
        }
        None => "--:--".to_string(),
    }
}
