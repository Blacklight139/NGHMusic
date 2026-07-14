using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using MusicPlayerApp.Views;

namespace MusicPlayerApp;

/// <summary>
/// 主窗口：承载 NavigationView、内容 Frame 与底部 PlaybackBar。
/// </summary>
public sealed partial class MainWindow : Window
{
    public Frame Frame => ContentFrame;

    public MainWindow()
    {
        InitializeComponent();
        Title = "逆光音乐";
        // 默认导航到搜索页
        NavigateTo("Search");
        NavView.SelectedItem = NavView.MenuItems[0];
    }

    private void NavView_SelectionChanged(NavigationView sender, NavigationViewSelectionChangedEventArgs args)
    {
        if (args.SelectedItem is NavigationViewItem item && item.Tag is string tag)
        {
            NavigateTo(tag);
        }
    }

    /// <summary>按 tag 名称导航至对应页面。</summary>
    public void NavigateTo(string tag)
    {
        var pageType = tag switch
        {
            "Search" => typeof(SearchPage),
            "Playlist" => typeof(PlaylistPage),
            "Favorites" => typeof(FavoritesPage),
            "Lyrics" => typeof(LyricsPage),
            "Leaderboard" => typeof(LeaderboardPage),
            "LocalMusic" => typeof(LocalMusicPage),
            "Nas" => typeof(NasPage),
            "Settings" => typeof(SettingsPage),
            _ => typeof(SearchPage),
        };
        ContentFrame.Navigate(pageType, null, new Microsoft.UI.Xaml.Media.Animation.EntranceNavigationTransitionInfo());
    }
}
