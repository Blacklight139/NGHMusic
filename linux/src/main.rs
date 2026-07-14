//! NGHMusic Linux 客户端入口。
//!
//! 基于 GTK4 + libadwaita + GStreamer + Rust，直接依赖共享核心 `music-core`。
//! 应用窗口采用侧边栏 + 详情区 + 底部播放栏的三段式布局，
//! 支持搜索/播放列表/收藏/歌词/排行榜/本地音乐/NAS/设置八个功能页。

use std::sync::Arc;

use gtk4::prelude::*;
use gtk4::{gdk, Application, ApplicationWindow, CssProvider, Orientation, Paned, StyleContext};

mod core_service;
mod player_service;
mod theme;
mod views;

use player_service::PlayerService;
use views::PageId;
use views::sidebar::create_sidebar;

/// 应用 ID。
const APP_ID: &str = "com.nghmusic.linux";

fn main() {
    // 初始化 GStreamer（必须在创建 GTK Application 之前）。
    gstreamer::init().expect("初始化 GStreamer 失败");

    // 初始化 GTK4 + libadwaita。
    gtk4::init().expect("初始化 GTK4 失败");
    libadwaita::init().expect("初始化 libadwaita 失败");

    // 加载豆包风格 CSS 主题。
    let provider = CssProvider::new();
    theme::apply_theme(&provider);
    if let Some(display) = gdk::Display::default() {
        StyleContext::add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    // 创建 GTK Application。
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run();
}

/// 构建 GTK4 主窗口。
fn build_ui(app: &Application) {
    // 初始化播放器服务。
    let player = Arc::new(PlayerService::new());

    // 创建 GtkStack 用于详情区页面切换。
    let stack = gtk4::Stack::new();
    let player_for_pages = Arc::clone(&player);

    // 搜索页
    let search_page = views::search::create_search_page(Arc::clone(&player_for_pages));
    stack.add_titled(&search_page, Some("search"), "搜索");

    // 播放列表页
    let playlist_page = views::playlist::create_playlist_page(Arc::clone(&player_for_pages));
    stack.add_titled(&playlist_page, Some("playlist"), "播放列表");

    // 收藏页
    let favorites_page = views::favorites::create_favorites_page(Arc::clone(&player_for_pages));
    stack.add_titled(&favorites_page, Some("favorites"), "收藏");

    // 歌词页
    let lyrics_page = views::lyrics::create_lyrics_page(Arc::clone(&player_for_pages));
    stack.add_titled(&lyrics_page, Some("lyrics"), "歌词");

    // 排行榜页
    let leaderboard_page = views::leaderboard::create_leaderboard_page(Arc::clone(&player_for_pages));
    stack.add_titled(&leaderboard_page, Some("leaderboard"), "排行榜");

    // 本地音乐页
    let local_music_page = views::local_music::create_local_music_page(Arc::clone(&player_for_pages));
    stack.add_titled(&local_music_page, Some("local-music"), "本地音乐");

    // NAS 页
    let nas_page = views::nas::create_nas_page(Arc::clone(&player_for_pages));
    stack.add_titled(&nas_page, Some("nas"), "NAS");

    // 设置页
    let settings_page = views::settings::create_settings_page();
    stack.add_titled(&settings_page, Some("settings"), "设置");

    // 默认显示搜索页
    stack.set_visible_child_name("search");

    // 创建侧边栏，连接页面切换回调。
    let stack_clone = stack.clone();
    let sidebar = create_sidebar(move |page_id| {
        let name = match page_id {
            PageId::Search => "search",
            PageId::Playlist => "playlist",
            PageId::Favorites => "favorites",
            PageId::Lyrics => "lyrics",
            PageId::Leaderboard => "leaderboard",
            PageId::LocalMusic => "local-music",
            PageId::Nas => "nas",
            PageId::Settings => "settings",
        };
        stack_clone.set_visible_child_name(name);
    });

    // 详情区容器：Stack + 分割线 + 播放控制栏
    let detail_box = gtk4::Box::new(Orientation::Vertical, 0);
    detail_box.append(&stack);
    detail_box.append(&gtk4::Separator::new(gtk4::Orientation::Horizontal));

    // 底部播放控制栏
    let playback_bar = views::playback_bar::create_playback_bar(Arc::clone(&player));
    playback_bar.set_size_request(-1, 72);
    detail_box.append(&playback_bar);

    // 使用 Paned 布局：侧边栏 | 详情区
    let paned = Paned::new(gtk4::Orientation::Horizontal);
    paned.set_start_child(Some(&sidebar));
    paned.set_end_child(Some(&detail_box));
    paned.set_position(220);
    paned.set_shrink_start_child(false);
    paned.set_shrink_end_child(false);

    // 创建主窗口
    let window = ApplicationWindow::builder()
        .application(app)
        .title("逆光音乐")
        .default_width(1100)
        .default_height(720)
        .child(&paned)
        .build();

    window.present();
}
