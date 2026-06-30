// MARK: - MusicPlayerApp
// 职责：iOS 应用入口（@main），提供 WindowGroup 并挂载 ContentView。
//
// 集成方式（在 Xcode 中编译运行）：
// 1. 用 Xcode 新建 iOS App 工程（SwiftUI 生命周期，命名 MusicPlayer），
//    Bundle Identifier 填 com.musicplayer.app。
// 2. 将本目录 ios/MusicPlayerApp 下所有 .swift 源文件导入工程（拖入勾选 Copy items if needed）。
// 3. 链接 music-core 静态库：
//    a) 在 core/ 下执行 `cargo build --release --target aarch64-apple-ios`（或 x86_64-apple-ios）生成
//       target/<triple>/release/libmusic_core.a；
//    b) 在 core/ 下执行 `cargo install uniffi-bindgen` 后
//       `uniffi-bindgen generate src/ffi.udl --language swift --out-dir <工程>/Generated`
//       得到 MusicCoreFFI.swift + music_coreFFI.h + music_coreFFI.modulemap；
//    c) 在 Xcode → Target → Build Phases → Link Binary With Libraries 添加 libmusic_core.a；
//    d) Build Settings → Framework Search Paths / Library Search Paths 添加 .a 与 .modulemap 路径；
//    e) Other Linker Flags 加 -lmusic_core（如使用静态库则无需）。
// 4. 运行：选择真机/模拟器，Cmd+R。
//
// 备注：本脚手架仅提供源码骨架，未含 .xcodeproj（需在 Xcode 新建工程后导入源文件）。

import SwiftUI

@main
struct MusicPlayerApp: App {
    @StateObject private var player = PlayerManager()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(player)
        }
    }
}
