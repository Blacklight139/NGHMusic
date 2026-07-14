// MARK: - SettingsView
// 设置页：音源管理（LXMusic 式列表 + 文件导入）+ 飞牛/协议源配置 + 缓存管理。
// 通过 CoreService.shared 调用 source_import / source_validate / source_list / source_enable / source_disable / source_delete。

import SwiftUI
import UniformTypeIdentifiers

struct SettingsView: View {
    @State private var sources: [SourceInfo] = []
    @State private var errorMessage: String?
    @State private var infoMessage: String?
    @State private var showFilePicker = false
    @State private var showPasteSheet = false
    @State private var pastedJson: String = ""

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 24) {
                sourcesSection
                Divider()
                cacheSection
                Divider()
                aboutSection
            }
            .padding(24)
        }
        .navigationTitle("设置")
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onAppear { loadSources() }
        .alert("提示",
              isPresented: Binding(
                  get: { infoMessage != nil },
                  set: { if !$0 { infoMessage = nil } }
              )) {
            Button("好") { infoMessage = nil }
        } message: {
            Text(infoMessage ?? "")
        }
        .alert("错误",
              isPresented: Binding(
                  get: { errorMessage != nil },
                  set: { if !$0 { errorMessage = nil } }
              )) {
            Button("好") { errorMessage = nil }
        } message: {
            Text(errorMessage ?? "")
        }
    }

    // MARK: - 音源管理

    private var sourcesSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("音源管理")
                .font(.title2.bold())

            HStack(spacing: 12) {
                Button {
                    showFilePicker = true
                } label: {
                    Label("导入音源文件", systemImage: "doc.badge.plus")
                }
                Button {
                    showPasteSheet = true
                } label: {
                    Label("粘贴 JSON 导入", systemImage: "doc.on.clipboard")
                }
                Spacer()
                Button(action: loadSources) {
                    Label("刷新", systemImage: "arrow.clockwise")
                }
            }
            .fileImporter(
                isPresented: $showFilePicker,
                allowedContentTypes: [.json]
            ) { result in
                switch result {
                case .success(let url):
                    let didStart = url.startAccessingSecurityScopedResource()
                    defer { if didStart { url.stopAccessingSecurityScopedResource() } }
                    importFromFile(url)
                case .failure(let error):
                    errorMessage = error.localizedDescription
                }
            }

            // LXMusic 式音源列表
            if sources.isEmpty {
                HStack(spacing: 8) {
                    Image(systemName: "music.note.list")
                        .foregroundColor(Color.nghTextSecondary)
                    Text("尚未导入任何音源")
                        .foregroundColor(Color.nghTextSecondary)
                }
                .padding(.vertical, NghSpacing.s3)
            } else {
                VStack(spacing: 0) {
                    HStack {
                        Text("音源 ID / 名称")
                            .font(.caption.weight(.semibold))
                            .foregroundColor(Color.nghTextSecondary)
                        Spacer()
                        Text("优先级")
                            .font(.caption.weight(.semibold))
                            .foregroundColor(Color.nghTextSecondary)
                        Text("启用")
                            .font(.caption.weight(.semibold))
                            .foregroundColor(Color.nghTextSecondary)
                            .frame(width: 60)
                        Text("操作")
                            .font(.caption.weight(.semibold))
                            .foregroundColor(Color.nghTextSecondary)
                            .frame(width: 80)
                    }
                    .padding(.horizontal, NghSpacing.s3)
                    .padding(.vertical, NghSpacing.s2)
                    Divider()
                    ForEach(sources) { src in
                        sourceRow(src)
                        Divider()
                    }
                }
                .background(Color(nsColor: .textBackgroundColor).opacity(0.4))
                .cornerRadius(8)
            }
        }
        .sheet(isPresented: $showPasteSheet) {
            pasteSheet
        }
    }

    private func sourceRow(_ src: SourceInfo) -> some View {
        HStack(spacing: 8) {
            VStack(alignment: .leading, spacing: 2) {
                Text(src.name).font(.body)
                Text(src.id).font(.caption).foregroundColor(Color.nghTextSecondary).monospaced()
            }
            Spacer()
            Text("\(src.priority)")
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
                .frame(width: 60)
            Toggle("", isOn: Binding(
                get: { src.enabled },
                set: { newValue in toggle(source: src, enabled: newValue) }
            ))
            .labelsHidden()
            .frame(width: 60)
            HStack(spacing: 4) {
                Button {
                    toggle(source: src, enabled: !src.enabled)
                } label: {
                    Image(systemName: src.enabled ? "checkmark.circle" : "circle")
                }
                .buttonStyle(.plain)
                Button {
                    delete(source: src)
                } label: {
                    Image(systemName: "trash")
                }
                .buttonStyle(.plain)
                .foregroundColor(Color.nghDanger)
                .disabled(src.id == "local") // 本地音源受保护
            }
            .frame(width: 80)
        }
        .padding(.horizontal, NghSpacing.s3)
        .padding(.vertical, NghSpacing.s2)
    }

    private var pasteSheet: some View {
        VStack(spacing: 12) {
            Text("粘贴音源 JSON").font(.headline)
            TextEditor(text: $pastedJson)
                .font(.system(.body, design: .monospaced))
                .frame(minHeight: 240)
                .padding(NghSpacing.s2)
                .background(Color(nsColor: .textBackgroundColor).opacity(0.6))
                .cornerRadius(8)
            HStack {
                Button("取消") { showPasteSheet = false; pastedJson = "" }
                Spacer()
                Button("导入") { importFromText(pastedJson) }
                    .buttonStyle(.borderedProminent)
                    .disabled(pastedJson.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
        }
        .padding(20)
        .frame(width: 560)
    }

    // MARK: - 缓存管理

    private var cacheSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("缓存管理").font(.title2.bold())
            HStack(spacing: 12) {
                Button(action: refreshCacheStats) {
                    Label("刷新统计", systemImage: "arrow.clockwise")
                }
                Button(role: .destructive, action: clearCache) {
                    Label("清空缓存", systemImage: "trash")
                }
                Spacer()
            }
        }
    }

    // MARK: - 关于

    private var aboutSection: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("关于").font(.title2.bold())
            HStack(spacing: 8) {
                Image(systemName: "music.note")
                    .foregroundColor(Color.nghPrimary)
                Text("逆光音乐 (NGHMusic)")
                    .font(.body)
            }
            Text("跨平台音乐播放器 · macOS 客户端")
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
            HStack(spacing: 8) {
                Button(action: fetchVersion) {
                    Label("查看核心版本", systemImage: "info.circle")
                }
            }
        }
    }

    // MARK: - 行为

    private func loadSources() {
        Task {
            do {
                let list = try await CoreService.shared.sourceList()
                await MainActor.run { self.sources = list }
            } catch {
                await MainActor.run {
                    self.errorMessage = (error as? MusicCoreError)?.description ?? error.localizedDescription
                }
            }
        }
    }

    private func importFromFile(_ url: URL) {
        guard let data = try? Data(contentsOf: url),
              let json = String(data: data, encoding: .utf8) else {
            errorMessage = "无法读取文件或文件非 UTF-8 文本"
            return
        }
        importFromText(json)
    }

    private func importFromText(_ json: String) {
        Task {
            do {
                // 先校验再导入
                let validate = try await CoreService.shared.sourceValidate(json)
                if !validate.valid {
                    await MainActor.run {
                        self.infoMessage = "音源校验未通过：\n" + validate.errors.joined(separator: "\n")
                    }
                    return
                }
                let result = try await CoreService.shared.sourceImport(json)
                var msg = "已导入音源（\(result.sourceFormat)）"
                if !result.warnings.isEmpty {
                    msg += "\n警告：\n" + result.warnings.joined(separator: "\n")
                }
                await MainActor.run {
                    self.infoMessage = msg
                    self.showPasteSheet = false
                    self.pastedJson = ""
                    self.loadSources()
                }
            } catch {
                await MainActor.run {
                    self.errorMessage = (error as? MusicCoreError)?.description ?? error.localizedDescription
                }
            }
        }
    }

    private func toggle(source: SourceInfo, enabled: Bool) {
        Task {
            do {
                if enabled {
                    try await CoreService.shared.sourceEnable(source.id)
                } else {
                    try await CoreService.shared.sourceDisable(source.id)
                }
                await MainActor.run { self.loadSources() }
            } catch {
                await MainActor.run {
                    self.errorMessage = (error as? MusicCoreError)?.description ?? error.localizedDescription
                }
            }
        }
    }

    private func delete(source: SourceInfo) {
        Task {
            do {
                try await CoreService.shared.sourceDelete(source.id)
                await MainActor.run { self.loadSources() }
            } catch {
                await MainActor.run {
                    self.errorMessage = (error as? MusicCoreError)?.description ?? error.localizedDescription
                }
            }
        }
    }

    private func refreshCacheStats() {
        Task {
            do {
                let stats = try await CoreService.shared.cacheStats()
                await MainActor.run {
                    self.infoMessage = """
缓存项数：\(stats.entries)
已用空间：\(formatBytes(stats.totalBytes))
最大容量：\(formatBytes(stats.maxBytes))
"""
                }
            } catch {
                await MainActor.run {
                    self.errorMessage = (error as? MusicCoreError)?.description ?? error.localizedDescription
                }
            }
        }
    }

    private func clearCache() {
        Task {
            do {
                try await CoreService.shared.cacheClear()
                await MainActor.run { self.infoMessage = "已清空缓存" }
            } catch {
                await MainActor.run {
                    self.errorMessage = (error as? MusicCoreError)?.description ?? error.localizedDescription
                }
            }
        }
    }

    private func fetchVersion() {
        Task {
            do {
                let v = try await CoreService.shared.version()
                await MainActor.run { self.infoMessage = "核心版本：\(v)" }
            } catch {
                await MainActor.run {
                    self.errorMessage = (error as? MusicCoreError)?.description ?? error.localizedDescription
                }
            }
        }
    }

    private func formatBytes(_ bytes: UInt64) -> String {
        let units = ["B", "KB", "MB", "GB", "TB"]
        var value = Double(bytes)
        var i = 0
        while value >= 1024, i < units.count - 1 {
            value /= 1024
            i += 1
        }
        return String(format: "%.2f %@", value, units[i])
    }
}

#Preview {
    SettingsView()
        .frame(width: 800, height: 600)
}
