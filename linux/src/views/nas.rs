//! NAS 页视图。
//!
//! 提供飞牛 NAS 登录、文件浏览与协议源管理，与 macOS 端对齐：
//! 登录后可浏览 NAS 文件，双击目录进入下级，双击音频文件流式播放。
//! 同时展示已注册的远程协议源（WebDAV/FTP/SMB 等）并支持删除。
//!
//! 设计要点：
//! - 登录与文件列表为阻塞调用，在独立线程执行，通过 channel 回传主线程。
//! - 文件浏览器维护当前路径状态（`Rc<RefCell<String>>`），支持进入目录与返回上级。
//! - 协议源列表来自 `CoreService::protocol_list()`，删除后实时刷新。

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::*;
use music_core::feiniu::NasFile;
use music_core::models::*;

use crate::core_service::{CoreService, ProtocolSourceInfo};
use crate::player_service::PlayerService;
use crate::theme;

/// 异步消息：NAS 线程 → 主线程。
enum NasMessage {
    /// 登录成功。
    LoginOk(String),
    /// 登录失败。
    LoginError(String),
    /// 文件列表加载成功。
    FilesLoaded(Vec<NasFile>),
    /// 文件列表加载失败。
    FilesError(String),
    /// 流式 URL 获取成功。
    StreamOk(String, String),
    /// 流式 URL 获取失败。
    StreamError(String),
    /// 健康检查成功。
    HealthOk,
    /// 健康检查失败。
    HealthError(String),
}

