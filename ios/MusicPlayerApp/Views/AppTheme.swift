// MARK: - AppTheme
// 职责：简约设计系统 token，与桌面端 styles.css 对齐。
// 配色：主色 #1db954、背景 #fff/#f5f5f5、文本 #333/#6b6b6b、边框 #e2e2e2。

import SwiftUI

enum AppTheme {
    static let primary = Color(hex: 0x1db954)
    static let primaryDark = Color(hex: 0x169c46)
    static let bg = Color(hex: 0xffffff)
    static let bgAlt = Color(hex: 0xf5f5f5)
    static let text = Color(hex: 0x333333)
    static let textMuted = Color(hex: 0x6b6b6b)
    static let border = Color(hex: 0xe2e2e2)

    static let space1: CGFloat = 4
    static let space2: CGFloat = 8
    static let space3: CGFloat = 12
    static let space4: CGFloat = 16
    static let space5: CGFloat = 24
    static let space6: CGFloat = 32
    static let radius: CGFloat = 8
}

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

// MARK: - PageContainer
// 通用页面容器：标题 + 副标题 + 内容，与桌面端 .page 样式对齐。
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
            VStack(alignment: .leading, spacing: AppTheme.space4) {
                Text(title).font(.title2).fontWeight(.semibold).foregroundColor(AppTheme.text)
                if let subtitle = subtitle {
                    Text(subtitle).font(.caption).foregroundColor(AppTheme.textMuted)
                }
                content()
            }
            .padding(AppTheme.space5)
            .frame(maxWidth: .infinity, alignment: .leading)
        }
        .background(AppTheme.bgAlt)
    }
}

// MARK: - EmptyState
struct EmptyState: View {
    let text: String
    var body: some View {
        Text(text)
            .font(.subheadline)
            .foregroundColor(AppTheme.textMuted)
            .frame(maxWidth: .infinity)
            .padding(AppTheme.space5)
            .background(AppTheme.bg)
            .overlay(
                RoundedRectangle(cornerRadius: AppTheme.radius)
                    .strokeBorder(style: StrokeStyle(lineWidth: 1, dash: [4]))
                    .foregroundColor(AppTheme.border)
            )
            .cornerRadius(AppTheme.radius)
    }
}
