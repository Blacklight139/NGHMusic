// MARK: - NasView
// NAS / 协议音源浏览页：
// - 飞牛 NAS：健康检查、登录、目录浏览、点击音频文件经 feiniu_stream 拉流播放。
// - 协议源：添加 / 列出 / 删除、浏览目录、点击音频文件经 protocol_stream 拉流播放。
// 重试策略遵循 docs：502/504 指数退避（1s/2s/4s，最多 3 次），401 提示重新登录，
// 404 修正路径，501 占位提示（SMB/DLNA/NFS）。

import SwiftUI

struct NasView: View {
    @EnvironmentObject var player: PlayerService

    private enum Mode: String, CaseIterable, Identifiable {
        case feiniu
        case `protocol`
        var id: String { rawValue }
        var title: String {
            switch self {
            case .feiniu: return "飞牛 NAS"
            case .protocol: return "协议源"
            }
        }
        var symbol: String {
            switch self {
            case .feiniu: return "externaldrive.connected.to.line.below"
            case .protocol: return "globe.asia.australia"
            }
        }
    }

    @State private var mode: Mode = .feiniu

    // 飞牛 NAS 状态
    @State private var baseUrl: String = ""
    @State private var username: String = ""
    @State private var password: String = ""
    @State private var healthText: String = "未检测"
    @State private var healthOk: Bool? = nil
    @State private var feiniuPath: String = "/"
    @State private var feiniuFiles: [NasFile] = []
    @State private var feiniuLoading: Bool = false

    // 协议源状态
    @State private var protocolSources: [ProtocolSource] = []
    @State private var selectedSourceId: String?
    @State private var protocolPath: String = "/"
    @State private var protocolEntries: [String] = []
    @State private var protocolLoading: Bool = false
    @State private var showAddSourceSheet: Bool = false
    @State private var newSourceJson: String = ""

    // 通用
    @State private var errorMessage: String?
    @State private var infoMessage: String?

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                Picker("模式", selection: $mode) {
                    ForEach(Mode.allCases) { m in
                        Label(m.title, systemImage: m.symbol).tag(m)
                    }
                }
                .pickerStyle(.segmented)

