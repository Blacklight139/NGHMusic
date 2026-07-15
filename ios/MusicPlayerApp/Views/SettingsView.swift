// MARK: - SettingsView
// 职责：设置页。LXMusic 风格音源管理（列表 / 排序 / 开关 / 删除 / 文件导入）+ 核心版本 + 本地目录 + 缓存占位。
// 音源管理通过 MusicCoreBridge.listSourcesOrdered / reorderSources / setSourceEnabled /
// deleteSource / importSourceFromJson 调用核心 SourceManager。
// 文件导入使用 SwiftUI 原生 .fileImporter（iOS 14+，无需 Info.plist 配置 UIDocumentPickerSupport）。

import SwiftUI
import UniformTypeIdentifiers

struct SettingsView: View {
    @State private var sources: [SourceInfo] = []
    @State private var appVersion = "（未连接）"
    @State private var showFileImporter = false
    @State private var importErrorMessage: String?
    @State private var sourceToDelete: SourceInfo?
    @State private var selectedSource: SourceInfo?

    var body: some View {
        List {
            // 核心版本
            Section {
                HStack {
                    Text("核心版本")
                        .font(.subheadline)
                        .foregroundColor(Color.nghTextSecondary)
                    Spacer()
                    Text(appVersion)
                        .font(.subheadline)
                        .foregroundColor(Color.nghText)
                }
            } header: {
                Text("关于")
            }
            .listRowBackground(Color.nghSurface)

            // 音源管理（LXMusic 风格）
            Section {
                if sources.isEmpty {
                    sourceEmptyState
                } else {
                    // IOS-016 修复：MusicCoreBridge.reorderSources / updateSourcePriority 当前为
                    // no-op（核心 FFI 未暴露优先级变更 ABI），拖动排序不会持久化，故移除 .onMove，
                    // 避免乐观 UI 更新与实际状态不一致。桥接支持后再恢复。
                    ForEach(sources) { source in
                        sourceRow(source)
                    }
                }
            } header: {
                sourceSectionHeader
            }
            .listRowBackground(Color.nghSurface)

            // 本地音乐目录（占位，逻辑不变）
            Section {
                Text("暂无目录，将在此添加 / 移除本地目录并触发扫描")
                    .font(.subheadline)
                    .foregroundColor(Color.nghTextSecondary)
            } header: {
                Text("本地音乐目录")
            }
            .listRowBackground(Color.nghSurface)

            // 播放缓存（占位，逻辑不变）
            Section {
                Text("缓存容量与清理功能将在此提供")
                    .font(.subheadline)
                    .foregroundColor(Color.nghTextSecondary)
            } header: {
                Text("播放缓存")
            }
            .listRowBackground(Color.nghSurface)
        }
        .listStyle(.insetGrouped)
        .scrollContentBackground(.hidden) // 需 iOS 16+；低版本可用 UITableView.appearance() 替代
        .background(Color.nghBackground)
        .safeAreaInset(edge: .top, spacing: 0) {
            settingsHeader
        }
        .onAppear {
            loadVersion()
            Task { await loadSources() }
        }
        .fileImporter(isPresented: $showFileImporter,
                      allowedContentTypes: [.json],
                      allowsMultipleSelection: false) { result in
            Task { await handleFileImport(result) }
        }
        .alert("导入失败",
               isPresented: Binding(get: { importErrorMessage != nil },
                                    set: { if !$0 { importErrorMessage = nil } })) {
            Button("好的", role: .cancel) {}
        } message: {
            Text(importErrorMessage ?? "")
        }
        .confirmationDialog(
            sourceToDelete.map { "确定删除音源「\($0.name)」？" } ?? "",
            isPresented: Binding(get: { sourceToDelete != nil },
                                 set: { if !$0 { sourceToDelete = nil } }),
            titleVisibility: .visible
        ) {
            Button("删除", role: .destructive) {
                if let source = sourceToDelete {
                    performDelete(source)
                }
                sourceToDelete = nil
            }
            Button("取消", role: .cancel) {
                sourceToDelete = nil
            }
        }
        .sheet(item: $selectedSource) { source in
            SourceDetailSheet(source: source)
        }
    }

    // MARK: - 子视图

    /// 顶部标题（替代 PageContainer 标题区，因本页改用 List 根布局以支持 .onMove）
    private var settingsHeader: some View {
        VStack(alignment: .leading, spacing: 2) {
            Text("设置")
                .font(.title2).fontWeight(.semibold)
                .foregroundColor(Color.nghText)
            Text("音源管理 / 缓存 / 本地目录")
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, NghSpacing.s4)
        .padding(.vertical, NghSpacing.s3)
        .background(Color.nghBackground)
    }

