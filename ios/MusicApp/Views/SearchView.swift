import SwiftUI

/// 搜索状态管理。封装 FFIBridge.search 为可分页的 async 流程。
@MainActor
final class SearchStore: ObservableObject {
    @Published var query = ""
    @Published var selectedType: SearchType = .song
    @Published private(set) var results: [SearchResult] = []
    @Published private(set) var isLoading = false
    @Published private(set) var isLoadingMore = false
    @Published var errorMessage: String?
    @Published private(set) var total: UInt32 = 0

    private var page = Page()
    private let limit: UInt32 = 20
    private var lastQuery = ""
    private let bridge = FFIBridge.shared

    var hasMore: Bool { page.hasMore(total: total) }

    /// 执行新搜索（首页）。
    func search() async {
        let trimmed = query.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        page = Page(offset: 0, limit: limit)
        lastQuery = trimmed
        isLoading = true
        errorMessage = nil
        defer { isLoading = false }
        do {
            let paged = try await bridge.search(keyword: trimmed, type: selectedType, page: page)
            results = paged.items
            total = paged.total
            page = Page(offset: paged.offset, limit: paged.limit)
        } catch {
            results = []
            errorMessage = error.localizedDescription
        }
    }

    /// 加载下一页。
    func loadMore() async {
        guard hasMore, !isLoadingMore, !lastQuery.isEmpty else { return }
        let next = page.next
        isLoadingMore = true
        defer { isLoadingMore = false }
        do {
            let paged = try await bridge.search(keyword: lastQuery, type: selectedType, page: next)
            results.append(contentsOf: paged.items)
            total = paged.total
            page = next
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    /// 切换分类时重置结果。
    func reset() {
        results = []
        total = 0
        errorMessage = nil
    }
}

/// 搜索页：输入框、分类(歌曲/专辑/艺术家)、分页、结果列表展示音源来源、点击播放。
struct SearchView: View {
    @EnvironmentObject private var audioPlayer: AudioPlayer
    @StateObject private var store = SearchStore()
    @FocusState private var isFocused: Bool

    var body: some View {
        ScrollView {
            VStack(spacing: Theme.Spacing.l) {
                // 分类选择器
                Picker("搜索分类", selection: $store.selectedType) {
                    ForEach(SearchType.allCases) { type in
                        Text(type.displayName).tag(type)
                    }
                }
                .pickerStyle(.segmented)

                if store.isLoading {
                    LoadingRow(text: "搜索中…")
                } else if let error = store.errorMessage, store.results.isEmpty {
                    EmptyStateView(
                        systemImage: "exclamationmark.magnifyingglass",
                        title: "搜索失败",
                        message: error
                    )
                    .padding(.top, Theme.Spacing.xxl)
                } else if store.results.isEmpty {
                    EmptyStateView(
                        systemImage: "magnifyingglass",
                        title: "输入关键词开始搜索",
                        message: "跨音源聚合搜索，支持歌曲 / 专辑 / 艺术家分类与分页"
                    )
                    .padding(.top, Theme.Spacing.xxl)
                } else {
                    LazyVStack(spacing: Theme.Spacing.s) {
                        ForEach(Array(store.results.enumerated()), id: \.element.id) { index, result in
                            SearchResultRow(result: result) {
                                handleTap(result)
                            }
                            .onAppear {
                                // 滚动至最后一条时自动加载下一页
                                if index == store.results.count - 1, store.hasMore {
                                    Task { await store.loadMore() }
                                }
                            }
                        }

                        if store.isLoadingMore {
                            LoadingRow(text: "加载更多…")
                        } else if store.hasMore {
                            Button {
                                Task { await store.loadMore() }
                            } label: {
                                Text("加载更多")
                                    .font(Theme.Typography.bodyEmphasized)
                                    .foregroundStyle(Theme.Palette.primary)
                                    .frame(maxWidth: .infinity)
                                    .padding()
                            }
                            .buttonStyle(SecondaryButtonStyle())
                        } else if !store.results.isEmpty {
                            Text("没有更多结果")
                                .font(Theme.Typography.caption)
                                .foregroundStyle(Theme.Palette.textTertiary)
                                .padding(.vertical, Theme.Spacing.m)
                        }
                    }
                    .padding(.horizontal, Theme.Spacing.l)
                }
            }
            .padding(.top, Theme.Spacing.s)
            .padding(.bottom, Theme.Spacing.xxl)
        }
        .scrollDismissesKeyboard(.interactively)
        .searchInput()
        .onChange(of: store.selectedType) { _ in store.reset() }
    }

    /// 顶部搜索框（通过 toolbar 实现）。
    @ViewBuilder
    private func searchInput() -> some View {
        self
            .navigationTitle("搜索")
            .navigationBarTitleDisplayMode(.inline)
            .searchable(text: $store.query, prompt: "搜索歌曲、专辑、艺术家")
            .onSubmit(of: .search) {
                Task { await store.search() }
            }
            .toolbar {
                ToolbarItem(placement: .keyboard) {
                    Button("搜索") {
                        isFocused = false
                        Task { await store.search() }
                    }
                }
            }
    }

    /// 处理结果点击：歌曲则播放（以当前结果中的歌曲为队列），其它类型进入详情。
    private func handleTap(_ result: SearchResult) {
        switch result.item {
        case .song(let song):
            // 以当前搜索结果中的所有歌曲构造播放队列
            let songs = store.results.compactMap { $0.item.song }
            audioPlayer.play(song, in: songs)
        case .album, .artist:
            // 专辑/艺术家详情需音源 metadata 端点；此处仅提示
            store.errorMessage = "该类型详情需音源 metadata 端点支持，暂不可用"
        }
    }
}

// MARK: - 行视图

/// 单条搜索结果行：封面/图标、标题、副标题、音源来源徽章。
struct SearchResultRow: View {
    let result: SearchResult
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: Theme.Spacing.m) {
                // 图标随类型变化
                iconView

                VStack(alignment: .leading, spacing: 4) {
                    Text(result.item.title)
                        .font(Theme.Typography.bodyEmphasized)
                        .foregroundStyle(Theme.Palette.textPrimary)
                        .lineLimit(1)
                    Text(result.item.subtitle)
                        .font(Theme.Typography.caption)
                        .foregroundStyle(Theme.Palette.textSecondary)
                        .lineLimit(1)
                    SourceBadge(name: result.sourceName)
                }

                Spacer()

                Image(systemName: result.item.song != nil ? "play.circle" : "chevron.right")
                    .font(.title3)
                    .foregroundStyle(Theme.Palette.textTertiary)
            }
            .padding(Theme.Spacing.m)
            .background(Theme.Palette.surface)
            .clipShape(RoundedRectangle(cornerRadius: Theme.Radius.m, style: .continuous))
        }
        .buttonStyle(.plain)
    }

    @ViewBuilder
    private var iconView: some View {
        if case .song(let song) = result.item, let cover = song.coverUrl {
            CoverArt(urlString: cover, size: 48)
        } else {
            RoundedRectangle(cornerRadius: Theme.Radius.cover, style: .continuous)
                .fill(Theme.Palette.surfaceHover)
                .frame(width: 48, height: 48)
                .overlay(
                    Image(systemName: iconForType)
                        .foregroundStyle(Theme.Palette.textTertiary)
                )
        }
    }

    private var iconForType: String {
        switch result.item {
        case .song: return "music.note"
        case .album: return "square.stack"
        case .artist: return "person"
        }
    }
}

/// 加载中行。
struct LoadingRow: View {
    let text: String

    var body: some View {
        HStack(spacing: Theme.Spacing.s) {
            ProgressView().tint(Theme.Palette.primary)
            Text(text)
                .font(Theme.Typography.caption)
                .foregroundStyle(Theme.Palette.textSecondary)
        }
        .frame(maxWidth: .infinity)
        .padding(Theme.Spacing.m)
    }
}
