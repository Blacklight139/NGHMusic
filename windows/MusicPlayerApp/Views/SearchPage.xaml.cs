using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Input;
using MusicPlayerApp.Models;
using MusicPlayerApp.Services;

namespace MusicPlayerApp.Views;

/// <summary>搜索页：通过 CoreService 进行聚合搜索并展示结果。</summary>
public sealed partial class SearchPage : Page
{
    private readonly CoreService _core = CoreService.Instance;
    private readonly PlayerService _player;

    public ObservableCollection<SongViewModel> Songs { get; } = new();

    private string _keyword = string.Empty;
    private uint _page = 1;
    private const uint PageSize = 20;
    private ulong _total;

    public SearchPage()
    {
        InitializeComponent();
        _player = App.Player;
        UpdatePageInfo();
    }

    private async void SearchButton_Click(object sender, RoutedEventArgs e)
    {
        await DoSearchAsync();
    }

    private async void KeywordBox_KeyDown(object sender, KeyRoutedEventArgs e)
    {
        if (e.Key == Windows.System.VirtualKey.Enter)
        {
            await DoSearchAsync();
        }
    }

    private async Task DoSearchAsync()
    {
        var keyword = KeywordBox.Text?.Trim() ?? string.Empty;
        if (string.IsNullOrEmpty(keyword))
        {
            return;
        }
        _keyword = keyword;
        _page = 1;
        await RunSearchAsync();
    }

    private async Task RunSearchAsync()
    {
        LoadingRing.IsActive = true;
        SearchButton.IsEnabled = false;
        try
        {
            var result = await _core.SearchAsync(_keyword, _page, PageSize);
            Songs.Clear();
            foreach (var s in result.Songs)
            {
                Songs.Add(new SongViewModel(s));
            }
            _total = result.Total;
            UpdatePageInfo();
        }
        catch (Exception ex)
        {
            await ShowErrorAsync($"搜索失败: {ex.Message}");
        }
        finally
        {
            LoadingRing.IsActive = false;
            SearchButton.IsEnabled = true;
        }
    }

    private void ResultsList_ItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is SongViewModel vm)
        {
            _player.PlaySingle(vm.Source);
        }
    }

    private void ResultsList_DoubleTapped(object sender, DoubleTappedRoutedEventArgs e)
    {
        // 占位：双击可加入播放列表（暂以单击播放为主）
    }

    private async void PrevPageButton_Click(object sender, RoutedEventArgs e)
    {
        if (_page <= 1)
        {
            return;
        }
        _page--;
        await RunSearchAsync();
    }

    private async void NextPageButton_Click(object sender, RoutedEventArgs e)
    {
        if ((_page * PageSize) >= _total)
        {
            return;
        }
        _page++;
        await RunSearchAsync();
    }

    private void UpdatePageInfo()
    {
        PageInfo.Text = Songs.Count > 0
            ? $"第 {_page} 页 / 共 {_total} 条"
            : "无结果";
    }

    private async System.Threading.Tasks.Task ShowErrorAsync(string message)
    {
        var dialog = new ContentDialog
        {
            XamlRoot = XamlRoot,
            Title = "提示",
            Content = message,
            CloseButtonText = "确定",
        };
        await dialog.ShowAsync();
    }
}

/// <summary>歌曲 ViewModel（用于 UI 绑定）。</summary>
public sealed class SongViewModel
{
    public Song Source { get; }
    public string Id => Source.Id;
    public string Title => Source.Title;
    public string ArtistsText => Source.Artists.Count > 0 ? string.Join(" / ", Source.Artists) : "未知艺术家";
    public string DurationText => Source.DurationMs.HasValue
        ? TimeSpan.FromMilliseconds(Source.DurationMs.Value).ToString(@"mm\:ss")
        : "--:--";

    public SongViewModel(Song source)
    {
        Source = source;
    }
}
