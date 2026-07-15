//! 歌词页视图。
//!
//! 展示当前播放歌曲的同步歌词，与 macOS 端 `LyricsView.swift` 对齐：
//! 按行显示 LRC 歌词，当前行高亮并自动滚动居中，点击行跳转播放进度。
//!
//! 设计要点：
//! - 歌词获取为阻塞调用（`CoreService::get_lyric`），在独立线程执行，
//!   通过 `glib::MainContext::channel` 将结果回传到主线程。
//! - 通过 `glib::timeout_add_local` 每 500ms 轮询 `PlayerService::current_time()`，
//!   更新当前行高亮与滚动位置。
//! - 当前行使用自定义 CSS 类 `ngh-lyric-active`（主色 + 加粗）。

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gtk4::{glib, prelude::*};
use music_core::models::*;

use crate::core_service::CoreService;
use crate::player_service::PlayerService;
use crate::theme;
use crate::utils::format_artists;

/// 歌词页内部状态。
struct LyricsState {
    /// 当前歌词。
    lyric: Option<Lyric>,
    /// 当前行索引。
    active_line: Option<usize>,
    /// 是否正在加载。
    loading: bool,
}

/// 异步消息：歌词线程 → 主线程。
enum LyricsMessage {
    /// 歌词获取成功。
    Loaded(Lyric),
    /// 歌词获取失败。
    Error(String),
}