/// 创建 NAS 页组件。
///
/// 布局自上而下：页面标题、飞牛登录区（URL + 用户名 + 密码 + 登录/健康检查按钮）、
/// 文件浏览区（当前路径 + 返回按钮 + 文件列表）、协议源管理区（列表 + 删除按钮）。
///
/// # 参数
/// - `player`：共享的播放器服务，用于流式播放 NAS 文件。
pub fn create_nas_page(player: Arc<PlayerService>) -> gtk4::Widget {
    // 当前浏览路径
    let current_path: Rc<RefCell<String>> = Rc::new(RefCell::new(String::from("/")));

    let container = Box::new(Orientation::Vertical, 0);

    // --- 页面标题 ---
    let title = Label::new(Some("NAS"));
    title.add_css_class("ngh-page-title");
    title.set_halign(Align::Start);
    title.set_margin_start(theme::SPACING_S4);
    title.set_margin_top(theme::SPACING_S4);
    title.set_margin_bottom(theme::SPACING_S2);
    container.append(&title);

    // 通道：NAS 线程 → 主线程
    let (sender, receiver) =
        glib::MainContext::channel::<NasMessage>(glib::Priority::default());

    // ===================================================================
    // 飞牛登录区
    // ===================================================================
    let login_section = Box::new(Orientation::Vertical, theme::SPACING_S2);
    login_section.set_margin_start(theme::SPACING_S4);
    login_section.set_margin_end(theme::SPACING_S4);
    login_section.set_margin_bottom(theme::SPACING_S3);

    let login_header = Label::new(Some("飞牛 NAS 登录"));
    login_header.add_css_class("ngh-label-primary");
    login_header.set_halign(Align::Start);

    let url_entry = Entry::new();
    url_entry.set_placeholder_text(Some("服务地址（如 https://nas.example.com）"));
    url_entry.set_hexpand(true);

    let user_entry = Entry::new();
    user_entry.set_placeholder_text(Some("用户名"));

    let pass_entry = PasswordEntry::new();
    pass_entry.set_placeholder_text(Some("密码"));
    pass_entry.set_show_peek_icon(true);

    let login_row = Box::new(Orientation::Horizontal, theme::SPACING_S2);
    let login_button = Button::with_label("登录");
    login_button.add_css_class("ngh-primary-button");
    let health_button = Button::with_label("健康检查");
    health_button.add_css_class("ngh-ghost-button");

    login_row.append(&login_button);
    login_row.append(&health_button);

    let status_label = Label::new(Some("尚未登录"));
    status_label.add_css_class("ngh-label-secondary");
    status_label.set_halign(Align::Start);

    login_section.append(&login_header);
    login_section.append(&url_entry);
    login_section.append(&user_entry);
    login_section.append(&pass_entry);
    login_section.append(&login_row);
    login_section.append(&status_label);
    container.append(&login_section);

    // 分割线
    let divider1 = Separator::new(Orientation::Horizontal);
    divider1.set_margin_start(theme::SPACING_S4);
    divider1.set_margin_end(theme::SPACING_S4);
    divider1.set_margin_bottom(theme::SPACING_S3);
    container.append(&divider1);

    // ===================================================================
    // 文件浏览区
    // ===================================================================
    let browser_section = Box::new(Orientation::Vertical, theme::SPACING_S2);
    browser_section.set_margin_start(theme::SPACING_S4);
    browser_section.set_margin_end(theme::SPACING_S4);
    browser_section.set_margin_bottom(theme::SPACING_S3);

    let browser_header_row = Box::new(Orientation::Horizontal, theme::SPACING_S2);
    let browser_header = Label::new(Some("文件浏览"));
    browser_header.add_css_class("ngh-label-primary");
    browser_header.set_halign(Align::Start);
    browser_header.set_hexpand(true);

    let back_button = Button::with_icon_name("go-previous");
    back_button.add_css_class("ngh-ghost-button");
    back_button.set_tooltip_text(Some("返回上级"));

    let refresh_button = Button::with_icon_name("view-refresh");
    refresh_button.add_css_class("ngh-ghost-button");
    refresh_button.set_tooltip_text(Some("刷新"));

    browser_header_row.append(&browser_header);
    browser_header_row.append(&back_button);
    browser_header_row.append(&refresh_button);

    let path_label = Label::new(Some("/"));
    path_label.add_css_class("ngh-label-secondary");
    path_label.set_halign(Align::Start);

    let file_placeholder = Label::new(Some("登录后浏览 NAS 文件"));
    file_placeholder.add_css_class("ngh-empty-state");
    file_placeholder.set_vexpand(true);
    file_placeholder.set_valign(Align::Center);

    let file_listbox = ListBox::new();
    file_listbox.add_css_class("ngh-list");
    file_listbox.set_selection_mode(SelectionMode::None);

    let file_scrolled = ScrolledWindow::new(None::<&Adjustment>, None::<&Adjustment>);
    file_scrolled.set_vexpand(true);
    file_scrolled.set_min_content_height(200);
    file_scrolled.set_child(Some(&file_listbox));
    file_scrolled.set_visible(false);

    browser_section.append(&browser_header_row);
    browser_section.append(&path_label);
    browser_section.append(&file_placeholder);
    browser_section.append(&file_scrolled);
    container.append(&browser_section);

    // 分割线
    let divider2 = Separator::new(Orientation::Horizontal);
    divider2.set_margin_start(theme::SPACING_S4);
    divider2.set_margin_end(theme::SPACING_S4);
    divider2.set_margin_bottom(theme::SPACING_S3);
    container.append(&divider2);

    // ===================================================================
    // 协议源管理区
    // ===================================================================
    let proto_section = Box::new(Orientation::Vertical, theme::SPACING_S2);
    proto_section.set_margin_start(theme::SPACING_S4);
    proto_section.set_margin_end(theme::SPACING_S4);
    proto_section.set_margin_bottom(theme::SPACING_S4);

    let proto_header = Label::new(Some("协议源管理"));
    proto_header.add_css_class("ngh-label-primary");
    proto_header.set_halign(Align::Start);

    let proto_listbox = ListBox::new();
    proto_listbox.add_css_class("ngh-list");
    proto_listbox.set_selection_mode(SelectionMode::None);

    let proto_placeholder = Label::new(Some("暂无协议源"));
    proto_placeholder.add_css_class("ngh-label-secondary");
    proto_placeholder.set_halign(Align::Start);

    proto_section.append(&proto_header);
    proto_section.append(&proto_placeholder);
    proto_section.append(&proto_listbox);
    container.append(&proto_section);

    // --- 加载文件列表 ---
    let load_files = {
        let sender = sender.clone();
        let path_label = path_label.clone();
        let file_placeholder = file_placeholder.clone();
        let file_scrolled = file_scrolled.clone();
        move |path: String| {
            file_placeholder.set_text("加载文件列表…");
            file_placeholder.set_visible(true);
            file_scrolled.set_visible(false);
            path_label.set_text(&path);
            let sender = sender.clone();
            std::thread::spawn(move || {
                let core = CoreService::instance();
                let result = core.feiniu_list_files(&path);
                let message = match result {
                    Ok(files) => NasMessage::FilesLoaded(files),
                    Err(e) => NasMessage::FilesError(format!("{e}")),
                };
                let _ = sender.send(message);
            });
        }
    };

    // --- 登录按钮 ---
    {
        let url_entry = url_entry.clone();
        let user_entry = user_entry.clone();
        let pass_entry = pass_entry.clone();
        let status_label = status_label.clone();
        let sender = sender.clone();
        login_button.connect_clicked(move |_| {
            let base_url = url_entry.text().trim().to_string();
            let username = user_entry.text().trim().to_string();
            let password = pass_entry.text().to_string();
            if base_url.is_empty() || username.is_empty() {
                status_label.set_text("请填写服务地址与用户名");
                return;
            }
            status_label.set_text("登录中…");
            let sender = sender.clone();
            std::thread::spawn(move || {
                let core = CoreService::instance();
                let result = core.feiniu_login(&base_url, &username, &password);
                let message = match result {
                    Ok((token, base)) => {
                        let _ = token;
                        NasMessage::LoginOk(base)
                    }
                    Err(e) => NasMessage::LoginError(format!("{e}")),
                };
                let _ = sender.send(message);
            });
        });
    }

    // --- 健康检查按钮 ---
    {
        let sender = sender.clone();
        let status_label = status_label.clone();
        health_button.connect_clicked(move |_| {
            status_label.set_text("健康检查中…");
            let sender = sender.clone();
            std::thread::spawn(move || {
                let core = CoreService::instance();
                let result = core.feiniu_health();
                let message = match result {
                    Ok(()) => NasMessage::HealthOk,
                    Err(e) => NasMessage::HealthError(format!("{e}")),
                };
                let _ = sender.send(message);
            });
        });
    }

    // --- 返回上级 ---
    {
        let current_path = Rc::clone(&current_path);
        let load_files = load_files.clone();
        back_button.connect_clicked(move |_| {
            let path = current_path.borrow().clone();
            let parent = parent_path(&path);
            *current_path.borrow_mut() = parent.clone();
            load_files(parent);
        });
    }

    // --- 刷新文件列表 ---
    {
        let current_path = Rc::clone(&current_path);
        let load_files = load_files.clone();
        refresh_button.connect_clicked(move |_| {
            let path = current_path.borrow().clone();
            load_files(path);
        });
    }

    // --- 文件行激活：目录进入 / 文件流式播放 ---
    {
        let current_path = Rc::clone(&current_path);
        let sender = sender.clone();
        file_listbox.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx < 0 {
                return;
            }
            // 从行数据中取出文件名与是否目录
            let row_widget = match row.child() {
                Some(w) => w,
                None => return,
            };
            let row_box = match row_widget.downcast::<Box>() {
                Ok(b) => b,
                Err(_) => return,
            };
            // 从行的 name 属性取出文件名，从是否含 folder 图标判断目录
            let file_name = row_box
                .first_child()
                .and_then(|c| c.next_sibling())
                .and_then(|w| w.downcast::<Label>().ok())
                .map(|l| l.text().to_string())
                .unwrap_or_default();
            let is_dir = row_box
                .first_child()
                .and_then(|w| w.downcast::<Image>().ok())
                .map(|img| {
                    img.icon_name()
                        .map(|n| n.as_str() == "folder")
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if is_dir {
                // 进入目录
                let base = current_path.borrow().clone();
                let new_path = join_path(&base, &file_name);
                *current_path.borrow_mut() = new_path.clone();
                let sender = sender.clone();
                std::thread::spawn(move || {
                    let core = CoreService::instance();
                    let result = core.feiniu_list_files(&new_path);
                    let message = match result {
                        Ok(files) => NasMessage::FilesLoaded(files),
                        Err(e) => NasMessage::FilesError(format!("{e}")),
                    };
                    let _ = sender.send(message);
                });
            } else {
                // 流式播放
                let base = current_path.borrow().clone();
                let full_path = join_path(&base, &file_name);
                let sender = sender.clone();
                std::thread::spawn(move || {
                    let core = CoreService::instance();
                    let result = core.feiniu_stream(&full_path);
                    let message = match result {
                        Ok(url) => NasMessage::StreamOk(full_path, url),
                        Err(e) => NasMessage::StreamError(format!("{e}")),
                    };
                    let _ = sender.send(message);
                });
            }
        });
    }

    // --- 接收 NAS 异步消息 ---
    {
        let status_label = status_label.clone();
        let file_placeholder = file_placeholder.clone();
        let file_scrolled = file_scrolled.clone();
        let file_listbox = file_listbox.clone();
        let player = Arc::clone(&player);
        receiver.attach(None, move |msg| {
            match msg {
                NasMessage::LoginOk(base) => {
                    status_label.set_text(&format!("登录成功：{base}"));
                }
                NasMessage::LoginError(err) => {
                    status_label.set_text(&format!("登录失败：{err}"));
                }
                NasMessage::FilesLoaded(files) => {
                    if files.is_empty() {
                        file_placeholder.set_text("该目录为空");
                        file_placeholder.set_visible(true);
                        file_scrolled.set_visible(false);
                    } else {
                        file_placeholder.set_visible(false);
                        file_scrolled.set_visible(true);
                        rebuild_file_list(&file_listbox, &files);
                    }
                }
                NasMessage::FilesError(err) => {
                    file_placeholder.set_text(&format!("加载失败：{err}"));
                    file_placeholder.set_visible(true);
                    file_scrolled.set_visible(false);
                }
                NasMessage::StreamOk(path, url) => {
                    // 构造 Song 并加载播放
                    let name = std::path::Path::new(&path)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| path.clone());
                    let song = Song {
                        id: path.clone(),
                        source_id: "feiniu".to_string(),
                        title: name,
                        artists: Vec::new(),
                        album: None,
                        cover_url: None,
                        duration_ms: None,
                        lyric_url: None,
                        play_url: Some(url),
                        local_path: None,
                        origin: SongOrigin::Nas {
                            protocol: "feiniu".to_string(),
                            url: path,
                        },
                    };
                    player.load_queue(vec![song], 0);
                }
                NasMessage::StreamError(err) => {
                    status_label.set_text(&format!("播放失败：{err}"));
                }
                NasMessage::HealthOk => {
                    status_label.set_text("健康检查通过");
                }
                NasMessage::HealthError(err) => {
                    status_label.set_text(&format!("健康检查失败：{err}"));
                }
            }
            glib::ControlFlow::Continue
        });
    }

    // --- 协议源列表 ---
    let proto_listbox_clone = proto_listbox.clone();
    let proto_placeholder = proto_placeholder.clone();
    let refresh_proto = move || {
        let protos: Vec<ProtocolSourceInfo> = CoreService::instance().protocol_list();
        // 清空旧内容
        while let Some(child) = proto_listbox_clone.first_child() {
            proto_listbox_clone.remove(&child);
        }
        if protos.is_empty() {
            proto_placeholder.set_visible(true);
        } else {
            proto_placeholder.set_visible(false);
            for proto in protos {
                let row = create_protocol_row(&proto);
                proto_listbox_clone.append(&row);
            }
        }
    };
    refresh_proto();

    // --- 协议源删除 ---
    {
        let proto_listbox = proto_listbox.clone();
        let proto_placeholder = proto_placeholder.clone();
        proto_listbox.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx < 0 {
                return;
            }
            let protos = CoreService::instance().protocol_list();
            if let Some(proto) = protos.get(idx as usize) {
                let id = proto.id.clone();
                let deleted = CoreService::instance().protocol_delete(&id);
                if deleted {
                    // 刷新列表
                    let protos = CoreService::instance().protocol_list();
                    while let Some(child) = proto_listbox.first_child() {
                        proto_listbox.remove(&child);
                    }
                    if protos.is_empty() {
                        proto_placeholder.set_visible(true);
                    } else {
                        proto_placeholder.set_visible(false);
                        for proto in protos {
                            let row = create_protocol_row(&proto);
                            proto_listbox.append(&row);
                        }
                    }
                }
            }
        });
    }

    container.upcast::<Widget>()
}

