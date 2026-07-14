using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using MusicPlayerApp.Models;
using MusicPlayerApp.Services;
using Windows.Storage;

namespace MusicPlayerApp.Views;

/// <summary>我喜欢页：本地收藏的歌曲列表，持久化到 LocalFolder/favorites.json。</summary>
public sealed partial class FavoritesPage : Page
{
    private readonly PlayerService _player;
    private readonly string _favoritesFile;
    private static readonly JsonSerializerOptions JsonOpts = new() { WriteIndented = true };

    public ObservableCollection<SongViewModel> Songs { get; } = new();
    private List<Song> _rawSongs = new();

    public FavoritesPage()
    {
        InitializeComponent();
        _player = App.Player;
        _favoritesFile = Path.Combine(ApplicationData.Current.LocalFolder.Path, "favorites.json");
        _ = LoadAsync();
    }

    private async Task LoadAsync()
    {
        try
        {
            if (!File.Exists(_favoritesFile))
            {
                UpdateCount();
                return;
            }
            var json = await File.ReadAllTextAsync(_favoritesFile);
            _rawSongs = JsonSerializer.Deserialize<List<Song>>(json, JsonOpts) ?? new();
            Songs.Clear();
            foreach (var s in _rawSongs)
            {
                Songs.Add(new SongViewModel(s));
            }
            UpdateCount();
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[FavoritesPage] 加载失败: {ex.Message}");
        }
    }

    private async Task SaveAsync()
    {
        try
        {
            var json = JsonSerializer.Serialize(_rawSongs, JsonOpts);
            await File.WriteAllTextAsync(_favoritesFile, json);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[FavoritesPage] 保存失败: {ex.Message}");
        }
    }

    /// <summary>外部调用：将歌曲加入收藏。</summary>
    public async Task AddAsync(Song song)
    {
        if (_rawSongs.Any(s => s.Id == song.Id && s.SourceId == song.SourceId))
        {
            return;
        }
        _rawSongs.Add(song);
        Songs.Add(new SongViewModel(song));
        await SaveAsync();
        UpdateCount();
    }

    private async void RemoveFavorite_Click(object sender, RoutedEventArgs e)
    {
        if (sender is Button btn && btn.Tag is string id)
        {
            var idx = _rawSongs.FindIndex(s => s.Id == id);
            if (idx >= 0)
            {
                _rawSongs.RemoveAt(idx);
                Songs.RemoveAt(idx);
                await SaveAsync();
                UpdateCount();
            }
        }
    }

    private void FavoritesList_ItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is SongViewModel vm)
        {
            _player.PlayAll(_rawSongs, _rawSongs.IndexOf(vm.Source));
        }
    }

    private void PlayAllButton_Click(object sender, RoutedEventArgs e)
    {
        if (_rawSongs.Count > 0)
        {
            _player.PlayAll(_rawSongs, 0);
        }
    }

    private void UpdateCount()
    {
        CountText.Text = $"共 {Songs.Count} 首";
    }
}
