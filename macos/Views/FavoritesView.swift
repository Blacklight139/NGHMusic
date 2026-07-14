// MARK: - FavoritesView
// 收藏页：展示收藏分组与已收藏歌曲（Scaffold）。
// 当前实现为占位骨架，后续接入持久化层。

import SwiftUI

struct FavoritesView: View {
    @State private var groups: [String] = ["我喜欢的音乐"]

    var body: some View {
        VStack(spacing: 0) {
            HStack {
                Text("收藏分组").font(.headline)
                Spacer()
                Button {
                    // 占位：新增分组（需后续接入数据层）
                } label: {
                    Label("新建", systemImage: "plus")
                }
            }
            .padding(.horizontal, NghSpacing.s4)
            .padding(.vertical, NghSpacing.s3)
            Divider()

            ScrollView {
                VStack(alignment: .leading, spacing: NghSpacing.s4) {
                    section(title: "分组（\(groups.count)）") {
                        ForEach(groups, id: \.self) { group in
                            HStack(spacing: NghSpacing.s3) {
                                Image(systemName: "heart.fill")
                                    .font(.title3)
                                    .foregroundColor(Color.nghPrimary)
                                    .frame(width: 40, height: 40)
                                    .background(Color.nghPrimarySoft)
                                    .clipShape(RoundedRectangle(cornerRadius: NghRadius.sm))
                                VStack(alignment: .leading, spacing: NghSpacing.s1) {
                                    Text(group).font(.body)
                                    Text("0 首")
                                        .font(.caption)
                                        .foregroundColor(Color.nghTextSecondary)
                                }
                                Spacer()
                            }
                            .padding(.vertical, NghSpacing.s2)
                            Divider()
                        }
                    }

                    section(title: "歌曲") {
                        HStack(spacing: NghSpacing.s3) {
                            Image(systemName: "heart.slash")
                                .font(.title3)
                                .foregroundColor(Color.nghTextTertiary)
                                .frame(width: 40, height: 40)
                                .background(Color.nghSurfaceAlt)
                                .clipShape(RoundedRectangle(cornerRadius: NghRadius.sm))
                            Text("尚无收藏歌曲")
                                .foregroundColor(Color.nghTextSecondary)
                        }
                        .padding(.vertical, NghSpacing.s2)
                    }
                }
                .padding(.horizontal, NghSpacing.s4)
                .padding(.top, NghSpacing.s2)
            }
        }
        .navigationTitle("收藏")
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func section<Content: View>(title: String, @ViewBuilder content: () -> Content) -> some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(title)
                .font(.headline)
                .foregroundColor(Color.nghText)
                .padding(.bottom, NghSpacing.s1)
            content()
        }
    }
}

#Preview {
    FavoritesView()
        .frame(width: 800, height: 600)
}
