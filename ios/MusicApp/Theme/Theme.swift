import SwiftUI

/// 简约风格设计系统。
///
/// 与桌面端（Tauri Web）保持视觉协调统一：深色背景（#1A1A1A） + 单一主色（紫罗兰）
/// + 中性灰阶 + 克制的圆角与间距。所有界面均通过 `Theme` 取色与尺寸，避免硬编码。
enum Theme {
    // MARK: - 色彩

    enum Palette {
        // 背景层
        static let background         = SwiftUI.Color(hex: 0x1A1A1A) // 主背景
        static let backgroundElevated = SwiftUI.Color(hex: 0x202020) // 顶栏/底栏背景
        static let surface            = SwiftUI.Color(hex: 0x262626) // 卡片
        static let surfaceHover       = SwiftUI.Color(hex: 0x2E2E2E) // 卡片悬浮/按下
        static let separator          = SwiftUI.Color(hex: 0x3A3A3A) // 分隔线

        // 主色调（紫罗兰），全 App 唯一强调色
        static let primary       = SwiftUI.Color(hex: 0xA78BFA)
        static let primaryActive = SwiftUI.Color(hex: 0xC4B5FD)
        static let primaryDim    = SwiftUI.Color(hex: 0xA78BFA).opacity(0.18)

        // 播放中指示色（仅用于“正在播放”动画/点）
        static let playing = SwiftUI.Color(hex: 0x34D399)

        // 文本层级
        static let textPrimary   = SwiftUI.Color(hex: 0xF5F5F5)
        static let textSecondary = SwiftUI.Color(hex: 0xA1A1A1)
        static let textTertiary   = SwiftUI.Color(hex: 0x6E6E6E)

        // 状态色
        static let error   = SwiftUI.Color(hex: 0xED5C5C)
        static let warning = SwiftUI.Color(hex: 0xF2C24A)
        static let success = SwiftUI.Color(hex: 0x34D399)
    }

    // MARK: - 间距

    enum Spacing {
        static let xs: CGFloat = 4
        static let s: CGFloat = 8
        static let m: CGFloat = 12
        static let l: CGFloat = 16
        static let xl: CGFloat = 24
        static let xxl: CGFloat = 32
    }

    // MARK: - 圆角

    enum Radius {
        static let s: CGFloat = 6
        static let m: CGFloat = 10
        static let l: CGFloat = 14
        static let xl: CGFloat = 20
        static let cover: CGFloat = 12
    }

    // MARK: - 字体

    enum Typography {
        static let largeTitle    = SwiftUI.Font.system(size: 28, weight: .bold)
        static let title         = SwiftUI.Font.system(size: 22, weight: .bold)
        static let title2        = SwiftUI.Font.system(size: 18, weight: .semibold)
        static let body          = SwiftUI.Font.system(size: 16, weight: .regular)
        static let bodyEmphasized = SwiftUI.Font.system(size: 16, weight: .semibold)
        static let caption        = SwiftUI.Font.system(size: 13, weight: .regular)
        static let small          = SwiftUI.Font.system(size: 12, weight: .regular)
        static let mono           = SwiftUI.Font.system(size: 13, weight: .regular, design: .monospaced)
    }
}

// MARK: - Color(hex:) 便捷构造

extension Color {
    /// 由 0xRRGGBB 整数构造颜色（默认不透明）。
    init(hex: UInt32, alpha: Double = 1.0) {
        let r = Double((hex >> 16) & 0xFF) / 255.0
        let g = Double((hex >> 8) & 0xFF) / 255.0
        let b = Double(hex & 0xFF) / 255.0
        self.init(.sRGB, red: r, green: g, blue: b, opacity: alpha)
    }
}

// MARK: - 通用视图修饰器

/// 卡片背景修饰器：统一卡片色、圆角、内边距。
struct CardModifier: ViewModifier {
    var padded: Bool = true

    func body(content: Content) -> some View {
        content
            .padding(padded ? Theme.Spacing.l : 0)
            .background(Theme.Palette.surface)
            .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.l, style: .continuous))
    }
}

extension View {
    /// 套用统一卡片样式。
    func card(padded: Bool = true) -> some View {
        modifier(CardModifier(padded: padded))
    }
}

/// 主色按钮样式（简约：圆角 + 主色填充 + 按下反馈）。
struct PrimaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(Theme.Typography.bodyEmphasized)
            .foregroundStyle(.white)
            .frame(maxWidth: .infinity)
            .padding(Theme.Spacing.m)
            .background(
                Theme.Palette.primary.opacity(configuration.isPressed ? 0.75 : 1.0)
            )
            .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.m, style: .continuous))
            .animation(.easeOut(duration: 0.12), value: configuration.isPressed)
    }
}

/// 次级（描边）按钮样式。
struct SecondaryButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(Theme.Typography.bodyEmphasized)
            .foregroundStyle(Theme.Palette.textPrimary)
            .frame(maxWidth: .infinity)
            .padding(Theme.Spacing.m)
            .background(
                RoundedRectangle(cornerRadius: Theme.Radius.m, style: .continuous)
                    .stroke(Theme.Palette.separator, lineWidth: 1)
            )
            .background(
                Theme.Palette.surfaceHover.opacity(configuration.isPressed ? 0.6 : 0)
            )
            .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.m, style: .continuous))
            .animation(.easeOut(duration: 0.12), value: configuration.isPressed)
    }
}

/// 全局页面背景（深色，带安全区域）。
struct AppBackground: ViewModifier {
    func body(content: Content) -> some View {
        content
            .background(Theme.Palette.background.ignoresSafeArea())
            .preferredColorScheme(.dark)
    }
}

extension View {
    func appBackground() -> some View { modifier(AppBackground()) }
}
