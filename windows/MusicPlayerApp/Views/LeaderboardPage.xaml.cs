using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using MusicPlayerApp.Models;
using MusicPlayerApp.Services;

namespace MusicPlayerApp.Views;

/// <summary>排行榜页：选择音源后展示该音源提供的所有排行榜及其歌曲。</summary>
public sealed partial class LeaderboardPage : Page
{
    private readonly CoreService _core = CoreService.Instance;
    private readonly PlayerService _player;

    public ObservableCollection<Leaderboard> Leaderboards { get; } = new();

    public LeaderboardPage()
    {
        InitializeComponent();
        _player = App.Player;
        _ = LoadSourcesAsync();
    }

    private async Task LoadSourcesAsync()
    {
        try
        {
            var sources = await _core.ListSourcesAsync();
            SourceCombo.Items.Clear();
            foreach (var s in sources.Where(s => s.Enabled))
            {
                SourceCombo.Items.Add(new ComboBoxItem
                {
                    Content = s.Name,
                    Tag = s.Id,
                });
            }
            if (SourceCombo.Items.Count > 0)
            {
                SourceCombo.SelectedIndex = 0;
            }
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[LeaderboardPage] 加载音源失败: {ex.Message}");
        }
    }

    private async void SourceCombo_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (SourceCombo.SelectedItem is not ComboBoxItem item || item.Tag is not string sourceId)
        {
            return;
        }
        await LoadLeaderboardsAsync(sourceId);
    }

    private async Task LoadLeaderboardsAsync(string sourceId)
    {
        LoadingRing.IsActive = true;
        try
        {
            var list = await _core.GetLeaderboardsAsync(sourceId);
            Leaderboards.Clear();
            foreach (var lb in list)
            {
                Leaderboards.Add(lb);
            }
        }
        catch (Exception ex)
        {
            await ShowErrorAsync($"获取排行榜失败: {ex.Message}");
        }
        finally
        {
            LoadingRing.IsActive = false;
        }
    }

    private void LeaderboardsList_ItemClick(object sender, ItemClickEventArgs e)
    {
        // 点击展开/收起由 Expander 自身处理；这里仅占位
    }

    private void SongItem_Click(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is Song song)
        {
            _player.PlaySingle(song);
        }
    }

    private async Task ShowErrorAsync(string message)
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
