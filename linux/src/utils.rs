//! 公共工具函数。
//!
//! 抽取各视图模块中重复的格式化辅助函数（艺术家列表、时长、文件大小）
//! 与协议源行构造，统一实现避免各模块各自维护副本导致行为不一致。

use gtk4::prelude::*;

use crate::core_service::ProtocolSourceInfo;
use crate::theme;

/// 格式化艺术家列表为「A / B / C」形式。
///
/// 空列表返回空字符串。
pub fn format_artists(artists: &[String]) -> String {
    artists.join(" / ")
}

/// 将毫秒时长格式化为 `mm:ss`。
///
/// `None` 返回 `"--:--"`。仅展示分与秒，不展示小时。
pub fn format_duration(duration_ms: Option<u64>) -> String {
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

/// 格式化字节数为人类可读字符串。
///
/// 统一使用 2 位小数精度，避免各模块精度不一致。
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

/// 创建协议源行（协议类型 + 根路径 + 删除提示）。
///
/// 供设置页与 NAS 页共享，避免两处重复构造。
pub fn create_proto_row(proto: &ProtocolSourceInfo) -> gtk4::ListBoxRow {
    let row_box = gtk4::Box::new(gtk4::Orientation::Horizontal, theme::SPACING_S3);
    row_box.add_css_class("ngh-song-row");

    let icon = gtk4::Image::from_icon_name("network-server");
    icon.set_pixel_size(18);
    icon.set_valign(gtk4::Align::Center);

    let info = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
    info.set_hexpand(true);
    info.set_halign(gtk4::Align::Start);
    let proto_label = gtk4::Label::new(Some(&format!(
        "{}{}",
        proto.protocol,
        if proto.placeholder { "（占位）" } else { "" }
    )));
    proto_label.add_css_class("ngh-song-title");
    proto_label.set_halign(gtk4::Align::Start);
    let root_label = gtk4::Label::new(Some(&proto.root));
    root_label.add_css_class("ngh-song-artist");
    root_label.set_halign(gtk4::Align::Start);
    root_label.set_ellipsize(gtk4::EllipsizeMode::End);
    info.append(&proto_label);
    info.append(&root_label);

    let delete_icon = gtk4::Image::from_icon_name("edit-delete");
    delete_icon.set_pixel_size(16);
    delete_icon.set_valign(gtk4::Align::Center);

    row_box.append(&icon);
    row_box.append(&info);
    row_box.append(&delete_icon);

    let row = gtk4::ListBoxRow::new();
    row.set_child(Some(&row_box));
    row.set_focusable(false);
    row.set_activatable(true);
    row.set_tooltip_text(Some("点击删除该协议源"));
    row
}
