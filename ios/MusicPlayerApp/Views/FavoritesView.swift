// MARK: - FavoritesView
// 职责：收藏页，展示收藏分组卡片网格，简约风格占位。

import SwiftUI

struct FavoritesView: View {
    @State private var groups: [FavoriteGroup] = [
        FavoriteGroup(id: "f1", name: "我的收藏", songIds: ["s1", "s2", "s3"]),
        FavoriteGroup(id: "f2", name: "睡前音乐", songIds: ["s4", "s5"]),
    ]

    var body: some View {
        PageContainer(title: "收藏", subtitle: "多分组管理，支持导入/导出") {
            if groups.isEmpty {
                EmptyState(text: "还没有收藏分组，点击右上角创建")
            } else {
                LazyVGrid(columns: [GridItem(.adaptive(minimum: 160), spacing: NghSpacing.s3)],
                          spacing: NghSpacing.s3) {
                    ForEach(groups) { group in
                        Button { /* press 反馈，暂无跳转 */ } label: {
                            Card(title: group.name, subtitle: "\(group.songIds.count) 首",
                                 systemImage: "heart.fill")
                        }
                        .nghPressableStyle()
                        .transition(.opacity.combined(with: .move(edge: .top)))
                    }
                }
                // iOS 15+：卡片出现时 staggered fade-in。
                .animation(.easeOut(duration: 0.3), value: groups.count)
            }
        }
    }
}

struct FavoriteGroup: Identifiable {
    let id: String
    let name: String
    let songIds: [String]
}

struct Card: View {
    let title: String
    let subtitle: String
    var systemImage: String? = nil

    var body: some View {
        HStack(spacing: NghSpacing.s3) {
            if let systemImage = systemImage {
                Image(systemName: systemImage)
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundColor(Color.nghPrimary)
                    .frame(width: 36, height: 36)
                    .background(Color.nghPrimarySoft)
                    .clipShape(RoundedRectangle(cornerRadius: NghRadius.sm, style: .continuous))
            }
            VStack(alignment: .leading, spacing: NghSpacing.s1) {
                Text(title).fontWeight(.semibold).foregroundColor(Color.nghText)
                Text(subtitle).font(.caption).foregroundColor(Color.nghTextSecondary)
            }
            Spacer(minLength: 0)
        }
        .padding(NghSpacing.s4)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(RoundedRectangle(cornerRadius: NghRadius.md, style: .continuous).fill(Color.nghSurface))
        .nghCardShadow()
    }
}
