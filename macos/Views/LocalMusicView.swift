// MARK: - LocalMusicView
// 本地音乐页：扫描目录管理 + 扫描进度 + 本地文件列表。
// 通过 CoreService.shared 调用 local_init / local_add_dir / local_rescan / local_progress。

import SwiftUI
import UniformTypeIdentifiers

struct LocalMusicView: View {
    @State private var scanDirs: [URL] = []
    @State private var progress: LocalProgressResponse?
    @State private var errorMessage: String?
    @State private var isLoading = false
    @State private var showFolderPicker = false

    /// 模拟本地歌曲列表（占位）。
    /// 真实数据需通过本地搜索接口获取（search 指定 source_id=local）。
    @State private var songs: [Song] = []

    var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            directoryList
            Divider()
            progressSection
            Divider()
            if isLoading {
                loadingView
            } else if let error = errorMessage {
                errorView(error)
            } else if songs.isEmpty {
                emptyView
            } else {
                songsList
            }
        }
        .navigationTitle("本地音乐")
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onAppear { loadInitial() }
    }

    private var header: some View {
        HStack(spacing: 12) {
            Button {
                showFolderPicker = true
            } label: {
                Label("添加目录", systemImage: "folder.badge.plus")
            }
            Button {
                rescan()
            } label: {
                Label("重新扫描", systemImage: "arrow.triangle.2.circlepath")
            }
            .disabled(scanDirs.isEmpty)
            Spacer()
            Button {
                refreshProgress()
            } label: {
                Label("刷新进度", systemImage: "arrow.clockwise")
            }
        }
        .padding(.horizontal, NghSpacing.s4)
        .padding(.vertical, NghSpacing.s3)
        .fileImporter(
            isPresented: $showFolderPicker,
            allowedContentTypes: [.folder]
        ) { result in
            switch result {
            case .success(let url):
                let didStart = url.startAccessingSecurityScopedResource()
                defer { if didStart { url.stopAccessingSecurityScopedResource() } }
                addDirectory(url)
            case .failure(let error):
                self.errorMessage = error.localizedDescription
            }
        }
    }

    private var directoryList: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text("已添加扫描目录")
                .font(.headline)
                .padding(.horizontal, NghSpacing.s4)
                .padding(.top, NghSpacing.s2)
            if scanDirs.isEmpty {
                Text("尚未添加任何目录")
                    .font(.caption)
                    .foregroundColor(Color.nghTextSecondary)
                    .padding(.horizontal, NghSpacing.s4)
                    .padding(.vertical, 6)
            } else {
                ForEach(scanDirs, id: \.self) { dir in
                    HStack(spacing: 8) {
                        Image(systemName: "folder")
                            .foregroundColor(Color.nghPrimary)
                        Text(dir.path)
                            .font(.caption)
                            .lineLimit(1)
                            .truncationMode(.middle)
                        Spacer()
                        Button {
                            removeDirectory(dir)
                        } label: {
                            Image(systemName: "minus.circle")
                                .foregroundColor(Color.nghDanger)
                        }
                        .buttonStyle(.plain)
                    }
                    .padding(.horizontal, NghSpacing.s4)
                    .padding(.vertical, 4)
                }
            }
        }
        .padding(.bottom, NghSpacing.s2)
    }

    private var progressSection: some View {
        HStack(spacing: 16) {
            if let p = progress {
                Label("\(p.currentCount) 首已扫描", systemImage: "checkmark.circle")
                    .font(.caption)
                if p.scanning {
                    Label("扫描中…", systemImage: "arrow.triangle.2.circlepath.circle")
                        .font(.caption)
                        .foregroundColor(Color.nghPrimary)
                }
            } else {
                Text("进度未知")
                    .font(.caption)
                    .foregroundColor(Color.nghTextSecondary)
            }
            Spacer()
        }
        .padding(.horizontal, NghSpacing.s4)
        .padding(.vertical, NghSpacing.s2)
    }

    private var songsList: some View {
        ScrollView {
            LazyVStack(spacing: 0) {
                ForEach(songs) { song in
                    SongRow(song: song)
                        .padding(.horizontal, NghSpacing.s4)
                        Divider().padding(.leading, 56)
                }
            }
        }
    }

    private var loadingView: some View {
        VStack(spacing: 12) {
            ProgressView()
            Text("加载中…")
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private func errorView(_ message: String) -> some View {
        VStack(spacing: 12) {
            Image(systemName: "exclamationmark.triangle")
                .font(.largeTitle)
                .foregroundColor(Color.nghWarning)
            Text("操作失败")
                .font(.headline)
            Text(message)
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
                .multilineTextAlignment(.center)
                .padding(.horizontal, 32)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    private var emptyView: some View {
        VStack(spacing: 8) {
            Image(systemName: "music.note.house")
                .font(.system(size: 48))
                .foregroundColor(Color.nghTextSecondary)
            Text("尚未发现本地音乐")
                .font(.title3)
            Text("点击上方“添加目录”开始扫描本地音乐")
                .font(.caption)
                .foregroundColor(Color.nghTextSecondary)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity)
    }

    // MARK: - 行为

    private func loadInitial() {
        Task {
            await refreshProgressAsync()
            // 占位：实际歌曲列表需通过 search(sourceId="local") 获取
        }
    }

    private func addDirectory(_ url: URL) {
        isLoading = true
        errorMessage = nil
        Task {
            do {
                _ = try await CoreService.shared.localAddDir(url.path)
                await MainActor.run {
                    if !self.scanDirs.contains(url) { self.scanDirs.append(url) }
                    self.isLoading = false
                    self.rescan()
                }
            } catch {
                await MainActor.run {
                    self.errorMessage = (error as? MusicCoreError)?.description ?? error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }

    private func removeDirectory(_ url: URL) {
        // 占位：core 未提供 local_remove_dir；仅从 UI 列表移除
        scanDirs.removeAll { $0 == url }
    }

    private func rescan() {
        isLoading = true
        errorMessage = nil
        Task {
            do {
                _ = try await CoreService.shared.localRescan()
                await self.refreshProgressAsync()
                await MainActor.run { self.isLoading = false }
            } catch {
                await MainActor.run {
                    self.errorMessage = (error as? MusicCoreError)?.description ?? error.localizedDescription
                    self.isLoading = false
                }
            }
        }
    }

    private func refreshProgress() {
        Task { await refreshProgressAsync() }
    }

    private func refreshProgressAsync() async {
        do {
            let p = try await CoreService.shared.localProgress()
            await MainActor.run { self.progress = p }
        } catch {
            // 静默失败：进度查询失败不阻塞主流程
            await MainActor.run { self.progress = nil }
        }
    }
}

#Preview {
    LocalMusicView()
        .frame(width: 800, height: 600)
}
