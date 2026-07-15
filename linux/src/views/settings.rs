//! 设置页视图。
//!
//! 提供音源管理、协议源管理、缓存管理与关于信息，与 macOS 端对齐：
//! 支持导入音源 JSON、启停/删除音源、删除协议源、查看与清空缓存、显示应用信息。
//!
//! 设计要点：
//! - 整体使用 `ScrolledWindow` 包裹纵向 `Box`，各分区以卡片样式呈现。
//! - 音源导入通过 `FileChooserDialog` 选择 `.json` 文件，读取后调用 `source_import`。
//! - 缓存统计来自 `CoreService::cache_stats()`，支持手动刷新与清空。

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::*;
use music_core::sources::SourceInfo;

use crate::core_service::{CoreService, ProtocolSourceInfo};
use crate::theme;
use crate::utils::{create_proto_row, format_size};

/// 创建设置页组件。
///
/// 布局自上而下：页面标题、音源管理区、协议源管理区、缓存管理区、关于区。
/// 整体包裹在 `ScrolledWindow` 中以支持内容超出视口时滚动。
pub fn create_settings_page() -> gtk4::Widget {
    let scrolled = ScrolledWindow::new(None::<&Adjustment>, None::<&Adjustment>);
    scrolled.set_vexpand(true);

    let container = Box::new(Orientation::Vertical, theme::SPACING_S6);
    container.set_margin_start(theme::SPACING_S4);
    container.set_margin_end(theme::SPACING_S4);
    container.set_margin_top(theme::SPACING_S4);
    container.set_margin_bottom(theme::SPACING_S4);

    // --- 页面标题 ---
    let title = Label::new(Some("设置"));
    title.add_css_class("ngh-page-title");
    title.set_halign(Align::Start);
    title.set_margin_bottom(theme::SPACING_S4);
    container.append(&title);

    // ===================================================================
    // 区块 1：音源管理
    // ===================================================================
    let source_section = build_source_section();
    container.append(&source_section);

    // ===================================================================
    // 区块 2：协议源管理
    // ===================================================================
    let proto_section = build_protocol_section();
    container.append(&proto_section);

    // ===================================================================
    // 区块 3：缓存管理
    // ===================================================================
    let cache_section = build_cache_section();
    container.append(&cache_section);

    // ===================================================================
    // 区块 4：关于
    // ===================================================================
    let about_section = build_about_section();
    container.append(&about_section);

    scrolled.set_child(Some(&container));
    scrolled.upcast::<Widget>()
}

/// 构建音源管理区：导入按钮 + 音源列表（名称 + 启停开关 + 删除按钮）。
fn build_source_section() -> Widget {
    let section = Box::new(Orientation::Vertical, theme::SPACING_S2);
    section.add_css_class("ngh-card");

    let header = Label::new(Some("音源管理"));
    header.add_css_class("ngh-label-primary");
    header.set_halign(Align::Start);

    let toolbar = Box::new(Orientation::Horizontal, theme::SPACING_S2);
    let import_button = Button::with_label("导入音源");
    import_button.add_css_class("ngh-primary-button");
    let refresh_button = Button::with_icon_name("view-refresh");
    refresh_button.add_css_class("ngh-ghost-button");
    refresh_button.set_tooltip_text(Some("刷新音源列表"));
    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    toolbar.append(&import_button);
    toolbar.append(&refresh_button);
    toolbar.append(&spacer);

    let listbox = ListBox::new();
    listbox.add_css_class("ngh-list");
    listbox.set_selection_mode(SelectionMode::None);

    let placeholder = Label::new(Some("暂无音源，点击「导入音源」加载 JSON 配置"));
    placeholder.add_css_class("ngh-empty-state");

    // 使用 Rc<RefCell<Option<...>>> 打破循环依赖：refresh_sources 在构造每行时
    // 需把自身克隆传给 create_source_row 的删除按钮，但此时闭包尚未构造完成。
    let refresh_cell: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));

    // 刷新音源列表的闭包
    let refresh_sources: Rc<dyn Fn()> = Rc::new({
        let listbox = listbox.clone();
        let placeholder = placeholder.clone();
        let refresh_cell = Rc::clone(&refresh_cell);
        move || {
            let sources: Vec<SourceInfo> = CoreService::instance().source_list();
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
            if sources.is_empty() {
                placeholder.set_visible(true);
            } else {
                placeholder.set_visible(false);
                let cb = refresh_cell.borrow().clone();
                for source in sources {
                    let row = create_source_row(&source, cb.clone());
                    listbox.append(&row);
                }
            }
        }
    });
    *refresh_cell.borrow_mut() = Some(Rc::clone(&refresh_sources));

    refresh_sources();

    // --- 导入按钮 ---
    {
        let refresh_sources = refresh_sources.clone();
        import_button.connect_clicked(move |_| {
            let dialog = FileChooserDialog::new(
                Some("选择音源 JSON 文件"),
                None::<&Window>,
                FileChooserAction::Open,
                &[
                    ("取消", ResponseType::Cancel),
                    ("导入", ResponseType::Accept),
                ],
            );
            // 添加 .json 过滤器
            let filter = FileFilter::new();
            filter.set_name(Some("JSON 文件"));
            filter.add_pattern("*.json");
            filter.add_mime_type("application/json");
            dialog.add_filter(&filter);

            let refresh_sources = refresh_sources.clone();
            dialog.connect_response(move |d, response| {
                if response == ResponseType::Accept {
                    if let Some(file) = d.file() {
                        if let Some(path) = file.path() {
                            let path_str = path.to_string_lossy().to_string();
                            let json = match std::fs::read_to_string(&path_str) {
                                Ok(content) => content,
                                Err(e) => {
                                    log::warn!("读取音源文件失败：{e}");
                                    d.close();
                                    return;
                                }
                            };
                            let core = CoreService::instance();
                            match core.source_import(&json) {
                                Ok(info) => {
                                    log::info!("音源导入成功：{}", info.name);
                                    refresh_sources();
                                }
                                Err(e) => {
                                    log::warn!("音源导入失败：{e}");
                                }
                            }
                        }
                    }
                }
                d.close();
            });
            dialog.show();
        });
    }

    // --- 刷新按钮 ---
    {
        refresh_button.connect_clicked(move |_| {
            refresh_sources();
        });
    }

    section.append(&header);
    section.append(&toolbar);
    section.append(&placeholder);
    section.append(&listbox);
    section.upcast::<Widget>()
}

