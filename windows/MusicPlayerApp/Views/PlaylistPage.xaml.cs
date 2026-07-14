using System.Collections.ObjectModel;
using System.Linq;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using MusicPlayerApp.Models;
using MusicPlayerApp.Services;

namespace MusicPlayerApp.Views;

/// <summary>播放列表页：展示当前播放队列，支持单首切换/移除/清空。</summary>
public sealed partial class PlaylistPage : Page
{
    private readonly PlayerService _player;
    public ObservableCollection<SongViewModel> Songs { get; } = new();

    public PlaylistPage()
    {
        InitializeComponent();
        _player = App.Player;
        RefreshList();
        _player.CurrentSongChanged += OnPlayerChanged;
    }

    private void OnPlayerChanged(object? sender, Song? song)
    {
        _ = DispatcherQueue.TryEnqueue(RefreshList);
    }

    private void RefreshList()
    {
        Songs.Clear();
        foreach (var s in _player.Queue)
        {
            Songs.Add(new SongViewModel(s));
        }
        CountText.Text = $"共 {Songs.Count} 首";
    }

    private void PlaylistView_ItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is SongViewModel vm)
        {
            var idx = _player.Queue.ToList().IndexOf(vm.Source);
            if (idx >= 0)
            {
                _player.PlayAll(_player.Queue, idx);
            }
        }
    }

    private void RemoveItem_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is string id)
        {
            var queue = _player.Queue.ToList();
            var idx = queue.FindIndex(s => s.Id == id);
            if (idx >= 0)
            {
                _player.RemoveAt(idx);
                RefreshList();
            }
        }
    }

    private void ClearButton_Click(object sender, RoutedEventArgs e)
    {
        _player.ClearQueue();
        RefreshList();
    }
}
