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
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
            Divider()

            List {
                Section("分组") {
                    ForEach(groups, id: \.self) { group in
                        HStack {
                            Image(systemName: "heart.fill")
                                .foregroundColor(.pink)
                            Text(group)
                            Spacer()
                            Text("0")
                                .font(.caption)
                                .foregroundColor(.secondary)
                        }
                    }
                }
                Section {
                    HStack(spacing: 12) {
                        Image(systemName: "heart.slash")
                            .font(.title3)
                            .foregroundColor(.secondary)
                        Text("尚无收藏歌曲")
                            .foregroundColor(.secondary)
                    }
                } header: {
                    Text("歌曲")
                }
            }
        }
        .navigationTitle("收藏")
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }
}

#Preview {
    FavoritesView()
        .frame(width: 800, height: 600)
}