/// 构建协议源管理区：协议源列表 + 删除按钮。
fn build_protocol_section() -> Widget {
    let section = Box::new(Orientation::Vertical, theme::SPACING_S2);
    section.add_css_class("ngh-card");

    let header = Label::new(Some("协议源管理"));
    header.add_css_class("ngh-label-primary");
    header.set_halign(Align::Start);

    let listbox = ListBox::new();
    listbox.add_css_class("ngh-list");
    listbox.set_selection_mode(SelectionMode::None);

    let placeholder = Label::new(Some("暂无协议源"));
    placeholder.add_css_class("ngh-empty-state");

    let refresh_protos = {
        let listbox = listbox.clone();
        let placeholder = placeholder.clone();
        move || {
            let protos: Vec<ProtocolSourceInfo> = CoreService::instance().protocol_list();
            while let Some(child) = listbox.first_child() {
                listbox.remove(&child);
            }
            if protos.is_empty() {
                placeholder.set_visible(true);
            } else {
                placeholder.set_visible(false);
                for proto in protos {
                    let row = create_proto_row(&proto);
                    listbox.append(&row);
                }
            }
        }
    };

    refresh_protos();

    // 点击协议源行删除
    {
        let listbox_clone = listbox.clone();
        let placeholder = placeholder.clone();
        listbox.connect_row_activated(move |_, row| {
            let idx = row.index();
            if idx < 0 {
                return;
            }
            let protos = CoreService::instance().protocol_list();
            if let Some(proto) = protos.get(idx as usize) {
                let id = proto.id.clone();
                if CoreService::instance().protocol_delete(&id) {
                    let protos = CoreService::instance().protocol_list();
                    while let Some(child) = listbox_clone.first_child() {
                        listbox_clone.remove(&child);
                    }
                    if protos.is_empty() {
                        placeholder.set_visible(true);
                    } else {
                        placeholder.set_visible(false);
                        for proto in protos {
                            let row = create_proto_row(&proto);
                            listbox_clone.append(&row);
                        }
                    }
                }
            }
        });
    }

    section.append(&header);
    section.append(&placeholder);
    section.append(&listbox);
    section.upcast::<Widget>()
}

