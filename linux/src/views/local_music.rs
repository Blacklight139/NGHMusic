//! 本地音乐页视图。
//!
//! 管理本地音乐扫描目录与展示已索引歌曲，与 macOS 端对齐：
//! 提供添加目录、重新扫描操作，实时显示扫描进度，点击歌曲行加载队列并播放。
//!
//! 设计要点：
//! - 本地索引库路径固定为 `~/.local/share/nghmusic/local.db`，首次添加目录前自动初始化。
//! - 扫描进度通过 `glib::timeout_add_local` 每 500ms 轮询 `local_progress()` 更新标签。
//! - 歌曲列表来自 `CoreService::search`（空关键词），在独立线程执行避免阻塞 UI。

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::*;
use music_core::models::*;

use crate::core_service::CoreService;
use crate::player_service::PlayerService;
use crate::theme;
use crate::utils::{format_artists, format_duration};

/// 本地音乐页内部状态。
struct LocalMusicState {
    /// 当前歌曲列表。
    songs: Vec<Song>,
}

/// 异步消息：工作线程 → 主线程。
enum LocalMusicMessage {
    /// 搜索成功，携带歌曲列表。
    Loaded(Vec<Song>),
    /// 搜索失败，携带错误描述。
    Error(String),
    /// 添加目录成功。
    DirAdded,
    /// 添加目录失败，携带错误描述。
    DirAddFailed(String),
    /// 重新扫描完成。
    RescanDone,
    /// 重新扫描失败，携带错误描述。
    RescanFailed(String),
}

/// 进度轮询间隔。
const POLL_INTERVAL: Duration = Duration::from_millis(500);

/// 创建本地音乐页组件。
///
/// 布局自上而下：页面标题、操作栏（添加目录 + 重新扫描 + 进度标签）、
/// 歌曲列表（`ScrolledWindow` 内嵌 `ListBox`）或空状态占位。
///
/// # 参数
/// - `player`：共享的播放器服务，用于加载队列与播放本地歌曲。
pub fn create_local_music_page(player: Arc<PlayerService>) -> gtk4::Widget {
    let state = Rc::new(RefCell::new(LocalMusicState { songs: Vec::new() }));

    let container = Box::new(Orientation::Vertical, 0);

    // --- 页面标题 ---
    let title = Label::new(Some("本地音乐"));
    title.add_css_class("ngh-page-title");
    title.set_halign(Align::Start);
    title.set_margin_start(theme::SPACING_S4);
    title.set_margin_top(theme::SPACING_S4);
    title.set_margin_bottom(theme::SPACING_S2);
    container.append(&title);

    // --- 操作栏 ---
    let toolbar = Box::new(Orientation::Horizontal, theme::SPACING_S2);
    toolbar.set_margin_start(theme::SPACING_S4);
    toolbar.set_margin_end(theme::SPACING_S4);
    toolbar.set_margin_bottom(theme::SPACING_S3);

    let add_button = Button::with_label("添加目录");
    add_button.add_css_class("ngh-primary-button");

    let rescan_button = Button::with_label("重新扫描");
    rescan_button.add_css_class("ngh-ghost-button");

    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);

    let progress_label = Label::new(Some("就绪"));
    progress_label.add_css_class("ngh-label-secondary");

    toolbar.append(&add_button);
    toolbar.append(&rescan_button);
    toolbar.append(&spacer);
    toolbar.append(&progress_label);
    container.append(&toolbar);

    // --- 占位 / 歌曲列表 ---
    let placeholder = Label::new(Some("尚无本地音乐，点击「添加目录」选择音乐文件夹"));
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

    // 通道：搜索线程 → 主线程
    let (sender, receiver) =
        glib::MainContext::channel::<LocalMusicMessage>(glib::Priority::default());

    // --- 加载本地歌曲 ---
    // 使用 local_list_songs 而非聚合 search("", ...)，仅返回本地音源已索引歌曲，
    // 避免把远程音源结果混入本地音乐页。
    let load_songs = {
        let sender = sender.clone();
        move || {
            let sender = sender.clone();
            std::thread::spawn(move || {
                let core = CoreService::instance();
                let message = match core.local_list_songs() {
                    Ok(songs) => LocalMusicMessage::Loaded(songs),
                    Err(e) => LocalMusicMessage::Error(format!("{e}")),
                };
                let _ = sender.send(message);
            });
        }
    };

    // --- 添加目录按钮 ---
    {
        let sender = sender.clone();
        add_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("选择音乐目录"),
                None::<&Window>,
                FileChooserAction::SelectFolder,
                &[
                    ("取消", ResponseType::Cancel),
                    ("添加", ResponseType::Accept),
                ],
            );
            let sender = sender.clone();
            dialog.connect_response(move |d, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = d.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            let sender = sender.clone();
                            // 在工作线程执行本地索引库初始化与目录添加，
                            // 仅捕获 Send 数据（sender + path_str），结果通过 channel 回传主线程。
                            std::thread::spawn(move || {
                                let core = CoreService::instance();
                                let db_path = local_db_path().to_string_lossy().to_string();
                                let _ = core.local_init(&db_path);
                                let message = match core.local_add_dir(&path_str) {
                                    Ok(()) => LocalMusicMessage::DirAdded,
                                    Err(e) => LocalMusicMessage::DirAddFailed(format!("{e}")),
                                };
                                let _ = sender.send(message);
                            });
                        }
                    }
                }
                d.close();
            });
            dialog.show();
        });
    }

    // --- 重新扫描按钮 ---
    {
        let sender = sender.clone();
        let progress_label = progress_label.clone();
        rescan_button.connect_clicked(move |_| {
            progress_label.set_text("扫描中…");
            let sender = sender.clone();
            // 在工作线程执行重新扫描，仅捕获 Send 数据（sender），结果通过 channel 回传主线程。
            std::thread::spawn(move || {
                let core = CoreService::instance();
                let db_path = local_db_path().to_string_lossy().to_string();
                let _ = core.local_init(&db_path);
                let message = match core.local_rescan() {
                    Ok(()) => LocalMusicMessage::RescanDone,
                    Err(e) => LocalMusicMessage::RescanFailed(format!("{e}")),
                };
                let _ = sender.send(message);
            });
        });
    }

    // --- 接收歌曲加载与操作结果 ---
    {
        let state = Rc::clone(&state);
        let placeholder = placeholder.clone();
        let scrolled = scrolled.clone();
        let listbox = listbox.clone();
        let player = Arc::clone(&player);
        let progress_label = progress_label.clone();
        let load_songs = load_songs.clone();
        receiver.attach(None, move |msg| {
            match msg {
                LocalMusicMessage::Loaded(songs) => {
                    if songs.is_empty() {
                        placeholder.set_text("未找到本地歌曲，请先添加音乐目录");
                        placeholder.set_visible(true);
                        scrolled.set_visible(false);
                    } else {
                        placeholder.set_visible(false);
                        scrolled.set_visible(true);
                        rebuild_song_list(&listbox, &songs, &player);
                        state.borrow_mut().songs = songs;
                    }
                }
                LocalMusicMessage::Error(err) => {
                    placeholder.set_text(&format!("加载失败：{err}"));
                    placeholder.set_visible(true);
                    scrolled.set_visible(false);
                }
                LocalMusicMessage::DirAdded => {
                    progress_label.set_text("目录添加成功");
                    load_songs();
                }
                LocalMusicMessage::DirAddFailed(err) => {
                    progress_label.set_text(&format!("添加失败：{err}"));
                }
                LocalMusicMessage::RescanDone => {
                    progress_label.set_text("扫描完成");
                    load_songs();
                }
                LocalMusicMessage::RescanFailed(err) => {
                    progress_label.set_text(&format!("扫描失败：{err}"));
                }
            }
            glib::ControlFlow::Continue
        });
    }

    // --- 歌曲行激活播放 ---
    {
        let state = Rc::clone(&state);
        let player = Arc::clone(&player);
        listbox.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx >= 0 {
                let idx = idx as usize;
                let songs = state.borrow().songs.clone();
                if idx < songs.len() {
                    player.load_queue(songs, idx);
                }
            }
        });
    }

    // --- 进度轮询：每 500ms 更新扫描进度标签 ---
    {
        let progress_label = progress_label.clone();
        let source_id = glib::timeout_add_local(POLL_INTERVAL, move || {
            let (count, scanning) = CoreService::instance().local_progress();
            if scanning {
                progress_label.set_text(&format!("扫描中… 已索引 {count} 首"));
            } else if count > 0 {
                progress_label.set_text(&format!("已索引 {count} 首"));
            }
            glib::ControlFlow::Continue
        });
        // 组件销毁时移除定时器，避免泄漏与对已销毁组件的访问
        container.connect_destroy(move |_| {
            source_id.remove();
        });
    }

    // 首次加载
    load_songs();

    container.upcast::<Widget>()
}