/// 重建 NAS 文件列表：清空旧行后逐条追加。
fn rebuild_file_list(listbox: &ListBox, files: &[NasFile]) {
    while let Some(child) = listbox.first_child() {
        listbox.remove(&child);
    }
    for file in files {
        let row = create_file_row(file);
        listbox.append(&row);
    }
}

/// 创建单条文件行（图标 + 名称 + 大小 + 修改时间）。
fn create_file_row(file: &NasFile) -> ListBoxRow {
    let row_box = Box::new(Orientation::Horizontal, theme::SPACING_S3);
    row_box.add_css_class("ngh-song-row");

    let icon = if file.is_dir {
        Image::from_icon_name("folder")
    } else {
        Image::from_icon_name("audio-x-generic")
    };
    icon.set_pixel_size(18);
    icon.set_valign(Align::Center);

    let name_label = Label::new(Some(&file.name));
    name_label.add_css_class("ngh-song-title");
    name_label.set_halign(Align::Start);
    name_label.set_hexpand(true);
    name_label.set_ellipsize(EllipsizeMode::End);

    let size_label = Label::new(Some(&format_size(file.size)));
    size_label.add_css_class("ngh-song-duration");
    size_label.set_valign(Align::Center);

    let time_label = Label::new(Some(file.modified.as_deref().unwrap_or("")));
    time_label.add_css_class("ngh-label-secondary");
    time_label.set_valign(Align::Center);

    row_box.append(&icon);
    row_box.append(&name_label);
    row_box.append(&size_label);
    row_box.append(&time_label);

    let row = ListBoxRow::new();
    row.set_child(Some(&row_box));
    row.set_focusable(false);
    row.set_activatable(true);
    row
}

