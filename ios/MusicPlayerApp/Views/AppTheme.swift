// MARK: - AppTheme
// 职责：逆光音乐 / NGHMusic 设计系统 token（豆包风格），与桌面/Android/HarmonyOS 对齐。
// 配色：主色 #4E6EF2（柔和蓝紫）、背景 #F7F8FA、卡片 #FFFFFF、
//       文本 #1F1F1F/#6B6B6B/#999999、边框 #EDEEF0。
// 旧 AppTheme API 保留为兼容层（值已对齐到新 token），避免破坏既有 View 引用。

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

// MARK: - NghColor（豆包风格 token，快捷访问 Color.ngh*）
extension Color {
    // 主色
    static let nghPrimary = Color(hex: 0x4E6EF2)        // #4E6EF2 柔和蓝紫
    static let nghPrimaryHover = Color(hex: 0x3D5AE0)   // #3D5AE0
    static let nghPrimarySoft = Color(red: 78.0 / 255.0,
                                      green: 110.0 / 255.0,
                                      blue: 242.0 / 255.0).opacity(0.08)
    // 背景 / 表面
    static let nghBackground = Color(hex: 0xF7F8FA)     // #F7F8FA 页面背景
    static let nghSurface = Color(hex: 0xFFFFFF)        // #FFFFFF 卡片
    static let nghSurfaceAlt = Color(hex: 0xF0F2F5)     // #F0F2F5
    static let nghSidebarBackground = Color(hex: 0xFFFFFF)
    // 文本
    static let nghText = Color(hex: 0x1F1F1F)           // #1F1F1F 主文本（nghTextPrimary 别名）
    static let nghTextPrimary = Color(hex: 0x1F1F1F)
    static let nghTextSecondary = Color(hex: 0x6B6B6B)  // #6B6B6B
    static let nghTextTertiary = Color(hex: 0x999999)   // #999999
    // 边框
    static let nghBorder = Color(hex: 0xEDEEF0)         // #EDEEF0
    static let nghBorderSoft = Color(hex: 0xF5F6F8)     // #F5F6F8
    // 语义色
    static let nghDanger = Color(hex: 0xF5483B)         // #F5483B
    static let nghSuccess = Color(hex: 0x00B96B)        // #00B96B
    static let nghWarning = Color(hex: 0xFA8C16)        // #FA8C16
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
    /// 应用豆包风格柔和卡片阴影，默认 color = black.opacity(0.06)。
    func nghCardShadow(color: Color = .nghShadow,
                       radius: CGFloat = 8,
                       x: CGFloat = 0,
                       y: CGFloat = 2) -> some View {
        shadow(color: color, radius: radius, x: x, y: y)
    }
}

// MARK: - NghPressableButtonStyle（豆包风格 press 反馈）
// 按下时 scale 0.97，松开时 scale 1.0，150ms easeOut。
// 通过 ButtonStyle 原生追踪按压状态，与 ScrollView 协作良好，不引入第三方库。
struct NghPressableButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .scaleEffect(configuration.isPressed ? 0.97 : 1.0)
            .animation(.easeOut(duration: 0.15), value: configuration.isPressed)
    }
}

extension View {
    /// 为 Button 应用豆包风格 press 反馈（按下 scale 0.97，松开 1.0）。
    /// 用法：Button { } label: { ... }.nghPressableStyle()
    func nghPressableStyle() -> some View {
        buttonStyle(NghPressableButtonStyle())
    }
}

// MARK: - AppTheme（旧 API 兼容层，值已对齐到豆包 token）
enum AppTheme {
    static let primary = Color.nghPrimary          // #4E6EF2
    static let primaryDark = Color.nghPrimaryHover // #3D5AE0
    static let bg = Color.nghSurface               // #FFFFFF 卡片
    static let bgAlt = Color.nghBackground         // #F7F8FA 页面背景
    static let text = Color.nghTextPrimary         // #1F1F1F
    static let textMuted = Color.nghTextSecondary  // #6B6B6B
    static let border = Color.nghBorder            // #EDEEF0

    static let space1: CGFloat = NghSpacing.s1     // 4
    static let space2: CGFloat = NghSpacing.s2     // 8
    static let space3: CGFloat = NghSpacing.s3     // 12
    static let space4: CGFloat = NghSpacing.s4     // 16
    static let space5: CGFloat = NghSpacing.s6     // 24（保持原值，对齐 s6）
    static let space6: CGFloat = NghSpacing.s7     // 32（保持原值，对齐 s7）
    static let radius: CGFloat = NghRadius.sm      // 8
}

// MARK: - PageContainer
// 通用页面容器：标题 + 副标题 + 内容，豆包风格（背景 nghBackground）。
struct PageContainer<Content: View>: View {
    let title: String
    let subtitle: String?
    @ViewBuilder let content: () -> Content

    init(title: String, subtitle: String? = nil, @ViewBuilder content: @escaping () -> Content) {
        self.title = title
        self.subtitle = subtitle
        self.content = content
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: NghSpacing.s4) {
                Text(title).font(.title2).fontWeight(.semibold).foregroundColor(Color.nghText)
                if let subtitle = subtitle {
                    Text(subtitle).font(.caption).foregroundColor(Color.nghTextSecondary)
                }
                content()
            }
            .padding(NghSpacing.s6)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .background(Color.nghBackground)
    }
}

// MARK: - EmptyState
struct EmptyState: View {
    let text: String
    var body: some View {
        Text(text)
            .font(.subheadline)
            .foregroundColor(Color.nghTextSecondary)
            .frame(maxWidth: .infinity)
            .padding(NghSpacing.s6)
            .background(Color.nghSurface)
            .overlay(
                RoundedRectangle(cornerRadius: NghRadius.sm)
                    .strokeBorder(style: StrokeStyle(lineWidth: 1, dash: [4]))
                    .foregroundColor(Color.nghBorder)
            )
            .cornerRadius(NghRadius.sm)
    }
}