/// 构建缓存管理区：缓存统计 + 清空按钮。
fn build_cache_section() -> Widget {
    let section = Box::new(Orientation::Vertical, theme::SPACING_S2);
    section.add_css_class("ngh-card");

    let header = Label::new(Some("缓存管理"));
    header.add_css_class("ngh-label-primary");
    header.set_halign(Align::Start);

    let stats_label = Label::new(Some("条目数：—  已用：—  上限：—"));
    stats_label.add_css_class("ngh-label-secondary");
    stats_label.set_halign(Align::Start);

    let toolbar = Box::new(Orientation::Horizontal, theme::SPACING_S2);
    let clear_button = Button::with_label("清空缓存");
    clear_button.add_css_class("ngh-ghost-button");
    let refresh_button = Button::with_label("刷新统计");
    refresh_button.add_css_class("ngh-ghost-button");
    let spacer = Box::new(Orientation::Horizontal, 0);
    spacer.set_hexpand(true);
    toolbar.append(&spacer);
    toolbar.append(&refresh_button);
    toolbar.append(&clear_button);

    // 更新缓存统计
    let update_stats = {
        let stats_label = stats_label.clone();
        move || {
            let (entries, total, max) = CoreService::instance().cache_stats();
            stats_label.set_text(&format!(
                "条目数：{}  已用：{}  上限：{}",
                entries,
                format_size(total),
                format_size(max)
            ));
        }
    };

    update_stats();

    // --- 刷新统计按钮 ---
    {
        let update_stats = update_stats.clone();
        refresh_button.connect_clicked(move |_| {
            update_stats();
        });
    }

    // --- 清空缓存按钮 ---
    {
        let update_stats = update_stats.clone();
        clear_button.connect_clicked(move |_| {
            let _ = CoreService::instance().cache_clear();
            update_stats();
        });
    }

    section.append(&header);
    section.append(&stats_label);
    section.append(&toolbar);
    section.upcast::<Widget>()
}

/// 构建关于区：应用名称、版本、平台。
fn build_about_section() -> Widget {
    let section = Box::new(Orientation::Vertical, theme::SPACING_S2);
    section.add_css_class("ngh-card");

    let header = Label::new(Some("关于"));
    header.add_css_class("ngh-label-primary");
    header.set_halign(Align::Start);

    let name_label = Label::new(Some("逆光音乐"));
    name_label.add_css_class("ngh-song-title");
    name_label.set_halign(Align::Start);

    let version_label = Label::new(Some("版本：0.1.0"));
    version_label.add_css_class("ngh-label-secondary");
    version_label.set_halign(Align::Start);

    let platform_label = Label::new(Some("平台：Linux"));
    platform_label.add_css_class("ngh-label-secondary");
    platform_label.set_halign(Align::Start);

    section.append(&header);
    section.append(&name_label);
    section.append(&version_label);
    section.append(&platform_label);
    section.upcast::<Widget>()
}

/// 创建音源行（名称 + 类型标签 + 启停开关 + 删除按钮）。
///
/// # 参数
/// - `on_changed`：删除音源后触发的刷新回调（重建音源列表）。
fn create_source_row(source: &SourceInfo, on_changed: Option<Rc<dyn Fn()>>) -> ListBoxRow {
    let row_box = Box::new(Orientation::Horizontal, theme::SPACING_S3);
    row_box.add_css_class("ngh-song-row");

    let info = Box::new(Orientation::Vertical, 2);
    info.set_hexpand(true);
    info.set_halign(Align::Start);
    let name_label = Label::new(Some(&source.name));
    name_label.add_css_class("ngh-song-title");
    name_label.set_halign(Align::Start);
    let desc = source
        .description
        .as_deref()
        .unwrap_or(&source.source_type);
    let meta_label = Label::new(Some(&format!(
        "v{} · {} · 优先级 {}",
        source.version, desc, source.priority
    )));
    meta_label.add_css_class("ngh-song-artist");
    meta_label.set_halign(Align::Start);
    meta_label.set_ellipsize(EllipsizeMode::End);
    info.append(&name_label);
    info.append(&meta_label);

    // 启停开关
    let switch = Switch::new();
    switch.set_active(source.enabled);
    switch.set_valign(Align::Center);
    {
        let id = source.id.clone();
        let switch = switch.clone();
        switch.connect_state_set(move |_, state| {
            let core = CoreService::instance();
            let result = if state {
                core.source_enable(&id)
            } else {
                core.source_disable(&id)
            };
            if let Err(e) = result {
                log::warn!("切换音源状态失败：{e}");
                // 还原开关状态
                switch.set_active(!state);
            }
            glib::Propagation::Stop
        });
    }

    // 删除按钮：删除成功后调用 on_changed 刷新音源列表
    let delete_button = Button::with_icon_name("edit-delete");
    delete_button.add_css_class("ngh-ghost-button");
    delete_button.set_tooltip_text(Some("删除该音源"));
    delete_button.set_valign(Align::Center);
    {
        let id = source.id.clone();
        delete_button.connect_clicked(move |_| {
            match CoreService::instance().source_delete(&id) {
                Ok(()) => {
                    if let Some(cb) = &on_changed {
                        cb();
                    }
                }
                Err(e) => log::warn!("删除音源失败：{e}"),
            }
        });
    }

    row_box.append(&info);
    row_box.append(&switch);
    row_box.append(&delete_button);

    let row = ListBoxRow::new();
    row.set_child(Some(&row_box));
    row.set_focusable(false);
    row.set_activatable(false);
    row
}
