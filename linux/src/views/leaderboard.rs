//! 排行榜页视图。
//!
//! 展示音源排行榜列表与排行榜内歌曲，与 macOS 端对齐：
//! 通过下拉选择音源，加载该音源的排行榜列表，点击排行榜行展开显示其歌曲，
//! 点击歌曲加载队列并播放。
//!
//! 设计要点：
//! - 音源来自 `CoreService::source_list()`，下拉切换后异步加载排行榜。
//! - 排行榜获取为阻塞调用，在独立线程执行，通过 channel 回传主线程。
//! - 点击排行榜行切换展开态：展开时在该行下方显示歌曲列表，再次点击收起。

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::*;
use music_core::models::*;
use music_core::sources::SourceInfo;

use crate::core_service::CoreService;
use crate::player_service::PlayerService;
use crate::theme;
use crate::utils::{format_artists, format_duration};

/// 排行榜页内部状态。
struct LeaderboardState {
    /// 当前选中音源的排行榜列表。
    leaderboards: Vec<Leaderboard>,
    /// 当前展开的排行榜索引（None 表示全部收起）。
    expanded: Option<usize>,
}

/// 异步消息：排行榜加载线程 → 主线程。
enum LeaderboardMessage {
    /// 加载成功，携带排行榜列表。
    Loaded(Vec<Leaderboard>),
    /// 加载失败，携带错误描述。
    Error(String),
}

