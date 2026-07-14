import SwiftUI

@main
struct MusicPlayerAppApp: App {
    @StateObject private var player = PlayerService()

    var body: some Scene {
        WindowGroup("逆光音乐") {
            ContentView()
                .environmentObject(player)
                .frame(minWidth: 900, minHeight: 600)
        }
        .windowResizability(.contentMinSize)
        .commands {
            // 占位：后续可在此注册菜单命令（如音量调节、下一首等）。
        }
    }
}
