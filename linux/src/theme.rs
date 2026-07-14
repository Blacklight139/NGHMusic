//! NGHMusic Linux 客户端主题模块。
//!
//! 定义与 macOS 端 `AppTheme.swift` 对齐的「豆包风格」设计 Token，
//! 涵盖颜色（`gdk::RGBA`）、圆角与间距，并提供加载 CSS 主题的能力。
//!
//! 所有颜色 Token 与 `resources/style.css` 中通过 `@define-color`
//! 声明的 CSS 变量一一对应，确保代码层与样式层视觉一致。

use std::sync::LazyLock;

use gtk4::{gdk, CssProvider};

// ===========================================================================
// 颜色设计 Token
// ===========================================================================

/// 主色：柔和蓝紫。
pub static PRIMARY: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#4E6EF2"));
/// 主色悬停态。
pub static PRIMARY_HOVER: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#3D5AE0"));
/// 主色软背景（8% 透明度）。
pub static PRIMARY_SOFT: LazyLock<gdk::RGBA> =
    LazyLock::new(|| rgba_from_hex_alpha("#4E6EF2", 0.08));

/// 应用背景色。
pub static BACKGROUND: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#F7F8FA"));
/// 表面色（卡片、弹层等）。
pub static SURFACE: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#FFFFFF"));
/// 次级表面色（hover 底色等）。
pub static SURFACE_ALT: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#F0F2F5"));
/// 侧边栏背景色。
pub static SIDEBAR_BACKGROUND: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#FFFFFF"));

/// 主文本色。
pub static TEXT: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#1F1F1F"));
/// 主要文本色（与 [`TEXT`] 一致）。
pub static TEXT_PRIMARY: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#1F1F1F"));
/// 次要文本色。
pub static TEXT_SECONDARY: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#6B6B6B"));
/// 三级文本色。
pub static TEXT_TERTIARY: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#999999"));

/// 边框色。
pub static BORDER: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#EDEEF0"));
/// 柔和边框色。
pub static BORDER_SOFT: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#F5F6F8"));

/// 危险 / 错误色。
pub static DANGER: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#F5483B"));
/// 成功色。
pub static SUCCESS: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#00B96B"));
/// 警告色。
pub static WARNING: LazyLock<gdk::RGBA> = LazyLock::new(|| rgba_from_hex("#FA8C16"));

// ===========================================================================
// 圆角 Token（单位：px）
// ===========================================================================

/// 小圆角。
pub const RADIUS_SM: i32 = 8;
/// 中圆角。
pub const RADIUS_MD: i32 = 12;
/// 大圆角。
pub const RADIUS_LG: i32 = 16;
/// 胶囊圆角。
pub const RADIUS_PILL: i32 = 999;

// ===========================================================================
// 间距 Token（单位：px）
// ===========================================================================

/// 间距 s1。
pub const SPACING_S1: i32 = 4;
/// 间距 s2。
pub const SPACING_S2: i32 = 8;
/// 间距 s3。
pub const SPACING_S3: i32 = 12;
/// 间距 s4。
pub const SPACING_S4: i32 = 16;
/// 间距 s5。
pub const SPACING_S5: i32 = 20;
/// 间距 s6。
pub const SPACING_S6: i32 = 24;
/// 间距 s7。
pub const SPACING_S7: i32 = 32;
/// 间距 s8。
pub const SPACING_S8: i32 = 40;

// ===========================================================================
// 辅助函数
// ===========================================================================

/// CSS 主题文件相对于包根目录的路径。
const CSS_PATH: &str = "resources/style.css";

/// 由十六进制颜色字符串构造不透明的 `gdk::RGBA`。
///
/// 支持 `#RRGGBB` 或 `RRGGBB` 格式，alpha 固定为 `1.0`。
///
/// # Panics
/// 当传入的字符串不是合法的 6 位十六进制颜色时触发 panic。
///
/// ```ignore
/// # use nghmusic_linux::theme::rgba_from_hex;
/// let c = rgba_from_hex("#4E6EF2");
/// // c.red() ≈ 78 / 255，c.green() ≈ 110 / 255，c.blue() ≈ 242 / 255
/// ```
pub fn rgba_from_hex(hex: &str) -> gdk::RGBA {
    rgba_from_hex_alpha(hex, 1.0)
}

/// 由十六进制颜色字符串构造带透明度的 `gdk::RGBA`。
///
/// 支持 `#RRGGBB` 或 `RRGGBB` 格式，`alpha` 取值范围为 `0.0..=1.0`。
///
/// # Panics
/// 当传入的字符串不是合法的 6 位十六进制颜色时触发 panic。
pub fn rgba_from_hex_alpha(hex: &str, alpha: f32) -> gdk::RGBA {
    let hex = hex.trim_start_matches('#');
    assert!(
        hex.len() == 6,
        "无效的十六进制颜色：期望 6 位 RRGGBB，实际得到 '{hex}'"
    );
    let r = u8::from_str_radix(&hex[0..2], 16).expect("无效的十六进制颜色（红色分量）");
    let g = u8::from_str_radix(&hex[2..4], 16).expect("无效的十六进制颜色（绿色分量）");
    let b = u8::from_str_radix(&hex[4..6], 16).expect("无效的十六进制颜色（蓝色分量）");
    gdk::RGBA::new(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        alpha,
    )
}

/// 加载「豆包风格」CSS 主题到指定的 [`CssProvider`]。
///
/// 从 `resources/style.css` 读取主题样式（其中通过 `@define-color`
/// 定义 CSS 变量），并加载到传入的 `provider` 中。调用方负责将该
/// provider 添加到目标 `gdk::Display`，例如：
///
/// ```ignore
/// use gtk4::{gdk, CssProvider, StyleContext};
///
/// let provider = CssProvider::new();
/// theme::apply_theme(&provider);
/// if let Some(display) = gdk::Display::default() {
///     StyleContext::add_provider_for_display(
///         &display,
///         &provider,
///         gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
///     );
/// }
/// ```
///
/// 若读取或解析失败，会通过 `log` 记录警告但不返回错误，以保证
/// 应用即便缺少主题文件也能继续启动。
pub fn apply_theme(provider: &CssProvider) {
    match provider.load_from_path(CSS_PATH) {
        Ok(()) => log::info!("已加载主题 CSS：{CSS_PATH}"),
        Err(e) => log::warn!("加载主题 CSS 失败（{CSS_PATH}）：{e}"),
    }
}
