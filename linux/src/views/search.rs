//! 搜索页视图。
//!
//! 提供搜索框、加载状态与聚合搜索结果列表（歌曲行）。与 macOS 端
//! `SearchView.swift` 对齐：点击歌曲行加载队列并播放，当前播放项高亮。
//!
//! 设计要点：
//! - 搜索为阻塞调用（`CoreService::search` 内部 `block_on`），因此在独立线程执行，
//!   通过 `glib::MainContext::channel` 将结果回传到 GTK 主线程更新 UI。
//! - 歌曲行采用线性列表风格（序号 + 封面占位 + 标题/艺术家 + 来源标签 + 时长）。
//! - 当前播放歌曲通过 CSS `.ngh-song-row.playing` 高亮。

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk4::{glib, prelude::*};
use music_core::models::*;

use crate::core_service::CoreService;
use crate::player_service::PlayerService;
use crate::theme;

/// 搜索页内部状态。
struct SearchState {
    /// 当前搜索结果歌曲列表。
    results: Vec<Song>,
    /// 是否正在搜索中。
    loading: bool,
}

/// 异步消息：搜索线程 → 主线程。
enum SearchMessage {
    /// 搜索成功，携带歌曲列表。
    Loaded(Vec<Song>),
    /// 搜索失败，携带错误描述。
    Error(String),
}

/// 单页搜索的页大小。
const PAGE_SIZE: u32 = 20;

/// 创建搜索页组件。
///
/// 布局自上而下：页面标题、搜索栏（输入框 + 搜索按钮）、加载提示、
/// 结果列表（`ScrolledWindow` 内嵌 `ListBox`）或空状态占位。
///
/// # 参数
/// - `player`：共享的播放器服务，用于加载队列与判断当前播放项。
pub fn create_search_page(player: Arc<PlayerService>) -> gtk4::Widget {
    let state = Rc::new(RefCell::new(SearchState {
        results: Vec::new(),
        loading: false,
    }));

    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // --- 页面标题 ---
    let title = gtk4::Label::new(Some("搜索"));
    title.add_css_class("ngh-page-title");
    title.set_halign(gtk4::Align::Start);
    title.set_margin_start(theme::SPACING_S4);
    title.set_margin_top(theme::SPACING_S4);
    title.set_margin_bottom(theme::SPACING_S2);
    container.append(&title);

    // --- 搜索栏 ---
    let search_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S2);
    search_bar.set_margin_start(theme::SPACING_S4);
    search_bar.set_margin_end(theme::SPACING_S4);
    search_bar.set_margin_bottom(theme::SPACING_S3);

    let entry = gtk4::Entry::new();
    entry.add_css_class("ngh-search-entry");
    entry.set_placeholder_text(Some("搜索歌曲、专辑、艺术家"));
    entry.set_hexpand(true);

    let search_button = gtk4::Button::with_label("搜索");
    search_button.add_css_class("ngh-primary-button");

    search_bar.append(&entry);
    search_bar.append(&search_button);
    container.append(&search_bar);

    // --- 加载提示 ---
    let spinner = gtk4::Spinner::new();
    spinner.set_visible(false);
    let loading_label = gtk4::Label::new(Some("搜索中…"));
    loading_label.add_css_class("ngh-loading");
    loading_label.set_visible(false);
    let loading_box = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S2);
    loading_box.set_halign(gtk4::Align::Center);
    loading_box.set_margin_bottom(theme::SPACING_S2);
    loading_box.append(&spinner);
    loading_box.append(&loading_label);
    container.append(&loading_box);

    // --- 占位 / 结果区 ---
    let placeholder = gtk4::Label::new(Some("搜索您喜欢的音乐，支持跨音源聚合搜索"));
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

    // 通道：搜索线程 → 主线程
    let (sender, receiver) =
        glib::MainContext::channel::<SearchMessage>(glib::Priority::default());

    // --- 搜索按钮点击 ---
    {
        let entry = entry.clone();
        let spinner = spinner.clone();
        let loading_label = loading_label.clone();
        let sender = sender.clone();
        let state = Rc::clone(&state);
        search_button.connect_clicked(move |_| {
            trigger_search(&entry, &spinner, &loading_label, &sender, &state);
        });
    }

    // --- 回车触发搜索 ---
    {
        let entry = entry.clone();
        let spinner = spinner.clone();
        let loading_label = loading_label.clone();
        let sender = sender.clone();
        let state = Rc::clone(&state);
        entry.connect_activate(move |_| {
            trigger_search(&entry, &spinner, &loading_label, &sender, &state);
        });
    }

    // --- 行激活（双击/Enter）播放 ---
    {
        let state = Rc::clone(&state);
        let player = Arc::clone(&player);
        listbox.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx >= 0 {
                let songs = state.borrow().results.clone();
                let idx = idx as usize;
                if idx < songs.len() {
                    player.load_queue(songs, idx);
                }
            }
        });
    }

    // --- 接收搜索结果 ---
    {
        let state = Rc::clone(&state);
        let spinner = spinner.clone();
        let loading_label = loading_label.clone();
        let listbox = listbox.clone();
        let placeholder = placeholder.clone();
        let scrolled = scrolled.clone();
        let player = Arc::clone(&player);
        receiver.attach(None, move |msg| {
            spinner.set_visible(false);
            spinner.stop();
            loading_label.set_visible(false);
            state.borrow_mut().loading = false;

            match msg {
                SearchMessage::Loaded(songs) => {
                    if songs.is_empty() {
                        placeholder.set_text("没有找到相关结果，尝试使用其他关键词");
                        placeholder.set_visible(true);
                        scrolled.set_visible(false);
                    } else {
                        placeholder.set_visible(false);
                        scrolled.set_visible(true);
                        rebuild_search_list(&listbox, &songs, &player);
                        state.borrow_mut().results = songs;
                    }
                }
                SearchMessage::Error(err) => {
                    placeholder.set_text(&format!("搜索失败：{err}"));
                    placeholder.set_visible(true);
                    scrolled.set_visible(false);
                }
            }
            glib::ControlFlow::Continue
        });
    }

    container.upcast::<gtk4::Widget>()
}