/// 重建歌曲列表：清空旧行后逐条追加。
fn rebuild_song_list(listbox: &ListBox, songs: &[Song], player: &Arc<PlayerService>) {
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
fn create_song_row(song: &Song, index: usize, is_playing: bool) -> ListBoxRow {
    let row_box = Box::new(Orientation::Horizontal, theme::SPACING_S3);
    row_box.add_css_class("ngh-song-row");
    if is_playing {
        row_box.add_css_class("playing");
    }

    // 序号
    let index_label = Label::new(Some(&format!("{}", index + 1)));
    index_label.add_css_class("ngh-song-index");
    index_label.set_valign(Align::Center);

    // 封面占位
    let cover = Box::new(Orientation::Vertical, 0);
    cover.add_css_class("ngh-cover-placeholder");
    cover.set_size_request(32, 32);
    let cover_icon = Image::from_icon_name("audio-x-generic");
    cover_icon.set_pixel_size(16);
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

    // 来源标签
    let source_tag = Label::new(Some(&song.source_id));
    source_tag.add_css_class("ngh-source-tag");
    source_tag.set_valign(Align::Center);

    // 时长
    let duration_label = Label::new(Some(&format_duration(song.duration_ms)));
    duration_label.add_css_class("ngh-song-duration");
    duration_label.set_valign(Align::Center);

    row_box.append(&index_label);
    row_box.append(&cover);
    row_box.append(&info);
    row_box.append(&source_tag);
    row_box.append(&duration_label);

    let row = ListBoxRow::new();
    row.set_child(Some(&row_box));
    row.set_focusable(false);
    row.set_activatable(true);
    row
}

/// 本地索引库路径：`~/.local/share/nghmusic/local.db`。
fn local_db_path() -> PathBuf {
    let base = dirs::data_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join("nghmusic").join("local.db")
}