                switch mode {
                case .feiniu:
                    feiniuPanel
                case .protocol:
                    protocolPanel
                }
            }
            .padding(24)
        }
        .navigationTitle("NAS / 协议音源")
        .frame(maxWidth: .infinity, maxHeight: .infinity)
        .onAppear {
            Task { await loadProtocolSources() }
        }
        .alert("错误", isPresented: Binding(get: { errorMessage != nil }, set: { if !$0 { errorMessage = nil } })) {
            Button("好") { errorMessage = nil }
        } message: { Text(errorMessage ?? "") }
        .alert("逆光音乐", isPresented: Binding(get: { infoMessage != nil }, set: { if !$0 { infoMessage = nil } })) {
            Button("好") { infoMessage = nil }
        } message: { Text(infoMessage ?? "") }
        .sheet(isPresented: $showAddSourceSheet) { addSourceSheet }
    }

    // MARK: - 飞牛 NAS

    private var feiniuPanel: some View {
        VStack(alignment: .leading, spacing: 16) {
            // 健康检查
            HStack(spacing: 8) {
                Image(systemName: healthSymbol)
                    .foregroundColor(healthOk == true ? Color.nghSuccess : (healthOk == false ? Color.nghDanger : Color.nghTextSecondary))
                Text(healthText)
                    .font(.body)
                Spacer()
                Button {
                    Task { await checkHealth() }
                } label: {
                    Label("健康检查", systemImage: "stethoscope")
                }
            }

            Divider()

            // 登录表单
            VStack(alignment: .leading, spacing: 8) {
                Text("飞牛登录").font(.headline)
                TextField("服务地址，如 https://nas.example.com", text: $baseUrl)
                    .textFieldStyle(.roundedBorder)
                TextField("用户名", text: $username)
                    .textFieldStyle(.roundedBorder)
                SecureField("密码", text: $password)
                    .textFieldStyle(.roundedBorder)
                Button {
                    Task { await login() }
                } label: {
                    Label("登录", systemImage: "arrow.right.square")
                }
                .buttonStyle(.borderedProminent)
            }

            Divider()

            // 浏览区
            feiniuBrowser
        }
    }

    private var feiniuBrowser: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                Button {
                    goFeiniuUp()
                } label: {
                    Image(systemName: "chevron.left")
                }
                .disabled(feiniuPath == "/" || feiniuPath.isEmpty)
                Text(feiniuPath).font(.body)
                Spacer()
                Button {
                    Task { await refreshFeiniuFiles() }
                } label: {
                    Image(systemName: "arrow.clockwise")
                }
            }
            if feiniuLoading { ProgressView().padding(.vertical, NghSpacing.s2) }
            if feiniuFiles.isEmpty {
                emptyState(symbol: "folder", text: "未列出文件", subtitle: "登录后点击刷新加载目录")
            } else {
                VStack(spacing: 0) {
                    ForEach(feiniuFiles) { file in
                        feiniuRow(file)
                        Divider()
                    }
                }
                .background(Color(nsColor: .textBackgroundColor).opacity(0.4))
                .cornerRadius(8)
            }
        }
    }

    private func feiniuRow(_ file: NasFile) -> some View {
        Button {
            handleFeiniuTap(file)
        } label: {
            HStack(spacing: 12) {
                Image(systemName: file.isDir ? "folder" : "music.note")
                    .foregroundColor(Color.nghPrimary)
                    .frame(width: 22)
                VStack(alignment: .leading, spacing: 2) {
                    Text(file.name).font(.body)
                    if let m = file.modified, !m.isEmpty {
                        Text(m).font(.caption).foregroundColor(Color.nghTextSecondary)
                    }
                }
                Spacer()
                if !file.isDir {
                    Text(formatBytes(file.size))
                        .font(.caption)
                        .foregroundColor(Color.nghTextSecondary)
                }
            }
            .padding(.horizontal, NghSpacing.s3)
            .padding(.vertical, NghSpacing.s2)
        }
        .buttonStyle(.plain)
    }

    // MARK: - 协议源

    private var protocolPanel: some View {
        VStack(alignment: .leading, spacing: 16) {
            HStack(spacing: 12) {
                Text("协议源").font(.headline)
                Button {
                    Task { await loadProtocolSources() }
                } label: {
                    Label("刷新", systemImage: "arrow.clockwise")
                }
                Button {
                    showAddSourceSheet = true
                } label: {
                    Label("添加", systemImage: "plus")
                }
                .buttonStyle(.borderedProminent)
            }

            if protocolSources.isEmpty {
                emptyState(symbol: "globe", text: "尚无协议源", subtitle: "点击「添加」创建 WebDAV / FTP 源（SMB / DLNA / NFS 为占位）")
            } else {
                VStack(spacing: 0) {
                    HStack {
                        Text("协议 · id")
                            .font(.caption.weight(.semibold)).foregroundColor(Color.nghTextSecondary)
                        Spacer()
                        Text("根")
                            .font(.caption.weight(.semibold)).foregroundColor(Color.nghTextSecondary)
                            .frame(width: 120)
                        Text("占位")
                            .font(.caption.weight(.semibold)).foregroundColor(Color.nghTextSecondary)
                            .frame(width: 60)
                        Text("操作")
                            .font(.caption.weight(.semibold)).foregroundColor(Color.nghTextSecondary)
                            .frame(width: 80)
                    }
                    .padding(.horizontal, NghSpacing.s3).padding(.vertical, NghSpacing.s2)
                    Divider()
                    ForEach(protocolSources) { src in
                        sourceRow(src)
                        Divider()
                    }
                }
                .background(Color(nsColor: .textBackgroundColor).opacity(0.4))
                .cornerRadius(8)
            }

            if let src = selectedSource {
                Divider()
                protocolBrowser(for: src)
            }
        }
    }

    private var selectedSource: ProtocolSource? {
        guard let id = selectedSourceId else { return nil }
        return protocolSources.first { $0.id == id }
    }

    private func sourceRow(_ src: ProtocolSource) -> some View {
        HStack(spacing: 8) {
            VStack(alignment: .leading, spacing: 2) {
                Text("\(src.protocolName) · \(src.id)").font(.body)
                if src.placeholder == true {
                    Text("占位实现，浏览不可用").font(.caption).foregroundColor(Color.nghWarning)
                }
            }
            Spacer()
            Text(src.root.isEmpty ? "/" : src.root)
                .font(.caption).foregroundColor(Color.nghTextSecondary)
                .frame(width: 120, alignment: .leading)
            Image(systemName: src.placeholder == true ? "exclamationmark.triangle" : "checkmark")
                .foregroundColor(src.placeholder == true ? Color.nghWarning : Color.nghSuccess)
                .frame(width: 60)
            HStack(spacing: 8) {
                Button {
                    selectedSourceId = src.id
                    if src.placeholder != true {
                        protocolPath = src.root.isEmpty ? "/" : src.root
                        Task { await refreshProtocolEntries() }
                    }
                } label: {
                    Image(systemName: "folder")
                }
                .buttonStyle(.plain)
                Button(role: .destructive) {
                    Task { await deleteSource(src) }
                } label: {
                    Image(systemName: "trash")
                }
                .buttonStyle(.plain)
            }
            .frame(width: 80)
        }
        .padding(.horizontal, NghSpacing.s3).padding(.vertical, NghSpacing.s2)
    }

    private func protocolBrowser(for src: ProtocolSource) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 8) {
                Text("当前协议源：\(src.protocolName) · \(src.id)").font(.headline)
                Spacer()
            }
            if src.placeholder == true {
                emptyState(symbol: "exclamationmark.triangle",
                          text: "\(src.protocolName) 为占位实现",
                          subtitle: "需启用对应 feature，建议使用 WebDAV / FTP")
            } else {
                HStack(spacing: 8) {
                    Button { goProtocolUp() } label: { Image(systemName: "chevron.left") }
                        .disabled(protocolPath == "/" || protocolPath.isEmpty)
                    Text(protocolPath).font(.body)
                    Spacer()
                    Button { Task { await refreshProtocolEntries() } } label: {
                        Image(systemName: "arrow.clockwise")
                    }
                }
                if protocolLoading { ProgressView().padding(.vertical, NghSpacing.s2) }
                if protocolEntries.isEmpty {
                    emptyState(symbol: "folder", text: "未列出条目", subtitle: "点击刷新加载目录")
                } else {
                    VStack(spacing: 0) {
                        ForEach(protocolEntries, id: \.self) { name in
                            protocolRow(name)
                            Divider()
                        }
                    }
                    .background(Color(nsColor: .textBackgroundColor).opacity(0.4))
                    .cornerRadius(8)
                }
            }
        }
    }

    private func protocolRow(_ rawName: String) -> some View {
        let isDir = rawName.hasSuffix("/")
        let name = rawName.hasSuffix("/") ? String(rawName.dropLast()) : rawName
        return Button {
            handleProtocolTap(name: name, isDir: isDir)
        } label: {
            HStack(spacing: 12) {
                Image(systemName: isDir ? "folder" : "music.note")
                    .foregroundColor(Color.nghPrimary)
                    .frame(width: 22)
                Text(name).font(.body)
                Spacer()
            }
            .padding(.horizontal, NghSpacing.s3).padding(.vertical, NghSpacing.s2)
        }
        .buttonStyle(.plain)
    }

    private var addSourceSheet: some View {
        VStack(spacing: 12) {
            Text("添加协议源").font(.headline)
            Text("粘贴协议源配置 JSON（参考 docs/api/protocol-api.md）")
                .font(.caption).foregroundColor(Color.nghTextSecondary)
            TextEditor(text: $newSourceJson)
                .font(.system(.body, design: .monospaced))
                .frame(minHeight: 220)
                .padding(NghSpacing.s2)
                .background(Color(nsColor: .textBackgroundColor).opacity(0.6))
                .cornerRadius(8)
            HStack {
                Button("取消") { showAddSourceSheet = false; newSourceJson = "" }
                Spacer()
                Button("添加") {
                    Task { await addSource() }
                }
                .buttonStyle(.borderedProminent)
                .disabled(newSourceJson.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
            }
        }
        .padding(20)
        .frame(width: 560)
    }

    // MARK: - 空态

    private func emptyState(symbol: String, text: String, subtitle: String) -> some View {
        VStack(spacing: 8) {
            Image(systemName: symbol)
                .font(.system(size: 36))
                .foregroundColor(Color.nghTextSecondary)
            Text(text).font(.body).foregroundColor(Color.nghTextSecondary)
            Text(subtitle).font(.caption).foregroundColor(Color.nghTextSecondary)
        }
        .frame(maxWidth: .infinity)
        .padding(.vertical, 32)
    }

    // MARK: - 行为：飞牛

    private var healthSymbol: String {
        switch healthOk {
        case .some(true): return "checkmark.seal.fill"
        case .some(false): return "xmark.octagon.fill"
        default: return "questionmark.circle"
        }
    }

    private func checkHealth() async {
        healthText = "检测中…"
        healthOk = nil
        do {
            let resp = try await retry { try await CoreService.shared.feiniuHealth() }
            healthOk = resp.healthy
            healthText = resp.healthy ? "飞牛服务可达" : "飞牛服务不可达"
        } catch {
            healthOk = false
            healthText = "健康检查失败"
            errorMessage = formatError(error)
        }
    }

    private func login() async {
        let base = baseUrl.trimmingCharacters(in: .whitespacesAndNewlines)
        let user = username.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !base.isEmpty, !user.isEmpty else {
            errorMessage = "请填写服务地址与用户名"
            return
        }
        feiniuLoading = true
        do {
            _ = try await CoreService.shared.feiniuLogin(baseUrl: base, username: user, password: password)
            feiniuPath = "/"
            await refreshFeiniuFiles()
            infoMessage = "登录成功"
        } catch {
            if isStatus(error, 401) {
                errorMessage = "用户名或密码错误（401），请检查凭据后重试。"
            } else {
                errorMessage = "登录失败：" + formatError(error)
            }
        }
        feiniuLoading = false
    }

    private func refreshFeiniuFiles() async {
        feiniuLoading = true
        do {
            let resp = try await retry { try await CoreService.shared.feiniuListFiles(feiniuPath) }
            feiniuFiles = resp.files.sorted { a, b in
                if a.isDir != b.isDir { return a.isDir && !b.isDir }
                return a.name < b.name
            }
        } catch {
            feiniuFiles = []
            errorMessage = "列目录失败：" + formatError(error)
        }
        feiniuLoading = false
    }

    private func handleFeiniuTap(_ file: NasFile) {
        if file.isDir {
            feiniuPath = joinPath(parent: feiniuPath, name: file.name, isDir: true)
            Task { await refreshFeiniuFiles() }
            return
        }
        guard isAudio(file.name) else { return }
        Task { await playFeiniu(file) }
    }

    private func playFeiniu(_ file: NasFile) async {
        feiniuLoading = true
        let full = joinPath(parent: feiniuPath, name: file.name, isDir: false)
        do {
            let resp = try await retry { try await CoreService.shared.feiniuStream(full) }
            guard !resp.url.isEmpty else {
                errorMessage = "未获取到播放地址"
                feiniuLoading = false
                return
            }
            let song = Song(
                id: "feiniu-" + file.name,
                sourceId: "feiniu",
                title: file.name,
                artists: [],
                playUrl: resp.url,
                origin: .nas(protocolName: "feiniu", url: resp.url)
            )
            await MainActor.run { player.loadQueue([song]) }
            infoMessage = "开始播放：" + file.name
        } catch {
            errorMessage = "获取播放地址失败：" + formatError(error)
        }
        feiniuLoading = false
    }

    private func goFeiniuUp() {
        feiniuPath = parentPath(feiniuPath)
        Task { await refreshFeiniuFiles() }
    }

    // MARK: - 行为：协议源

    private func loadProtocolSources() async {
        do {
            let list = try await CoreService.shared.protocolList()
            await MainActor.run { protocolSources = list }
        } catch {
            errorMessage = "加载协议源失败：" + formatError(error)
        }
    }

    private func addSource() async {
        let json = newSourceJson.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !json.isEmpty else { return }
        do {
            _ = try await CoreService.shared.protocolAdd(json)
            await MainActor.run {
                showAddSourceSheet = false
                newSourceJson = ""
            }
            await loadProtocolSources()
            infoMessage = "协议源已添加"
        } catch {
            errorMessage = "添加协议源失败：" + formatError(error)
        }
    }

    private func deleteSource(_ src: ProtocolSource) async {
        do {
            try await CoreService.shared.protocolDelete(src.id)
            if selectedSourceId == src.id { selectedSourceId = nil }
            await loadProtocolSources()
            infoMessage = "已删除协议源 " + src.id
        } catch {
            errorMessage = "删除失败：" + formatError(error)
        }
    }

    private func refreshProtocolEntries() async {
        guard let id = selectedSourceId, !id.isEmpty else { return }
        protocolLoading = true
        do {
            let resp = try await retry { try await CoreService.shared.protocolListFiles(id: id, path: protocolPath) }
            protocolEntries = resp.entries.sorted { a, b in
                let aDir = a.hasSuffix("/")
                let bDir = b.hasSuffix("/")
                if aDir != bDir { return aDir && !bDir }
                return a < b
            }
        } catch {
            protocolEntries = []
            errorMessage = "浏览失败：" + formatError(error)
        }
        protocolLoading = false
    }

    private func handleProtocolTap(name: String, isDir: Bool) {
        if isDir {
            protocolPath = joinPath(parent: protocolPath, name: name, isDir: true)
            Task { await refreshProtocolEntries() }
            return
        }
        guard isAudio(name) else { return }
        Task { await playProtocol(name: name) }
    }

    private func playProtocol(name: String) async {
        guard let id = selectedSourceId else { return }
        protocolLoading = true
        let full = joinPath(parent: protocolPath, name: name, isDir: false)
        do {
            let resp = try await retry { try await CoreService.shared.protocolStream(id: id, path: full) }
            guard !resp.url.isEmpty else {
                errorMessage = "未获取到播放地址"
                protocolLoading = false
                return
            }
            let song = Song(
                id: "proto-" + name,
                sourceId: "protocol",
                title: name,
                artists: [],
                playUrl: resp.url,
                origin: .nas(protocolName: "protocol", url: resp.url)
            )
            await MainActor.run { player.loadQueue([song]) }
            infoMessage = "开始播放：" + name
        } catch {
            errorMessage = "获取播放地址失败：" + formatError(error)
        }
        protocolLoading = false
    }

    private func goProtocolUp() {
        protocolPath = parentPath(protocolPath)
        Task { await refreshProtocolEntries() }
    }

    // MARK: - 重试与错误处理

    /// 对 502/504 类网络错误做指数退避重试（1s/2s/4s，最多 3 次）；
    /// 401 / 404 / 501 等不重试，直接抛出由调用方处理。
    private func retry<T>(_ fn: @escaping () async throws -> T) async throws -> T {
        let delays: [UInt64] = [1_000_000_000, 2_000_000_000, 4_000_000_000]
        var last: Error?
        for attempt in 0...3 {
            do {
                return try await fn()
            } catch {
                last = error
                if !isRetryable(error) || attempt == 3 { break }
                try? await Task.sleep(nanoseconds: delays[attempt])
            }
        }
        throw last ?? MusicCoreError.ffi("重试后仍失败")
    }

    private func isRetryable(_ error: Error) -> Bool {
        let msg = (error as? MusicCoreError)?.description ?? error.localizedDescription
        return msg.contains("502") || msg.contains("504") ||
               msg.contains("不可达") || msg.contains("请求失败")
    }

    private func isStatus(_ error: Error, _ status: Int) -> Bool {
        let msg = (error as? MusicCoreError)?.description ?? error.localizedDescription
        return msg.contains(String(status))
    }

    private func formatError(_ error: Error) -> String {
        let msg = (error as? MusicCoreError)?.description ?? error.localizedDescription
        if isStatus(error, 401) { return msg + "\n提示：未登录或 token 失效，请重新登录。" }
        if isStatus(error, 404) { return msg + "\n提示：路径不存在，请修正路径。" }
        if isStatus(error, 501) { return msg + "\n提示：该协议为占位实现，请使用 WebDAV / FTP 或启用对应 feature。" }
        return msg
    }

    // MARK: - 路径与格式工具

    private func joinPath(parent: String, name: String, isDir: Bool) -> String {
        let p = parent.hasSuffix("/") ? String(parent.dropLast()) : parent
        let n = name.hasPrefix("/") ? String(name.dropFirst()) : name
        var full = p.isEmpty ? "/" + n : p + "/" + n
        if isDir && !full.hasSuffix("/") { full += "/" }
        return full
    }

    private func parentPath(_ path: String) -> String {
        guard !path.isEmpty, path != "/" else { return "/" }
        var p = path.hasSuffix("/") ? String(path.dropLast()) : path
        if p.isEmpty { return "/" }
        if let idx = p.lastIndex(of: "/") {
            if idx == p.startIndex { return "/" }
            p = String(p[..<idx])
            return p.isEmpty ? "/" : p
        }
        return "/"
    }

    private func isAudio(_ name: String) -> Bool {
        let exts = [".mp3", ".flac", ".wav", ".m4a", ".aac", ".ogg", ".opus", ".wma"]
        let lower = name.lowercased()
        return exts.contains(where: { lower.hasSuffix($0) })
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
    NasView()
        .environmentObject(PlayerService())
        .frame(width: 900, height: 640)
}