/// 创建排行榜页组件。
///
/// 布局自上而下：页面标题、音源选择下拉、加载提示、
/// 排行榜列表（`ScrolledWindow` 内嵌 `ListBox`）或空状态占位。
///
/// # 参数
/// - `player`：共享的播放器服务，用于加载队列与播放排行榜歌曲。
pub fn create_leaderboard_page(player: Arc<PlayerService>) -> gtk4::Widget {
    let state = Rc::new(RefCell::new(LeaderboardState {
        leaderboards: Vec::new(),
        expanded: None,
    }));

    let container = Box::new(Orientation::Vertical, 0);

    // --- 页面标题 ---
    let title = Label::new(Some("排行榜"));
    title.add_css_class("ngh-page-title");
    title.set_halign(Align::Start);
    title.set_margin_start(theme::SPACING_S4);
    title.set_margin_top(theme::SPACING_S4);
    title.set_margin_bottom(theme::SPACING_S2);
    container.append(&title);

    // --- 音源选择下拉 ---
    let selector_box = Box::new(Orientation::Horizontal, theme::SPACING_S2);
    selector_box.set_margin_start(theme::SPACING_S4);
    selector_box.set_margin_end(theme::SPACING_S4);
    selector_box.set_margin_bottom(theme::SPACING_S3);

    let source_label = Label::new(Some("音源："));
    source_label.add_css_class("ngh-label-secondary");
    source_label.set_halign(Align::Start);

    let sources: Vec<SourceInfo> = CoreService::instance().source_list();
    let source_names: Vec<&str> = sources.iter().map(|s| s.name.as_str()).collect();
    let dropdown = DropDown::from_strings(&source_names);
    dropdown.set_hexpand(true);

    selector_box.append(&source_label);
    selector_box.append(&dropdown);
    container.append(&selector_box);

    // --- 加载提示 ---
    let spinner = Spinner::new();
    let loading_label = Label::new(Some("加载排行榜…"));
    loading_label.add_css_class("ngh-loading");
    let loading_box = Box::new(Orientation::Horizontal, theme::SPACING_S2);
    loading_box.set_halign(Align::Center);
    loading_box.set_margin_bottom(theme::SPACING_S2);
    loading_box.append(&spinner);
    loading_box.append(&loading_label);
    loading_box.set_visible(false);
    container.append(&loading_box);

    // --- 占位 / 排行榜列表 ---
    let placeholder = Label::new(Some("选择一个音源以查看排行榜"));
    placeholder.add_css_class("ngh-empty-state");
    placeholder.set_vexpand(true);
    placeholder.set_valign(Align::Center);

    let listbox = ListBox::new();
    listbox.add_css_class("ngh-list");
    listbox.set_selection_mode(SelectionMode::None);

    let scrolled = ScrolledWindow::new(None::<&Adjustment>, None::<&Adjustment>);
    scrolled.set_vexpand(true);
    scrolled.set_child(Some(&listbox));
    scrolled.set_visible(false);

    container.append(&placeholder);
    container.append(&scrolled);

    // 通道：排行榜加载线程 → 主线程
    let (sender, receiver) =
        glib::MainContext::channel::<LeaderboardMessage>(glib::Priority::default());

    // --- 加载排行榜 ---
    let load_leaderboards = {
        let spinner = spinner.clone();
        let loading_box = loading_box.clone();
        let placeholder = placeholder.clone();
        let scrolled = scrolled.clone();
        let sender = sender.clone();
        move |source_id: String| {
            spinner.start();
            loading_box.set_visible(true);
            placeholder.set_visible(false);
            scrolled.set_visible(false);
            let sender = sender.clone();
            std::thread::spawn(move || {
                let core = CoreService::instance();
                let result = core.get_leaderboards(&source_id);
                let message = match result {
                    Ok(boards) => LeaderboardMessage::Loaded(boards),
                    Err(e) => LeaderboardMessage::Error(format!("{e}")),
                };
                let _ = sender.send(message);
            });
        }
    };

    // --- 下拉切换 ---
    {
        let sources = sources.clone();
        let load_leaderboards = load_leaderboards.clone();
        let state = Rc::clone(&state);
        dropdown.connect_selected_notify(move |dd| {
            let idx = dd.selected();
            if let Some(source) = sources.get(idx as usize) {
                state.borrow_mut().expanded = None;
                load_leaderboards(source.id.clone());
            }
        });
    }

    // --- 首次自动加载（若存在音源） ---
    if let Some(source) = sources.first() {
        load_leaderboards(source.id.clone());
    }

    // --- 接收排行榜加载结果 ---
    {
        let state = Rc::clone(&state);
        let spinner = spinner.clone();
        let loading_box = loading_box.clone();
        let placeholder = placeholder.clone();
        let scrolled = scrolled.clone();
        let listbox = listbox.clone();
        let player = Arc::clone(&player);
        receiver.attach(None, move |msg| {
            spinner.stop();
            loading_box.set_visible(false);
            match msg {
                LeaderboardMessage::Loaded(boards) => {
                    if boards.is_empty() {
                        placeholder.set_text("该音源暂无排行榜");
                        placeholder.set_visible(true);
                        scrolled.set_visible(false);
                    } else {
                        placeholder.set_visible(false);
                        scrolled.set_visible(true);
                        state.borrow_mut().leaderboards = boards;
                        state.borrow_mut().expanded = None;
                        rebuild_leaderboard_list(&state, &listbox, &player);
                    }
                }
                LeaderboardMessage::Error(err) => {
                    placeholder.set_text(&format!("排行榜加载失败：{err}"));
                    placeholder.set_visible(true);
                    scrolled.set_visible(false);
                }
            }
            glib::ControlFlow::Continue
        });
    }

    // --- 排行榜行激活：切换展开态 ---
    {
        let state = Rc::clone(&state);
        let listbox = listbox.clone();
        let player = Arc::clone(&player);
        listbox.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx < 0 {
                return;
            }
            let idx = idx as usize;
            let boards_len = state.borrow().leaderboards.len();
            if idx >= boards_len {
                return;
            }
            // 切换展开态
            {
                let mut s = state.borrow_mut();
                s.expanded = if s.expanded == Some(idx) {
                    None
                } else {
                    Some(idx)
                };
            }
            rebuild_leaderboard_list(&state, &listbox, &player);
        });
    }

    container.upcast::<Widget>()
}

/// 重建排行榜列表：按展开态渲染排行榜头与（展开时的）歌曲列表。
fn rebuild_leaderboard_list(
    state: &Rc<RefCell<LeaderboardState>>,
    listbox: &ListBox,
    player: &Arc<PlayerService>,
) {
    // 清空旧内容
    while let Some(child) = listbox.first_child() {
        listbox.remove(&child);
    }

    let s = state.borrow();
    let expanded = s.expanded;
    for (idx, board) in s.leaderboards.iter().enumerate() {
        let is_expanded = expanded == Some(idx);
        let row = create_leaderboard_row(board, is_expanded, player);
        listbox.append(&row);
    }
}