/// 创建协议源行（协议类型 + 根路径 + 删除提示）。
fn create_protocol_row(proto: &ProtocolSourceInfo) -> ListBoxRow {
    let row_box = Box::new(Orientation::Horizontal, theme::SPACING_S3);
    row_box.add_css_class("ngh-song-row");

    let icon = Image::from_icon_name("network-server");
    icon.set_pixel_size(18);
    icon.set_valign(Align::Center);

    let info = Box::new(Orientation::Vertical, 2);
    info.set_hexpand(true);
    info.set_halign(Align::Start);
    let proto_label = Label::new(Some(&format!(
        "{}{}",
        proto.protocol,
        if proto.placeholder { "（占位）" } else { "" }
    )));
    proto_label.add_css_class("ngh-song-title");
    proto_label.set_halign(Align::Start);
    let root_label = Label::new(Some(&proto.root));
    root_label.add_css_class("ngh-song-artist");
    root_label.set_halign(Align::Start);
    root_label.set_ellipsize(EllipsizeMode::End);
    info.append(&proto_label);
    info.append(&root_label);

    let delete_icon = Image::from_icon_name("edit-delete");
    delete_icon.set_pixel_size(16);
    delete_icon.set_valign(Align::Center);

    row_box.append(&icon);
    row_box.append(&info);
    row_box.append(&delete_icon);

    let row = ListBoxRow::new();
    row.set_child(Some(&row_box));
    row.set_focusable(false);
    row.set_activatable(true);
    row.set_tooltip_text(Some("点击删除该协议源"));
    row
}

/// 拼接路径：确保路径间有且仅有一个分隔符。
fn join_path(base: &str, name: &str) -> String {
    let base = base.trim_end_matches('/');
    if base.is_empty() {
        format!("/{name}")
    } else {
        format!("{base}/{name}")
    }
}

/// 获取父级路径。
fn parent_path(path: &str) -> String {
    let trimmed = path.trim_end_matches('/');
    match trimmed.rfind('/') {
        Some(0) => "/".to_string(),
        Some(idx) => trimmed[..idx].to_string(),
        None => "/".to_string(),
    }
}

/// 格式化文件大小为人类可读字符串。
fn format_size(size: u64) -> String {
    if size == 0 {
        return "—".to_string();
    }
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if size >= GB {
        format!("{:.1} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.1} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.1} KB", size as f64 / KB as f64)
    } else {
        format!("{size} B")
    }
}