/// 行高亮轮询间隔。
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// 创建歌词页组件。
///
/// 布局自上而下：页面标题、歌曲信息栏（标题 + 艺术家 + 刷新按钮）、
/// 歌词列表（`ScrolledWindow` 内嵌 `ListBox`）或空状态 / 加载 / 错误占位。
///
/// # 参数
/// - `player`：共享的播放器服务，用于读取当前歌曲、获取播放时间与跳转进度。
pub fn create_lyrics_page(player: Arc<PlayerService>) -> gtk4::Widget {
    // 注册当前行高亮 CSS（主色 + 加粗）
    let css_provider = gtk4::CssProvider::new();
    let _ = css_provider.load_from_data(
        ".ngh-lyric-active { color: #4E6EF2; font-weight: 600; font-size: 16px; }",
    );
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::StyleContext::add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    let state = Rc::new(RefCell::new(LyricsState {
        lyric: None,
        active_line: None,
        loading: false,
    }));

    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

    // --- 页面标题 ---
    let title = gtk4::Label::new(Some("歌词"));
    title.add_css_class("ngh-page-title");
    title.set_halign(gtk4::Align::Start);
    title.set_margin_start(theme::SPACING_S4);
    title.set_margin_top(theme::SPACING_S4);
    title.set_margin_bottom(theme::SPACING_S2);
    container.append(&title);

    // --- 歌曲信息栏 ---
    let header = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S3);
    header.set_margin_start(theme::SPACING_S4);
    header.set_margin_end(theme::SPACING_S4);
    header.set_margin_bottom(theme::SPACING_S3);

    let song_icon = gtk4::Image::from_icon_name("audio-x-generic");
    song_icon.set_pixel_size(20);

    let info_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    info_box.set_hexpand(true);
    info_box.set_halign(gtk4::Align::Start);

    let song_title = gtk4::Label::new(Some("未在播放"));
    song_title.add_css_class("ngh-label-primary");
    song_title.set_halign(gtk4::Align::Start);
    song_title.set_ellipsize(gtk4::EllipsizeMode::End);

    let song_artist = gtk4::Label::new(Some("—"));
    song_artist.add_css_class("ngh-label-secondary");
    song_artist.set_halign(gtk4::Align::Start);
    song_artist.set_ellipsize(gtk4::EllipsizeMode::End);

    info_box.append(&song_title);
    info_box.append(&song_artist);

    let refresh_button = gtk4::Button::with_icon_name("view-refresh");
    refresh_button.add_css_class("ngh-ghost-button");
    refresh_button.set_tooltip_text(Some("刷新歌词"));

    header.append(&song_icon);
    header.append(&info_box);
    header.append(&refresh_button);
    container.append(&header);

    // 分割线
    let divider = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    container.append(&divider);

    // --- 加载提示 ---
    let spinner = gtk4::Spinner::new();
    let loading_label = gtk4::Label::new(Some("加载歌词…"));
    loading_label.add_css_class("ngh-loading");
    let loading_box = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S2);
    loading_box.set_halign(gtk4::Align::Center);
    loading_box.set_vexpand(true);
    loading_box.set_valign(gtk4::Align::Center);
    loading_box.append(&spinner);
    loading_box.append(&loading_label);
    loading_box.set_visible(false);
    container.append(&loading_box);

    // --- 空状态 / 错误提示 ---
    let placeholder = gtk4::Label::new(Some("暂无歌词，播放歌曲后将自动加载歌词"));
    placeholder.add_css_class("ngh-empty-state");
    placeholder.set_vexpand(true);
    placeholder.set_valign(gtk4::Align::Center);
    container.append(&placeholder);

    // --- 歌词列表 ---
    let listbox = gtk4::ListBox::new();
    listbox.add_css_class("ngh-list");
    listbox.set_selection_mode(gtk4::SelectionMode::None);

    let scrolled = gtk4::ScrolledWindow::new(None::<&gtk4::Adjustment>, None::<&gtk4::Adjustment>);
    scrolled.set_vexpand(true);
    scrolled.set_child(Some(&listbox));
    scrolled.set_visible(false);
    container.append(&scrolled);

    // 通道：歌词线程 → 主线程
    let (sender, receiver) =
        glib::MainContext::channel::<LyricsMessage>(glib::Priority::default());

    // --- 加载歌词（首次加载与刷新按钮共用同一闭包，避免重复实现） ---
    let load_lyric = build_load_lyric_closure(
        &player,
        &state,
        &spinner,
        &loading_box,
        &placeholder,
        &scrolled,
        &listbox,
        &sender,
    );

    // --- 更新歌曲信息栏 ---
    let update_song_info = {
        let player = Arc::clone(&player);
        let song_title = song_title.clone();
        let song_artist = song_artist.clone();
        move || {
            match player.current_song() {
                Some(song) => {
                    song_title.set_text(&song.title);
                    song_artist.set_text(&format_artists(&song.artists));
                }
                None => {
                    song_title.set_text("未在播放");
                    song_artist.set_text("—");
                }
            }
        }
    };

    // --- 首次加载 ---
    update_song_info();
    load_lyric();

    // --- 刷新按钮 ---
    refresh_button.connect_clicked(move |_| {
        load_lyric();
    });

    // --- 接收歌词结果 ---
    {
        let state = Rc::clone(&state);
        let spinner = spinner.clone();
        let loading_box = loading_box.clone();
        let placeholder = placeholder.clone();
        let scrolled = scrolled.clone();
        let listbox = listbox.clone();
        receiver.attach(None, move |msg| {
            spinner.stop();
            loading_box.set_visible(false);
            state.borrow_mut().loading = false;

            match msg {
                LyricsMessage::Loaded(lyric) => {
                    if lyric.lines.is_empty() {
                        placeholder.set_text("该歌曲暂无歌词");
                        placeholder.set_visible(true);
                        scrolled.set_visible(false);
                        state.borrow_mut().lyric = None;
                    } else {
                        placeholder.set_visible(false);
                        scrolled.set_visible(true);
                        rebuild_lyrics_list(&listbox, &lyric);
                        state.borrow_mut().lyric = Some(lyric);
                        state.borrow_mut().active_line = None;
                    }
                }
                LyricsMessage::Error(err) => {
                    placeholder.set_text(&format!("歌词加载失败：{err}"));
                    placeholder.set_visible(true);
                    scrolled.set_visible(false);
                    state.borrow_mut().lyric = None;
                }
            }
            glib::ControlFlow::Continue
        });
    }

    // --- 行激活（点击）跳转进度 ---
    {
        let state = Rc::clone(&state);
        let player = Arc::clone(&player);
        listbox.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx >= 0 {
                let idx = idx as usize;
                let s = state.borrow();
                if let Some(lyric) = &s.lyric {
                    if idx < lyric.lines.len() {
                        if let Some(time_ms) = lyric.lines[idx].time_ms {
                            player.seek(time_ms as f64 / 1000.0);
                        }
                    }
                }
            }
        });
    }

    // --- 定时轮询：高亮当前行 + 自动滚动 ---
    {
        let state = Rc::clone(&state);
        let player = Arc::clone(&player);
        let listbox = listbox.clone();
        let scrolled = scrolled.clone();
        let source_id = glib::timeout_add_local(POLL_INTERVAL, move || {
            let time_ms = (player.current_time() * 1000.0) as u64;
            let new_active = {
                let s = state.borrow();
                s.lyric
                    .as_ref()
                    .and_then(|lyric| active_line_index(lyric, time_ms))
            };
            let prev_active = state.borrow().active_line;
            if new_active != prev_active {
                state.borrow_mut().active_line = new_active;
                update_active_highlight(&listbox, prev_active, new_active);
                if let Some(idx) = new_active {
                    scroll_to_row(&scrolled, &listbox, idx);
                }
            }
            glib::ControlFlow::Continue
        });
        // 组件销毁时移除定时器，避免泄漏与对已销毁组件的访问
        container.connect_destroy(move |_| {
            source_id.remove();
        });
    }

    container.upcast::<gtk4::Widget>()
}

