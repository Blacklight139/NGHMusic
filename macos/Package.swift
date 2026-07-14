// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "MusicPlayerApp",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .executable(name: "MusicPlayerApp", targets: ["MusicPlayerApp"])
    ],
    targets: [
        .executableTarget(
            name: "MusicPlayerApp",
            path: ".",
            exclude: [
                // 排除非源码资源
                "module.modulemap",
                "Services/module.modulemap",
                ".swiftlint.yml"
            ],
            sources: [
                "MusicPlayerAppApp.swift",
                "ContentView.swift",
                "Models",
                "Services",
                "Theme",
                "Views"
            ],
            resources: [],
            swiftSettings: [
                .unsafeFlags(["-parse-as-library"])
            ],
            linkerSettings: [
                // 链接 AVFoundation 用于 AVPlayer 播放
                .linkedFramework("AVFoundation"),
                .linkedFramework("AVFAudio"),
                .linkedFramework("AppKit"),
                // 链接 music_core cdylib（C ABI 实现于 core/src/ffi.rs）。
                // CI/构建环境若无 libmusic_core，可暂时注释；运行时仍需 dylib 在 dyld 搜索路径。
                .linkedLibrary("music_core")
            ]
        )
    ]
)