/// 创建单条排行榜行：头部（图标 + 名称 + 歌曲数），展开时附带歌曲列表。
fn create_leaderboard_row(
    board: &Leaderboard,
    is_expanded: bool,
    player: &Arc<PlayerService>,
) -> ListBoxRow {
    let row_box = Box::new(Orientation::Vertical, 0);
    row_box.add_css_class("ngh-song-row");

    // 头部
    let header = Box::new(Orientation::Horizontal, theme::SPACING_S3);
    header.set_margin_start(theme::SPACING_S4);
    header.set_margin_end(theme::SPACING_S4);
    header.set_margin_top(theme::SPACING_S3);
    header.set_margin_bottom(theme::SPACING_S3);

    let icon = Image::from_icon_name("media-playback-start");
    icon.set_pixel_size(18);
    icon.set_valign(Align::Center);

    let name_label = Label::new(Some(&board.name));
    name_label.add_css_class("ngh-song-title");
    name_label.set_halign(Align::Start);
    name_label.set_hexpand(true);

    let count_label = Label::new(Some(&format!("{} 首", board.songs.len())));
    count_label.add_css_class("ngh-label-secondary");
    count_label.set_valign(Align::Center);

    let expand_icon = Image::from_icon_name(if is_expanded {
        "go-down-symbolic"
    } else {
        "go-next-symbolic"
    });
    expand_icon.set_pixel_size(16);
    expand_icon.set_valign(Align::Center);

    header.append(&icon);
    header.append(&name_label);
    header.append(&count_label);
    header.append(&expand_icon);
    row_box.append(&header);

    // 展开时显示歌曲列表
    if is_expanded {
        let divider = Separator::new(Orientation::Horizontal);
        divider.set_margin_start(theme::SPACING_S4);
        divider.set_margin_end(theme::SPACING_S4);
        row_box.append(&divider);

        let songs_box = Box::new(Orientation::Vertical, 0);
        // 使用 Arc<Vec<Song>> 共享引用，避免每行 clone 整个 Vec 导致 O(n²) 内存开销。
        let songs: Arc<Vec<Song>> = Arc::new(board.songs.clone());
        for (song_idx, song) in board.songs.iter().enumerate() {
            let song_row = create_song_row(song, song_idx, Arc::clone(&songs), player);
            songs_box.append(&song_row);
        }
        row_box.append(&songs_box);
    }

    let row = ListBoxRow::new();
    row.set_child(Some(&row_box));
    row.set_focusable(false);
    row.set_activatable(true);
    row
}

/// 创建排行榜内单首歌曲行（序号 + 封面占位 + 标题/艺术家 + 时长）。
///
/// 点击歌曲行时将整个排行榜歌曲作为队列装载，并从当前歌曲开始播放。
fn create_song_row(
    song: &Song,
    index: usize,
    songs: Arc<Vec<Song>>,
    player: &Arc<PlayerService>,
) -> Widget {
    let row_box = Box::new(Orientation::Horizontal, theme::SPACING_S3);
    row_box.add_css_class("ngh-song-row");
    row_box.set_margin_start(theme::SPACING_S7);
    row_box.set_margin_end(theme::SPACING_S4);
    row_box.set_margin_top(theme::SPACING_S2);
    row_box.set_margin_bottom(theme::SPACING_S2);

    // 序号
    let index_label = Label::new(Some(&format!("{}", index + 1)));
    index_label.add_css_class("ngh-song-index");
    index_label.set_valign(Align::Center);

    // 封面占位
    let cover = Box::new(Orientation::Vertical, 0);
    cover.add_css_class("ngh-cover-placeholder");
    cover.set_size_request(28, 28);
    let cover_icon = Image::from_icon_name("audio-x-generic");
    cover_icon.set_pixel_size(14);
    cover.append(&cover_icon);

    // 标题 + 艺术家
    let info = Box::new(Orientation::Vertical, 2);
    info.set_hexpand(true);
    info.set_halign(Align::Start);
    let title_label = Label::new(Some(&song.title));
    title_label.add_css_class("ngh-song-title");
    title_label.set_halign(Align::Start);
    title_label.set_ellipsize(EllipsizeMode::End);
    let artist_label = Label::new(Some(&format_artists(&song.artists)));
    artist_label.add_css_class("ngh-song-artist");
    artist_label.set_halign(Align::Start);
    artist_label.set_ellipsize(EllipsizeMode::End);
    info.append(&title_label);
    info.append(&artist_label);

    // 时长
    let duration_label = Label::new(Some(&format_duration(song.duration_ms)));
    duration_label.add_css_class("ngh-song-duration");
    duration_label.set_valign(Align::Center);

    row_box.append(&index_label);
    row_box.append(&cover);
    row_box.append(&info);
    row_box.append(&duration_label);

    // 点击歌曲：装载排行榜队列并从当前歌曲开始播放
    let click = GestureClick::new();
    {
        let player = Arc::clone(player);
        click.connect_released(move |_, _, _, _| {
            player.load_queue((*songs).clone(), index);
        });
    }
    row_box.add_controller(click);

    row_box.upcast::<Widget>()
}