/// 构造加载歌词的闭包，供刷新按钮使用。
fn build_load_lyric_closure(
    player: &Arc<PlayerService>,
    state: &Rc<RefCell<LyricsState>>,
    spinner: &gtk4::Spinner,
    loading_box: &gtk4::Box,
    placeholder: &gtk4::Label,
    scrolled: &gtk4::ScrolledWindow,
    listbox: &gtk4::ListBox,
    sender: &glib::Sender<LyricsMessage>,
) -> impl Fn() + 'static {
    let player = Arc::clone(player);
    let state = Rc::clone(state);
    let spinner = spinner.clone();
    let loading_box = loading_box.clone();
    let placeholder = placeholder.clone();
    let scrolled = scrolled.clone();
    let listbox = listbox.clone();
    let sender = sender.clone();
    move || {
        let song = match player.current_song() {
            Some(s) => s,
            None => {
                let mut s = state.borrow_mut();
                s.lyric = None;
                s.active_line = None;
                drop(s);
                spinner.stop();
                loading_box.set_visible(false);
                listbox_cleanup(&listbox);
                placeholder.set_visible(true);
                scrolled.set_visible(false);
                return;
            }
        };
        {
            let mut s = state.borrow_mut();
            s.loading = true;
        }
        spinner.start();
        loading_box.set_visible(true);
        placeholder.set_visible(false);
        scrolled.set_visible(false);

        let source_id = song.source_id.clone();
        let song_id = song.id.clone();
        let sender = sender.clone();
        std::thread::spawn(move || {
            let core = CoreService::instance();
            let result = core.get_lyric(&source_id, &song_id);
            let message = match result {
                Ok(lyric) => LyricsMessage::Loaded(lyric),
                Err(e) => LyricsMessage::Error(format!("{e}")),
            };
            let _ = sender.send(message);
        });
    }
}

/// 重建歌词列表：清空旧行后逐行追加，无时间戳的行显示为次要文本。
fn rebuild_lyrics_list(listbox: &gtk4::ListBox, lyric: &Lyric) {
    while let Some(child) = listbox.first_child() {
        listbox.remove(&child);
    }
    for (idx, line) in lyric.lines.iter().enumerate() {
        let row = create_lyric_row(&line.text, idx);
        listbox.append(&row);
    }
}

/// 清空歌词列表内容。
fn listbox_cleanup(listbox: &gtk4::ListBox) {
    while let Some(child) = listbox.first_child() {
        listbox.remove(&child);
    }
}

/// 创建单行歌词：默认次要文本色，高亮态由 `update_active_highlight` 切换 CSS 类。
fn create_lyric_row(text: &str, _idx: usize) -> gtk4::ListBoxRow {
    let label = gtk4::Label::new(Some(text));
    label.add_css_class("ngh-label-secondary");
    label.set_halign(gtk4::Align::Start);
    label.set_hexpand(true);
    label.set_wrap(true);
    label.set_margin_start(theme::SPACING_S4);
    label.set_margin_end(theme::SPACING_S4);
    label.set_margin_top(theme::SPACING_S1);
    label.set_margin_bottom(theme::SPACING_S1);

    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&label));
    row.set_focusable(false);
    row.set_activatable(true);
    row
}

/// 更新当前行高亮：仅更新上一行与当前行，避免遍历所有行。
fn update_active_highlight(listbox: &gtk4::ListBox, prev: Option<usize>, active: Option<usize>) {
    // 移除上一行的高亮
    if let Some(idx) = prev {
        if let Some(row) = listbox.row_at_index(idx as i32) {
            if let Some(label) = row.child().and_then(|c| c.downcast::<gtk4::Label>().ok()) {
                label.remove_css_class("ngh-lyric-active");
                label.add_css_class("ngh-label-secondary");
            }
        }
    }
    // 高亮当前行
    if let Some(idx) = active {
        if let Some(row) = listbox.row_at_index(idx as i32) {
            if let Some(label) = row.child().and_then(|c| c.downcast::<gtk4::Label>().ok()) {
                label.remove_css_class("ngh-label-secondary");
                label.add_css_class("ngh-lyric-active");
            }
        }
    }
}

/// 将 `ScrolledWindow` 滚动到指定行，使其居中显示。
fn scroll_to_row(scrolled: &gtk4::ScrolledWindow, listbox: &gtk4::ListBox, idx: usize) {
    if let Some(row) = listbox.row_at_index(idx as i32) {
        let alloc = row.allocation();
        let adj = scrolled.vadjustment();
        let row_top = alloc.y() as f64;
        let row_height = alloc.height() as f64;
        let page_size = adj.page_size();
        let current = adj.value();

        // 当前行不在可视区域时才滚动，避免频繁跳动
        if row_top < current || row_top + row_height > current + page_size {
            let target = row_top - (page_size - row_height) / 2.0;
            let clamped = target.max(0.0).min(adj.upper() - page_size);
            adj.set_value(clamped);
        }
    }
}

/// 根据当前播放时间（毫秒）查找对应的活跃行索引。
///
/// 遍历歌词行，返回最后一个 `time_ms <= time_ms` 的行索引；
/// 无时间戳行不参与判断。
fn active_line_index(lyric: &Lyric, time_ms: u64) -> Option<usize> {
    let mut idx: Option<usize> = None;
    for (i, line) in lyric.lines.iter().enumerate() {
        if let Some(t) = line.time_ms {
            if t <= time_ms {
                idx = Some(i);
            } else {
                break;
            }
        }
    }
    idx
}
