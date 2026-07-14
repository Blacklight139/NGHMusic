// MARK: - AppTheme
// 职责：逆光音乐 / NGHMusic macOS 设计系统 token（豆包风格），与 iOS/Android/HarmonyOS 对齐。
// 配色：主色 #4E6EF2（柔和蓝紫）、背景 #F7F8FA、表面 #FFFFFF、
//       文本 #1F1F1F/#6B6B6B/#999999、边框 #EDEEF0。

import SwiftUI

// MARK: - Color hex initializer
extension Color {
    init(hex: UInt32, alpha: Double = 1) {
        self.init(
            .sRGB,
            red: Double((hex >> 16) & 0xff) / 255,
            green: Double((hex >> 8) & 0xff) / 255,
            blue: Double(hex & 0xff) / 255,
            opacity: alpha
        )
    }
}

// MARK: - NghColor（豆包风格 token）
extension Color {
    // 主色
    static let nghPrimary = Color(hex: 0x4E6EF2)
    static let nghPrimaryHover = Color(hex: 0x3D5AE0)
    static let nghPrimarySoft = Color(red: 78.0 / 255.0,
                                      green: 110.0 / 255.0,
                                      blue: 242.0 / 255.0).opacity(0.08)
    // 背景 / 表面
    static let nghBackground = Color(hex: 0xF7F8FA)
    static let nghSurface = Color(hex: 0xFFFFFF)
    static let nghSurfaceAlt = Color(hex: 0xF0F2F5)
    static let nghSidebarBackground = Color(hex: 0xFFFFFF)
    // 文本
    static let nghText = Color(hex: 0x1F1F1F)
    static let nghTextPrimary = Color(hex: 0x1F1F1F)
    static let nghTextSecondary = Color(hex: 0x6B6B6B)
    static let nghTextTertiary = Color(hex: 0x999999)
    // 边框
    static let nghBorder = Color(hex: 0xEDEEF0)
    static let nghBorderSoft = Color(hex: 0xF5F6F8)
    // 语义色
    static let nghDanger = Color(hex: 0xF5483B)
    static let nghSuccess = Color(hex: 0x00B96B)
    static let nghWarning = Color(hex: 0xFA8C16)
    // 阴影
    static let nghShadow = Color.black.opacity(0.06)
}

// MARK: - NghRadius
enum NghRadius {
    static let sm: CGFloat = 8
    static let md: CGFloat = 12
    static let lg: CGFloat = 16
    static let pill: CGFloat = 999
}

// MARK: - NghSpacing
enum NghSpacing {
    static let s1: CGFloat = 4
    static let s2: CGFloat = 8
    static let s3: CGFloat = 12
    static let s4: CGFloat = 16
    static let s5: CGFloat = 20
    static let s6: CGFloat = 24
    static let s7: CGFloat = 32
    static let s8: CGFloat = 40
}

// MARK: - NghShadow（柔和卡片阴影）
extension View {
    func nghCardShadow(color: Color = .nghShadow,
                       radius: CGFloat = 8,
                       x: CGFloat = 0,
                       y: CGFloat = 2) -> some View {
        shadow(color: color, radius: radius, x: x, y: y)
    }
}

// MARK: - NghPressableButtonStyle（豆包风格 press 反馈）
struct NghPressableButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .scaleEffect(configuration.isPressed ? 0.97 : 1.0)
            .animation(.easeOut(duration: 0.15), value: configuration.isPressed)
    }
}

extension View {
    func nghPressableStyle() -> some View {
        buttonStyle(NghPressableButtonStyle())
    }
}

// MARK: - EmptyState
struct EmptyState: View {
    let text: String
    var body: some View {
        VStack(spacing: NghSpacing.s2) {
            Image(systemName: "music.note")
                .font(.system(size: 36))
                .foregroundColor(Color.nghTextTertiary)
            Text(text)
                .font(.subheadline)
                .foregroundColor(Color.nghTextSecondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}