    /// 音源管理 Section header：标题 + 副标题「已导入 N 个音源」+ 右上角导入按钮
    private var sourceSectionHeader: some View {
        HStack(spacing: NghSpacing.s3) {
            VStack(alignment: .leading, spacing: 2) {
                Text("音源管理")
                    .font(.subheadline).fontWeight(.semibold)
                    .foregroundColor(Color.nghText)
                Text("已导入 \(sources.count) 个音源")
                    .font(.caption2)
                    .foregroundColor(Color.nghTextSecondary)
            }
            Spacer()
            Button {
                showFileImporter = true
            } label: {
                Image(systemName: "square.and.arrow.down")
                    .font(.system(size: 16, weight: .medium))
                    .foregroundColor(Color.nghPrimary)
                    .frame(width: 32, height: 32)
            }
        }
    }

    /// 空状态：暂无音源，点击导入
    private var sourceEmptyState: some View {
        Button {
            showFileImporter = true
        } label: {
            VStack(spacing: NghSpacing.s2) {
                Image(systemName: "tray.and.arrow.down")
                    .font(.system(size: 28))
                    .foregroundColor(Color.nghTextTertiary)
                Text("暂无音源，点击导入")
                    .font(.subheadline)
                    .foregroundColor(Color.nghTextSecondary)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, NghSpacing.s5)
        }
        .buttonStyle(.plain)
    }

    /// 单行音源：拖动手柄 + 名称/标签 + 开关 + 上移/下移 + 删除，点击行展开详情
    private func sourceRow(_ source: SourceInfo) -> some View {
        HStack(spacing: NghSpacing.s2) {
            // 拖动手柄（视觉提示；.onMove 提供实际长按拖动排序）
            Image(systemName: "line.3.horizontal")
                .font(.caption)
                .foregroundColor(Color.nghTextTertiary)

            // 名称 + 来源标签 + 版本
            VStack(alignment: .leading, spacing: NghSpacing.s1) {
                Text(source.name)
                    .font(.subheadline).fontWeight(.semibold)
                    .foregroundColor(Color.nghText)
                    .lineLimit(1)
                HStack(spacing: NghSpacing.s1) {
                    SourceTypeTag(sourceType: source.sourceType)
                    Text("v\(source.version)")
                        .font(.caption2)
                        .foregroundColor(Color.nghTextTertiary)
                }
            }

            Spacer(minLength: NghSpacing.s2)

            // 启用开关
            Toggle("", isOn: Binding(
                get: { source.enabled },
                set: { newValue in toggleSource(source, enabled: newValue) }
            ))
            .labelsHidden()
            .tint(Color.nghPrimary)
            // iOS 15+：开关状态与 tint 切换平滑过渡。
            .animation(.easeInOut(duration: 0.2), value: source.enabled)

            // 上移 / 下移
            VStack(spacing: 0) {
                Button {
                    moveSource(source, up: true)
                } label: {
                    Image(systemName: "chevron.up")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundColor(canMove(source, up: true)
                                         ? Color.nghTextSecondary : Color.nghBorder)
                }
                .disabled(!canMove(source, up: true))
                Button {
                    moveSource(source, up: false)
                } label: {
                    Image(systemName: "chevron.down")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundColor(canMove(source, up: false)
                                         ? Color.nghTextSecondary : Color.nghBorder)
                }
                .disabled(!canMove(source, up: false))
            }
            .buttonStyle(.borderless)

            // 删除
            Button {
                sourceToDelete = source
            } label: {
                Image(systemName: "trash")
                    .font(.system(size: 13))
                    .foregroundColor(Color.nghDanger)
            }
            .buttonStyle(.borderless)
        }
        .contentShape(Rectangle())
        .onTapGesture {
            selectedSource = source
        }
    }

    // MARK: - 数据操作

    private func loadVersion() {
        appVersion = MusicCoreBridge.appVersion()
    }

    private func loadSources() async {
        do {
            sources = try await MusicCoreBridge.listSourcesOrdered()
        } catch {
            sources = []
        }
    }

    private func canMove(_ source: SourceInfo, up: Bool) -> Bool {
        // IOS-016 修复：MusicCoreBridge.reorderSources / updateSourcePriority 当前为 no-op
        // （核心 FFI 未暴露优先级变更 ABI），排序不会持久化 → 禁用上移/下移按钮，
        // 避免乐观 UI 更新与实际状态不一致。桥接支持后再恢复。
        return false
    }

