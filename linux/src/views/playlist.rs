//! 播放列表页视图。
//!
//! 展示当前播放队列，与 macOS 端 `PlaylistView.swift` 对齐：点击歌曲行播放、
//! 清空队列按钮、当前播放项高亮。
//!
//! 设计要点：
//! - 队列数据来自 `PlayerService::queue()`，当前索引来自 `PlayerService::current_index()`。
//! - 由于 GTK4 无响应式绑定，提供「刷新」按钮手动重建列表。
//! - 歌曲行采用线性列表风格，当前播放项使用 `play` 图标 + `.playing` 高亮。

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk4::prelude::*;

use crate::player_service::PlayerService;
use crate::theme;
use music_core::models::*;

/// 创建播放列表页组件。
///
/// 布局自上而下：页面标题、操作栏（队列数量 + 刷新按钮 + 清空按钮）、
/// 播放队列列表（`ScrolledWindow` 内嵌 `ListBox`）或空状态占位。
///
/// # 参数
/// - `player`：共享的播放器服务，用于读取队列、播放与清空。
pub fn create_playlist_page(player: Arc<PlayerService>) -> gtk4::Widget {
    // 缓存队列快照，供行激活回调使用
    let queue_cache: Rc<RefCell<Vec<Song>>> = Rc::new(RefCell::new(Vec::new()));

    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // --- 页面标题 ---
    let title = gtk4::Label::new(Some("播放列表"));
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

    let count_label = gtk4::Label::new(Some("当前队列（0 首）"));
    count_label.add_css_class("ngh-label-primary");
    count_label.set_halign(gtk4::Align::Start);
    count_label.set_hexpand(true);

    let refresh_button = gtk4::Button::with_icon_name("view-refresh");
    refresh_button.add_css_class("ngh-ghost-button");
    refresh_button.set_tooltip_text(Some("刷新队列"));

    let clear_button = gtk4::Button::with_label("清空");
    clear_button.add_css_class("ngh-ghost-button");
    clear_button.set_tooltip_text(Some("清空播放队列"));

    toolbar.append(&count_label);
    toolbar.append(&refresh_button);
    toolbar.append(&clear_button);
    container.append(&toolbar);

    // --- 空状态 / 列表 ---
    let placeholder = gtk4::Label::new(Some("播放队列为空，从搜索或排行榜中开始播放歌曲"));
    placeholder.add_css_class("ngh-empty-state");
    placeholder.set_vexpand(true);
    placeholder.set_valign(gtk4::Align::Center);

    let listbox = gtk4::ListBox::new();
    listbox.add_css_class("ngh-list");
    listbox.set_selection_mode(gtk4::SelectionMode::None);

    let scrolled = gtk4::ScrolledWindow::new(None::<&gtk4::Adjustment>, None::<&gtk4::Adjustment>);
    scrolled.set_vexpand(true);
    scrolled.set_child(Some(&listbox));
    scrolled.set_visible(false);

    container.append(&placeholder);
    container.append(&scrolled);

    // --- 重建队列列表 ---
    let rebuild = build_rebuild_closure(
        &player,
        &queue_cache,
        &count_label,
        &placeholder,
        &scrolled,
        &listbox,
    );

    // 首次构建
    rebuild();

    // --- 刷新按钮 ---
    {
        let rebuild = build_rebuild_closure(
            &player,
            &queue_cache,
            &count_label,
            &placeholder,
            &scrolled,
            &listbox,
        );
        refresh_button.connect_clicked(move |_| {
            rebuild();
        });
    }

    // --- 清空按钮 ---
    {
        let player = Arc::clone(&player);
        let queue_cache = Rc::clone(&queue_cache);
        let count_label = count_label.clone();
        let placeholder = placeholder.clone();
        let scrolled = scrolled.clone();
        let listbox = listbox.clone();
        clear_button.connect_clicked(move |_| {
            player.load_queue(Vec::new(), 0);
            // 立即更新 UI
            count_label.set_text("当前队列（0 首）");
            placeholder.set_visible(true);
            scrolled.set_visible(false);
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
            queue_cache.borrow_mut().clear();
        });
    }

    // --- 行激活播放 ---
    {
        let player = Arc::clone(&player);
        let queue_cache = Rc::clone(&queue_cache);
        listbox.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx >= 0 {
                let idx = idx as usize;
                let queue = queue_cache.borrow();
                if idx < queue.len() {
                    player.play_at(idx);
                }
            }
        });
    }

    container.upcast::<gtk4::Widget>()
}

/// 构造一个重建队列列表的闭包，捕获所需的克隆引用。
///
/// 读取 `player.queue()` 与 `player.current_index()`，清空并重建 `listbox`，
/// 同步更新计数标签、空状态可见性与队列缓存。
fn build_rebuild_closure(
    player: &Arc<PlayerService>,
    queue_cache: &Rc<RefCell<Vec<Song>>>,
    count_label: &gtk4::Label,
    placeholder: &gtk4::Label,
    scrolled: &gtk4::ScrolledWindow,
    listbox: &gtk4::ListBox,
) -> impl Fn() + 'static {
    let player = Arc::clone(player);
    let queue_cache = Rc::clone(queue_cache);
    let count_label = count_label.clone();
    let placeholder = placeholder.clone();
    let scrolled = scrolled.clone();
    let listbox = listbox.clone();
    move || {
        let queue = player.queue();
        let current_idx = player.current_index();
        let count = queue.len();

        count_label.set_text(&format!("当前队列（{count} 首）"));

        if count == 0 {
            placeholder.set_visible(true);
            scrolled.set_visible(false);
            queue_cache.borrow_mut().clear();
            return;
        }

        placeholder.set_visible(false);
        scrolled.set_visible(true);

        // 清空旧内容
        while let Some(child) = listbox.first_child() {
            listbox.remove(&child);
        }

        for (index, song) in queue.iter().enumerate() {
            let is_current = current_idx >= 0 && current_idx as usize == index;
            let row = create_queue_row(song, index, is_current);
            listbox.append(&row);
        }

        *queue_cache.borrow_mut() = queue;
    }
}

/// 创建播放队列中的单行（当前播放项用 `play` 图标 + 高亮）。
fn create_queue_row(song: &Song, _index: usize, is_current: bool) -> gtk4::ListBoxRow {
    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S3);
    row_box.add_css_class("ngh-song-row");
    if is_current {
        row_box.add_css_class("playing");
    }

    // 序号 / 播放指示
    let indicator = if is_current {
        gtk4::Image::from_icon_name("media-playback-start")
    } else {
        gtk4::Image::from_icon_name("audio-x-generic")
    };
    indicator.set_pixel_size(18);
    indicator.set_valign(gtk4::Align::Center);

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

    row_box.append(&indicator);
    row_box.append(&info);
    row_box.append(&duration_label);

    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&row_box));
    row.set_focusable(false);
    row.set_activatable(true);
    row
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
