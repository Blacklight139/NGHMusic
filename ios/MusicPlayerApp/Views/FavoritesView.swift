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
                LazyVGrid(columns: [GridItem(.adaptive(minimum: 160), spacing: AppTheme.space3)],
                          spacing: AppTheme.space3) {
                    ForEach(groups) { group in
                        Card(title: group.name, subtitle: "\(group.songIds.count) 首")
                    }
                }
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
    var body: some View {
        VStack(alignment: .leading, spacing: AppTheme.space1) {
            Text(title).fontWeight(.semibold).foregroundColor(AppTheme.text)
            Text(subtitle).font(.caption).foregroundColor(AppTheme.textMuted)
        }
        .padding(AppTheme.space4)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(AppTheme.bg)
        .cornerRadius(AppTheme.radius)
        .overlay(RoundedRectangle(cornerRadius: AppTheme.radius).stroke(AppTheme.border, lineWidth: 1))
    }
}