    /// 上移 / 下移：桥接层 reorderSources 为 no-op，不执行乐观更新（见 canMove 注释）。
    private func moveSource(_ source: SourceInfo, up: Bool) {
        // no-op：排序不会持久化，保持 sources 数组不变。
    }

    /// 开关：乐观更新本地状态，失败回退
    private func toggleSource(_ source: SourceInfo, enabled: Bool) {
        guard let idx = sources.firstIndex(where: { $0.id == source.id }) else { return }
        sources[idx].enabled = enabled
        Task {
            do {
                try await MusicCoreBridge.setSourceEnabled(id: source.id, enabled: enabled)
            } catch {
                if let idx = sources.firstIndex(where: { $0.id == source.id }) {
                    sources[idx].enabled = !enabled
                }
                importErrorMessage = "更新开关失败：\(error.localizedDescription)"
            }
        }
    }

    /// 删除：确认后调用 deleteSource
    private func performDelete(_ source: SourceInfo) {
        Task {
            do {
                try await MusicCoreBridge.deleteSource(id: source.id)
                sources.removeAll { $0.id == source.id }
            } catch {
                importErrorMessage = "删除失败：\(error.localizedDescription)"
            }
        }
    }

    /// 文件导入：读取 JSON → importSourceFromJson → 刷新列表
    private func handleFileImport(_ result: Result<URL, Error>) async {
        switch result {
        case .success(let url):
            let didStart = url.startAccessingSecurityScopedResource()
            defer { if didStart { url.stopAccessingSecurityScopedResource() } }
            do {
                let data = try Data(contentsOf: url)
                let jsonStr = String(data: data, encoding: .utf8) ?? ""
                guard !jsonStr.isEmpty else {
                    importErrorMessage = "文件内容为空或非 UTF-8 文本"
                    return
                }
                _ = try await MusicCoreBridge.importSourceFromJson(jsonStr)
                await loadSources()
            } catch {
                importErrorMessage = "导入失败：\(error.localizedDescription)"
            }
        case .failure(let error):
            importErrorMessage = "文件选择失败：\(error.localizedDescription)"
        }
    }
}

// MARK: - SourceTypeTag
/// 来源标签：json=nghPrimary / community=nghWarning / local=nghSuccess
private struct SourceTypeTag: View {
    let sourceType: String
    private var color: Color {
        switch sourceType {
        case "json": return Color.nghPrimary
        case "community": return Color.nghWarning
        case "local": return Color.nghSuccess
        default: return Color.nghTextTertiary
        }
    }
    var body: some View {
        Text(sourceType)
            .font(.caption2)
            .foregroundColor(.white)
            .padding(.horizontal, NghSpacing.s2)
            .padding(.vertical, 2)
            .background(color)
            .clipShape(Capsule())
    }
}

// MARK: - SourceDetailSheet
/// 音源详情：name / id / version / source_type / priority / enabled / description
private struct SourceDetailSheet: View {
    let source: SourceInfo
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List {
                Section {
                    detailRow(title: "名称", value: source.name)
                    detailRow(title: "ID", value: source.id)
                    detailRow(title: "版本", value: source.version)
                    detailRow(title: "类型", value: source.sourceType)
                    detailRow(title: "优先级", value: "\(source.priority)")
                    detailRow(title: "状态", value: source.enabled ? "已启用" : "已禁用")
                } header: {
                    Text("基本信息")
                }
                .listRowBackground(Color.nghSurface)

                Section {
                    Text(source.description.isEmpty ? "（无描述）" : source.description)
                        .font(.subheadline)
                        .foregroundColor(Color.nghTextSecondary)
                } header: {
                    Text("描述")
                }
                .listRowBackground(Color.nghSurface)
            }
            .listStyle(.insetGrouped)
            .scrollContentBackground(.hidden)
            .background(Color.nghBackground)
            .navigationTitle("音源详情")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("关闭") { dismiss() }
                }
            }
        }
    }

    private func detailRow(title: String, value: String) -> some View {
        HStack {
            Text(title).foregroundColor(Color.nghTextSecondary)
            Spacer()
            Text(value)
                .foregroundColor(Color.nghText)
                .lineLimit(1)
                .truncationMode(.middle)
        }
    }
}

#Preview {
    SettingsView()
}
