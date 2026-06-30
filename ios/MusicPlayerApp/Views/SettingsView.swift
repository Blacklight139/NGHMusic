// MARK: - SettingsView
// 职责：设置页，展示核心版本、音源导入（JSON）、缓存/目录管理，简约风格占位。
// 对齐桌面端 pages/settings.js：调用 MusicCore.importSource 导入音源 JSON。

import SwiftUI

struct SettingsView: View {
    @State private var sourceJson: String = ""
    @State private var resultMessage: String = ""
    @State private var resultOk = false
    @State private var appVersion = "（未连接）"

    var body: some View {
        PageContainer(title: "设置", subtitle: "音源管理 / 缓存 / 本地目录") {
            VStack(alignment: .leading, spacing: AppTheme.space4) {
                // 版本
                HStack {
                    Text("核心版本").font(.subheadline).foregroundColor(AppTheme.textMuted)
                    Spacer()
                    Text(appVersion).font(.subheadline).foregroundColor(AppTheme.text)
                }

                // 音源导入
                VStack(alignment: .leading, spacing: AppTheme.space2) {
                    Text("音源导入").font(.subheadline).fontWeight(.medium)
                    Text("粘贴标准音源 JSON Schema 内容").font(.caption)
                        .foregroundColor(AppTheme.textMuted)
                    TextEditor(text: $sourceJson)
                        .font(.system(size: 12, design: .monospaced))
                        .frame(minHeight: 160)
                        .padding(AppTheme.space2)
                        .background(AppTheme.bg)
                        .overlay(RoundedRectangle(cornerRadius: AppTheme.radius)
                            .stroke(AppTheme.border, lineWidth: 1))
                        .cornerRadius(AppTheme.radius)
                    HStack {
                        Button(action: importSource) {
                            Text("导入")
                                .foregroundColor(.white)
                                .padding(.horizontal, AppTheme.space4)
                                .padding(.vertical, AppTheme.space3)
                                .background(AppTheme.primary)
                                .cornerRadius(AppTheme.radius)
                        }
                        Spacer()
                    }
                    if !resultMessage.isEmpty {
                        Text(resultMessage)
                            .font(.caption)
                            .foregroundColor(resultOk ? AppTheme.primary : .red)
                            .padding(AppTheme.space3)
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(AppTheme.bg)
                            .overlay(RoundedRectangle(cornerRadius: AppTheme.radius)
                                .stroke(resultOk ? AppTheme.primary : Color(hex: 0xe57373), lineWidth: 1))
                            .cornerRadius(AppTheme.radius)
                    }
                }

                // 本地目录占位
                VStack(alignment: .leading, spacing: AppTheme.space2) {
                    Text("本地音乐目录").font(.subheadline).fontWeight(.medium)
                    EmptyState(text: "暂无目录，将在此添加 / 移除本地目录并触发扫描")
                }

                // 缓存占位
                VStack(alignment: .leading, spacing: AppTheme.space2) {
                    Text("播放缓存").font(.subheadline).fontWeight(.medium)
                    EmptyState(text: "缓存容量与清理功能将在此提供")
                }
            }
            .onAppear(perform: loadVersion)
        }
    }

    private func loadVersion() {
        appVersion = MusicCore.appVersion()
    }

    private func importSource() {
        guard !sourceJson.isEmpty else {
            resultMessage = "请先粘贴音源 JSON"
            resultOk = false
            return
        }
        Task {
            do {
                let resp = try await MusicCore.importSource(sourceJson)
                resultMessage = resp
                resultOk = true
            } catch {
                resultMessage = "（占位）导入失败：\(error.localizedDescription)"
                resultOk = false
            }
        }
    }
}