/// 触发一次搜索：设置加载态，在独立线程调用 `CoreService::search`。
fn trigger_search(
    entry: &gtk4::Entry,
    spinner: &gtk4::Spinner,
    loading_label: &gtk4::Label,
    sender: &glib::Sender<SearchMessage>,
    state: &Rc<RefCell<SearchState>>,
) {
    let keyword = entry.text().trim().to_string();
    if keyword.is_empty() {
        return;
    }
    // 防止并发搜索
    {
        let mut s = state.borrow_mut();
        if s.loading {
            return;
        }
        s.loading = true;
    }
    spinner.set_visible(true);
    spinner.start();
    loading_label.set_visible(true);

    let sender = sender.clone();
    std::thread::spawn(move || {
        let core = CoreService::instance();
        let result = core.search(&keyword, 1, PAGE_SIZE);
        let message = match result {
            Ok(r) => SearchMessage::Loaded(r.songs),
            Err(e) => SearchMessage::Error(format!("{e}")),
        };
        let _ = sender.send(message);
    });
}

/// 重建搜索结果列表：清空旧行后逐条追加。
fn rebuild_search_list(listbox: &gtk4::ListBox, songs: &[Song], player: &Arc<PlayerService>) {
    // 清空旧内容
    while let Some(child) = listbox.first_child() {
        listbox.remove(&child);
    }

    let current = player.current_song();
    let current_key = current.as_ref().map(|s| (s.id.clone(), s.source_id.clone()));

    for (index, song) in songs.iter().enumerate() {
        let is_playing = current_key
            .as_ref()
            .map(|(id, sid)| id == &song.id && sid == &song.source_id)
            .unwrap_or(false);
        let row = create_song_row(song, index, is_playing);
        listbox.append(&row);
    }
}

/// 创建单条歌曲行（序号 + 封面占位 + 标题/艺术家 + 来源标签 + 时长）。
fn create_song_row(song: &Song, index: usize, is_playing: bool) -> gtk4::ListBoxRow {
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

    // 来源标签
    let source_tag = gtk4::Label::new(Some(&song.source_id));
    source_tag.add_css_class("ngh-source-tag");
    source_tag.set_valign(gtk4::Align::Center);

    // 时长
    let duration_label = gtk4::Label::new(Some(&format_duration(song.duration_ms)));
    duration_label.add_css_class("ngh-song-duration");
    duration_label.set_valign(gtk4::Align::Center);

    row_box.append(&index_label);
    row_box.append(&cover);
    row_box.append(&info);
    row_box.append(&source_tag);
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
