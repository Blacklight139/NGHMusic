// MARK: - NasView
// NAS / 协议音源浏览页（iOS）：
// - 飞牛 NAS：健康检查、登录、目录浏览、点击音频文件经 feiniu_stream 拉流播放。
// - 协议源：添加 / 列出 / 删除、浏览目录、点击音频文件经 protocol_stream 拉流播放。
// 重试策略遵循 docs：502/504 指数退避（1s/2s/4s，最多 3 次），401 提示重新登录，
// 404 修正路径，501 占位提示（SMB/DLNA/NFS）。
// 豆包风格 token（Color.ngh* / NghSpacing / NghRadius），图标统一 SF Symbols。

import SwiftUI

struct NasView: View {
    @EnvironmentObject var player: PlayerManager

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
        PageContainer(title: "NAS / 协议音源", subtitle: "飞牛 NAS 与网络协议源浏览播放") {
            VStack(alignment: .leading, spacing: NghSpacing.s4) {
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
        }
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
        VStack(alignment: .leading, spacing: NghSpacing.s4) {
            // 健康检查
            HStack(spacing: NghSpacing.s2) {
                Image(systemName: healthSymbol)
                    .foregroundColor(healthOk == true ? .nghSuccess : (healthOk == false ? .nghDanger : .nghTextTertiary))
                Text(healthText).font(.subheadline).foregroundColor(.nghTextSecondary)
                Spacer()
                Button {
                    Task { await checkHealth() }
                } label: {
                    Label("健康检查", systemImage: "stethoscope")
                        .font(.caption)
                }
                .nghPressableStyle()
            }
            .padding(NghSpacing.s3)
            .background(Color.nghSurface)
            .cornerRadius(NghRadius.md)

            // 登录表单
            VStack(alignment: .leading, spacing: NghSpacing.s2) {
                Text("飞牛登录").font(.headline).foregroundColor(.nghText)
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
                        .frame(maxWidth: .infinity)
                }
                .buttonStyle(.borderedProminent)
                .nghPressableStyle()
            }
            .padding(NghSpacing.s3)
            .background(Color.nghSurface)
            .cornerRadius(NghRadius.md)

            feiniuBrowser
        }
    }

    private var feiniuBrowser: some View {
        VStack(alignment: .leading, spacing: NghSpacing.s2) {
            HStack(spacing: NghSpacing.s2) {
                Button { goFeiniuUp() } label: {
                    Image(systemName: "chevron.left")
                }
                .disabled(feiniuPath == "/" || feiniuPath.isEmpty)
                Text(feiniuPath).font(.subheadline).foregroundColor(.nghTextSecondary).lineLimit(1)
                Spacer()
                Button { Task { await refreshFeiniuFiles() } } label: {
                    Image(systemName: "arrow.clockwise")
                }
            }
            if feiniuLoading {
                ProgressView().padding(.vertical, NghSpacing.s2)
            }
            if feiniuFiles.isEmpty {
                EmptyState(text: "未列出文件，登录后刷新")
            } else {
                VStack(spacing: 0) {
                    ForEach(feiniuFiles) { file in
                        feiniuRow(file)
                        Divider().background(Color.nghBorderSoft)
                    }
                }
                .background(Color.nghSurface)
                .cornerRadius(NghRadius.md)
                .nghCardShadow()
            }
        }
    }

    private func feiniuRow(_ file: NasFile) -> some View {
        Button {
            handleFeiniuTap(file)
        } label: {
            HStack(spacing: NghSpacing.s3) {
                Image(systemName: file.isDir ? "folder" : "music.note")
                    .foregroundColor(.nghPrimary)
                    .frame(width: 22)
                VStack(alignment: .leading, spacing: 2) {
                    Text(file.name).font(.body).foregroundColor(.nghText).lineLimit(1)
                    if let m = file.modified, !m.isEmpty {
                        Text(m).font(.caption).foregroundColor(.nghTextTertiary)
                    }
                }
                Spacer()
                if !file.isDir {
                    Text(formatBytes(file.size))
                        .font(.caption)
                        .foregroundColor(.nghTextSecondary)
                }
            }
            .padding(.horizontal, NghSpacing.s3)
            .padding(.vertical, NghSpacing.s2)
        }
        .buttonStyle(.plain)
        .nghPressableStyle()
    }

    // MARK: - 协议源

    private var protocolPanel: some View {
        VStack(alignment: .leading, spacing: NghSpacing.s4) {
            HStack(spacing: NghSpacing.s2) {
                Text("协议源").font(.headline).foregroundColor(.nghText)
                Spacer()
                Button { Task { await loadProtocolSources() } } label: {
                    Image(systemName: "arrow.clockwise")
                }
                Button { showAddSourceSheet = true } label: {
                    Label("添加", systemImage: "plus")
                }
                .buttonStyle(.borderedProminent)
                .nghPressableStyle()
            }

            if protocolSources.isEmpty {
                EmptyState(text: "尚无协议源，点击「添加」创建（WebDAV / FTP 可用，SMB / DLNA / NFS 为占位）")
            } else {
                VStack(spacing: 0) {
                    ForEach(protocolSources) { src in
                        sourceRow(src)
                        Divider().background(Color.nghBorderSoft)
                    }
                }
                .background(Color.nghSurface)
                .cornerRadius(NghRadius.md)
                .nghCardShadow()
            }

            if let src = selectedSource {
                protocolBrowser(for: src)
            }
        }
    }

    private var selectedSource: ProtocolSource? {
        guard let id = selectedSourceId else { return nil }
        return protocolSources.first { $0.id == id }
    }

    private func sourceRow(_ src: ProtocolSource) -> some View {
        HStack(spacing: NghSpacing.s2) {
            VStack(alignment: .leading, spacing: 2) {
                Text("\(src.protocolName) · \(src.id)").font(.body).foregroundColor(.nghText)
                if src.placeholder {
                    Text("占位实现，浏览不可用").font(.caption).foregroundColor(.nghWarning)
                }
            }
            Spacer()
            Button {
                selectedSourceId = src.id
                if !src.placeholder {
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
                    .foregroundColor(.nghDanger)
            }
            .buttonStyle(.plain)
        }
        .padding(.horizontal, NghSpacing.s3)
        .padding(.vertical, NghSpacing.s2)
    }

    private func protocolBrowser(for src: ProtocolSource) -> some View {
        VStack(alignment: .leading, spacing: NghSpacing.s2) {
            Text("当前协议源：\(src.protocolName) · \(src.id)").font(.headline).foregroundColor(.nghText)
            if src.placeholder {
                EmptyState(text: "\(src.protocolName) 为占位实现，需启用对应 feature，建议使用 WebDAV / FTP")
            } else {
                HStack(spacing: NghSpacing.s2) {
                    Button { goProtocolUp() } label: { Image(systemName: "chevron.left") }
                        .disabled(protocolPath == "/" || protocolPath.isEmpty)
                    Text(protocolPath).font(.subheadline).foregroundColor(.nghTextSecondary).lineLimit(1)
                    Spacer()
                    Button { Task { await refreshProtocolEntries() } } label: {
                        Image(systemName: "arrow.clockwise")
                    }
                }
                if protocolLoading {
                    ProgressView().padding(.vertical, NghSpacing.s2)
                }
                if protocolEntries.isEmpty {
                    EmptyState(text: "未列出条目，点击刷新")
                } else {
                    VStack(spacing: 0) {
                        ForEach(protocolEntries, id: \.self) { name in
                            protocolRow(name)
                            Divider().background(Color.nghBorderSoft)
                        }
                    }
                    .background(Color.nghSurface)
                    .cornerRadius(NghRadius.md)
                    .nghCardShadow()
                }
            }
        }
        .padding(NghSpacing.s3)
        .background(Color.nghSurface)
        .cornerRadius(NghRadius.md)
    }

    private func protocolRow(_ rawName: String) -> some View {
        let isDir = rawName.hasSuffix("/")
        let name = isDir ? String(rawName.dropLast()) : rawName
        return Button {
            handleProtocolTap(name: name, isDir: isDir)
        } label: {
            HStack(spacing: NghSpacing.s3) {
                Image(systemName: isDir ? "folder" : "music.note")
                    .foregroundColor(.nghPrimary)
                    .frame(width: 22)
                Text(name).font(.body).foregroundColor(.nghText).lineLimit(1)
                Spacer()
            }
            .padding(.horizontal, NghSpacing.s3)
            .padding(.vertical, NghSpacing.s2)
        }
        .buttonStyle(.plain)
        .nghPressableStyle()
    }

    private var addSourceSheet: some View {
        NavigationStack {
            VStack(alignment: .leading, spacing: NghSpacing.s3) {
                Text("粘贴协议源配置 JSON（参考 docs/api/protocol-api.md）")
                    .font(.caption).foregroundColor(.nghTextSecondary)
                TextEditor(text: $newSourceJson)
                    .font(.system(.body, design: .monospaced))
                    .frame(minHeight: 220)
                    .padding(NghSpacing.s2)
                    .background(Color.nghSurfaceAlt)
                    .cornerRadius(NghRadius.md)
                Spacer()
            }
            .padding()
            .navigationTitle("添加协议源")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("取消") { showAddSourceSheet = false; newSourceJson = "" }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("添加") {
                        Task { await addSource() }
                    }
                    .disabled(newSourceJson.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
            }
        }
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
        await MainActor.run {
            healthText = "检测中…"
            healthOk = nil
        }
        do {
            let resp = try await retry { try await MusicCoreBridge.feiniuHealth() }
            await MainActor.run {
                healthOk = resp.healthy
                healthText = resp.healthy ? "飞牛服务可达" : "飞牛服务不可达"
            }
        } catch {
            await MainActor.run {
                healthOk = false
                healthText = "健康检查失败"
                errorMessage = formatError(error)
            }
        }
    }

    private func login() async {
        let base = baseUrl.trimmingCharacters(in: .whitespacesAndNewlines)
        let user = username.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !base.isEmpty, !user.isEmpty else {
            await MainActor.run { errorMessage = "请填写服务地址与用户名" }
            return
        }
        await MainActor.run { feiniuLoading = true }
        do {
            _ = try await MusicCoreBridge.feiniuLogin(baseUrl: base, username: user, password: password)
            await MainActor.run {
                feiniuPath = "/"
                infoMessage = "登录成功"
            }
            await refreshFeiniuFiles()
        } catch {
            await MainActor.run {
                if isStatus(error, 401) {
                    errorMessage = "用户名或密码错误（401），请检查凭据后重试。"
                } else {
                    errorMessage = "登录失败：" + formatError(error)
                }
            }
        }
        await MainActor.run { feiniuLoading = false }
    }

    private func refreshFeiniuFiles() async {
        await MainActor.run { feiniuLoading = true }
        let path = feiniuPath
        do {
            let files = try await retry { try await MusicCoreBridge.feiniuListFiles(path: path) }
            let sorted = files.sorted { a, b in
                if a.isDir != b.isDir { return a.isDir && !b.isDir }
                return a.name < b.name
            }
            await MainActor.run { feiniuFiles = sorted }
        } catch {
            await MainActor.run {
                feiniuFiles = []
                errorMessage = "列目录失败：" + formatError(error)
            }
        }
        await MainActor.run { feiniuLoading = false }
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
        await MainActor.run { feiniuLoading = true }
        let full = joinPath(parent: feiniuPath, name: file.name, isDir: false)
        do {
            let url = try await retry { try await MusicCoreBridge.feiniuStream(path: full) }
            guard !url.isEmpty else {
                await MainActor.run {
                    errorMessage = "未获取到播放地址"
                    feiniuLoading = false
                }
                return
            }
            // IOS-014 修复：Song id 拼入完整路径，防止不同目录下同名文件 id 冲突。
            let song = Song(id: "feiniu-" + full, sourceId: "feiniu", title: file.name,
                            artists: [], album: nil, coverUrl: nil, durationMs: nil, lyricUrl: nil,
                            playUrl: url, localPath: nil,
                            origin: .nas(protocolName: "feiniu", url: url))
            await MainActor.run {
                player.play(song: song)
                infoMessage = "开始播放：" + file.name
            }
        } catch {
            await MainActor.run {
                errorMessage = "获取播放地址失败：" + formatError(error)
            }
        }
        await MainActor.run { feiniuLoading = false }
    }

    private func goFeiniuUp() {
        feiniuPath = parentPath(feiniuPath)
        Task { await refreshFeiniuFiles() }
    }

    // MARK: - 行为：协议源

    private func loadProtocolSources() async {
        do {
            let list = try await MusicCoreBridge.protocolList()
            await MainActor.run { protocolSources = list }
        } catch {
            await MainActor.run { errorMessage = "加载协议源失败：" + formatError(error) }
        }
    }

    private func addSource() async {
        let json = newSourceJson.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !json.isEmpty else { return }
        do {
            _ = try await MusicCoreBridge.protocolAdd(configJson: json)
            await MainActor.run {
                showAddSourceSheet = false
                newSourceJson = ""
            }
            await loadProtocolSources()
            await MainActor.run { infoMessage = "协议源已添加" }
        } catch {
            await MainActor.run { errorMessage = "添加协议源失败：" + formatError(error) }
        }
    }

    private func deleteSource(_ src: ProtocolSource) async {
        do {
            try await MusicCoreBridge.protocolDelete(id: src.id)
            if selectedSourceId == src.id { await MainActor.run { selectedSourceId = nil } }
            await loadProtocolSources()
            await MainActor.run { infoMessage = "已删除协议源 " + src.id }
        } catch {
            await MainActor.run { errorMessage = "删除失败：" + formatError(error) }
        }
    }

    private func refreshProtocolEntries() async {
        guard let id = selectedSourceId, !id.isEmpty else { return }
        await MainActor.run { protocolLoading = true }
        let path = protocolPath
        do {
            let entries = try await retry { try await MusicCoreBridge.protocolListFiles(id: id, path: path) }
            let sorted = entries.sorted { a, b in
                let aDir = a.hasSuffix("/")
                let bDir = b.hasSuffix("/")
                if aDir != bDir { return aDir && !bDir }
                return a < b
            }
            await MainActor.run { protocolEntries = sorted }
        } catch {
            await MainActor.run {
                protocolEntries = []
                errorMessage = "浏览失败：" + formatError(error)
            }
        }
        await MainActor.run { protocolLoading = false }
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
        await MainActor.run { protocolLoading = true }
        let full = joinPath(parent: protocolPath, name: name, isDir: false)
        do {
            let url = try await retry { try await MusicCoreBridge.protocolStream(id: id, path: full) }
            guard !url.isEmpty else {
                await MainActor.run {
                    errorMessage = "未获取到播放地址"
                    protocolLoading = false
                }
                return
            }
            // IOS-014 修复：Song id 拼入完整路径，防止不同目录下同名文件 id 冲突。
            let song = Song(id: "proto-" + full, sourceId: "protocol", title: name,
                            artists: [], album: nil, coverUrl: nil, durationMs: nil, lyricUrl: nil,
                            playUrl: url, localPath: nil,
                            origin: .nas(protocolName: "protocol", url: url))
            await MainActor.run {
                player.play(song: song)
                infoMessage = "开始播放：" + name
            }
        } catch {
            await MainActor.run {
                errorMessage = "获取播放地址失败：" + formatError(error)
            }
        }
        await MainActor.run { protocolLoading = false }
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
                // IOS-015 修复：使用 try 而非 try?，保留 CancellationError 传播语义，
                // 使上层 Task 取消时能立即终止重试。
                try await Task.sleep(nanoseconds: delays[attempt])
            }
        }
        throw last ?? MusicCoreBridgeError.message("重试后仍失败")
    }

    private func isRetryable(_ error: Error) -> Bool {
        let msg = error.localizedDescription
        return msg.contains("502") || msg.contains("504") ||
               msg.contains("不可达") || msg.contains("请求失败")
    }

    private func isStatus(_ error: Error, _ status: Int) -> Bool {
        error.localizedDescription.contains(String(status))
    }

    private func formatError(_ error: Error) -> String {
        let msg = error.localizedDescription
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
    NasView().environmentObject(PlayerManager())
}
