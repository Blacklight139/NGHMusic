//! 侧边栏导航视图。
//!
//! 提供品牌头部、功能页导航列表与底部信息条，与 macOS 端 `ContentView.swift`
//! 中的侧边栏对齐。导航项点击时通过回调通知父组件切换页面。
//!
//! 设计要点：
//! - 使用 GTK Symbolic Icons（禁用 emoji），图标名见 [`NAV_ITEMS`]。
//! - 导航行采用线性列表风格（`gtk4::ListBox`），选中态由 CSS `.ngh-sidebar-item.active` 控制。
//! - 间距与圆角统一使用 `theme` 模块的 Token（`SPACING_*` / `RADIUS_*`）。

use gtk4::prelude::*;

use crate::theme;

/// 侧边栏可切换的功能页标识。
///
/// 与 macOS 端 `SidebarPage` 枚举一一对应，由 [`create_sidebar`] 在选中项变化时
/// 通过回调传递给父组件，用于驱动详情区页面切换。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    /// 搜索页。
    Search,
    /// 播放列表页。
    Playlist,
    /// 收藏页。
    Favorites,
    /// 歌词页。
    Lyrics,
    /// 排行榜页。
    Leaderboard,
    /// 本地音乐页。
    LocalMusic,
    /// NAS 页。
    Nas,
    /// 设置页。
    Settings,
}

/// 导航项描述：页面 ID、显示名称、GTK 图标名。
struct NavItem {
    /// 对应的功能页标识。
    id: PageId,
    /// 中文显示名称。
    title: &'static str,
    /// GTK Symbolic Icon 名称。
    icon: &'static str,
}

/// 全部导航项，顺序即侧边栏从上到下的展示顺序。
const NAV_ITEMS: &[NavItem] = &[
    NavItem {
        id: PageId::Search,
        title: "搜索",
        icon: "system-search",
    },
    NavItem {
        id: PageId::Playlist,
        title: "播放列表",
        icon: "view-list",
    },
    NavItem {
        id: PageId::Favorites,
        title: "收藏",
        icon: "emblem-favorite",
    },
    NavItem {
        id: PageId::Lyrics,
        title: "歌词",
        icon: "format-justify-left",
    },
    NavItem {
        id: PageId::Leaderboard,
        title: "排行榜",
        icon: "office-calendar",
    },
    NavItem {
        id: PageId::LocalMusic,
        title: "本地音乐",
        icon: "folder-music",
    },
    NavItem {
        id: PageId::Nas,
        title: "NAS",
        icon: "network-server",
    },
    NavItem {
        id: PageId::Settings,
        title: "设置",
        icon: "preferences-system",
    },
];

/// 创建侧边栏组件。
///
/// 布局自上而下依次为：品牌头部（音符图标 + 「逆光音乐」）、分割线、
/// 功能页导航列表（[`NAV_ITEMS`]）、分割线、底部信息条（「NGHMusic · Linux」）。
///
/// # 参数
/// - `selection_changed`：当用户选中某个导航项时触发的回调，传入对应的 [`PageId`]。
///
/// # 返回
/// 装配完成的 `gtk4::Widget`（纵向 `Box`），可直接放入 `PanedWindow` / `OverlaySplitView`。
pub fn create_sidebar(selection_changed: impl Fn(PageId) + 'static) -> gtk4::Widget {
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    container.add_css_class("ngh-sidebar");

    // --- 品牌头部 ---
    let brand = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S2);
    brand.set_margin_start(theme::SPACING_S4);
    brand.set_margin_end(theme::SPACING_S4);
    brand.set_margin_top(14);
    brand.set_margin_bottom(14);

    let brand_icon = gtk4::Image::from_icon_name("audio-x-generic");
    brand_icon.set_pixel_size(24);

    let brand_label = gtk4::Label::new(Some("逆光音乐"));
    brand_label.add_css_class("ngh-brand-label");
    brand_label.set_halign(gtk4::Align::Start);
    brand_label.set_hexpand(true);

    brand.append(&brand_icon);
    brand.append(&brand_label);
    container.append(&brand);

    // 品牌区与导航区之间的分割线
    let top_divider = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    container.append(&top_divider);

    // --- 导航列表 ---
    let nav = gtk4::ListBox::new();
    nav.set_selection_mode(gtk4::SelectionMode::Single);
    nav.add_css_class("ngh-list");
    nav.set_margin_start(theme::SPACING_S2);
    nav.set_margin_end(theme::SPACING_S2);
    nav.set_margin_top(theme::SPACING_S2);
    nav.set_hexpand(true);
    nav.set_vexpand(true);

    for item in NAV_ITEMS {
        let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S3);
        row_box.add_css_class("ngh-sidebar-item");

        let icon = gtk4::Image::from_icon_name(item.icon);
        icon.set_pixel_size(18);

        let label = gtk4::Label::new(Some(item.title));
        label.set_halign(gtk4::Align::Start);

        row_box.append(&icon);
        row_box.append(&label);

        let row = gtk4::ListBoxRow::new();
        row.set_child(Some(&row_box));
        row.set_focusable(false);
        nav.append(&row);
    }

    // 选中项变化时通知父组件
    nav.connect_selected_rows_changed(move |listbox| {
        if let Some(row) = listbox.selected_row() {
            let idx = row.index();
            if idx >= 0 {
                let idx = idx as usize;
                if idx < NAV_ITEMS.len() {
                    selection_changed(NAV_ITEMS[idx].id);
                }
            }
        }
    });

    container.append(&nav);

    // --- 底部信息条 ---
    let bottom_divider = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    container.append(&bottom_divider);

    let footer = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S2);
    footer.set_margin_start(theme::SPACING_S4);
    footer.set_margin_end(theme::SPACING_S4);
    footer.set_margin_top(10);
    footer.set_margin_bottom(10);

    let footer_icon = gtk4::Image::from_icon_name("audio-input-microphone");
    footer_icon.set_pixel_size(16);

    let footer_label = gtk4::Label::new(Some("NGHMusic · Linux"));
    footer_label.add_css_class("ngh-label-secondary");

    footer.append(&footer_icon);
    footer.append(&footer_label);
    container.append(&footer);

    // 默认选中第一项（搜索页）
    if let Some(first_row) = nav.row_at_index(0) {
        nav.select_row(Some(&first_row));
    }

    container.upcast::<gtk4::Widget>()
}
