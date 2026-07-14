using Microsoft.UI.Xaml;
using MusicPlayerApp.Services;

namespace MusicPlayerApp;

/// <summary>
/// 应用入口：注册全局服务（CoreService、PlayerService）并激活主窗口。
/// </summary>
public partial class App : Application
{
    private Window? mainWindow;

    /// <summary>全局播放服务实例（页面通过 App.Player 访问）。</summary>
    public static PlayerService Player { get; } = new PlayerService();

    public App()
    {
        InitializeComponent();
    }

    protected override void OnLaunched(LaunchActivatedEventArgs args)
    {
        // 后台初始化核心（音源管理器、缓存、本地索引库）
        _ = CoreService.Instance.InitializeAsync();
        mainWindow = new MainWindow();
        mainWindow.Activate();
    }
}
