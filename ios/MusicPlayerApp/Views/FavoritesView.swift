// MARK: - FavoritesView
// 职责：收藏页，展示收藏分组列表（无卡片，与搜索/播放列表风格一致）。
// 收藏数据由客户端持久化管理（docs FFI 未暴露收藏 API），脚手架阶段展示占位分组。
// 后续 Task 14 将实现多分组 / 添加 / 移除 / 导入 / 导出 + 持久化。

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
                // Linear 风格：与搜索/播放列表一致的无卡片列表 + 底部分割线。
                VStack(spacing: 0) {
                    ForEach(groups) { group in
                        Button { /* press 反馈，暂无跳转 */ } label: {
                            FavoriteGroupRow(name: group.name, count: group.songIds.count)
                        }
                        .nghPressableStyle()
                        .transition(.opacity.combined(with: .move(edge: .top)))
                    }
                    if !groups.isEmpty {
                        Button {
                            // 占位：新增分组（需后续接入数据层）
                        } label: {
                            Label("新建分组", systemImage: "plus")
                                .font(.subheadline)
                                .foregroundColor(Color.nghPrimary)
                                .frame(maxWidth: .infinity)
                                .padding(.vertical, NghSpacing.s3)
                        }
                        .padding(.top, NghSpacing.s2)
                    }
                }
                // iOS 15+：列表项出现时 staggered fade-in。
                .animation(.easeOut(duration: 0.3), value: groups.count)
            }
        }
    }
}

/// 收藏分组行：无卡片，与 SongRow 视觉权重对齐。
struct FavoriteGroupRow: View {
    let name: String
    let count: Int

    var body: some View {
        VStack(spacing: 0) {
            HStack(spacing: NghSpacing.s3) {
                Image(systemName: "heart.fill")
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundColor(Color.nghPrimary)
                    .frame(width: 36, height: 36)
                    .background(Color.nghPrimarySoft)
                    .clipShape(RoundedRectangle(cornerRadius: NghRadius.sm, style: .continuous))
                VStack(alignment: .leading, spacing: NghSpacing.s1) {
                    Text(name).fontWeight(.semibold).foregroundColor(Color.nghText)
                    Text("\(count) 首").font(.caption).foregroundColor(Color.nghTextSecondary)
                }
                Spacer(minLength: 0)
                Image(systemName: "chevron.right")
                    .font(.caption)
                    .foregroundColor(Color.nghTextTertiary)
            }
            .padding(.horizontal, NghSpacing.s4)
            .padding(.vertical, NghSpacing.s3)
            Divider()
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
