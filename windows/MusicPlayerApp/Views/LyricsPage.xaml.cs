using System;
using System.Collections.Generic;
using System.Linq;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using Microsoft.UI.Xaml.Media;
using MusicPlayerApp.Models;
using MusicPlayerApp.Services;

namespace MusicPlayerApp.Views;

/// <summary>歌词页：跟随当前播放歌曲自动加载歌词并按时间戳滚动。</summary>
public sealed partial class LyricsPage : Page
{
    private readonly PlayerService _player;
    private readonly CoreService _core = CoreService.Instance;
    private List<LyricLine> _lines = new();
    private int _currentIndex = -1;

    public LyricsPage()
    {
        InitializeComponent();
        _player = App.Player;
        _player.CurrentSongChanged += OnCurrentSongChanged;
        _player.PositionChanged += OnPositionChanged;
        // 初始化展示
        OnCurrentSongChanged(this, _player.CurrentSong);
    }

    private void OnCurrentSongChanged(object? sender, Song? song)
    {
        _ = DispatcherQueue.TryEnqueue(async () =>
        {
            if (song is null)
            {
                CurrentTitleText.Text = "尚未播放";
                CurrentArtistText.Text = string.Empty;
                RenderEmpty();
                return;
            }
            CurrentTitleText.Text = song.Title;
            CurrentArtistText.Text = string.Join(" / ", song.Artists);
            await LoadLyricAsync(song);
        });
    }

    private async System.Threading.Tasks.Task LoadLyricAsync(Song song)
    {
        _lines.Clear();
        _currentIndex = -1;
        if (string.IsNullOrEmpty(song.SourceId) || string.IsNullOrEmpty(song.Id))
        {
            RenderEmpty();
            return;
        }
        try
        {
            var lyric = await _core.GetLyricAsync(song.SourceId, song.Id);
            _lines = lyric.Lines;
            RenderLyric();
        }
        catch (Exception ex)
        {
            RenderEmpty($"歌词加载失败: {ex.Message}");
        }
    }

    private void OnPositionChanged(object? sender, TimeSpan position)
    {
        _ = DispatcherQueue.TryEnqueue(() => UpdateHighlight(position));
    }

    private void UpdateHighlight(TimeSpan position)
    {
        if (_lines.Count == 0)
        {
            return;
        }
        var ms = (ulong)position.TotalMilliseconds;
        int idx = -1;
        for (var i = 0; i < _lines.Count; i++)
        {
            if (_lines[i].TimeMs is { } t && t <= ms)
            {
                idx = i;
            }
            else if (_lines[i].TimeMs is null)
            {
                break;
            }
        }
        if (idx == _currentIndex || idx < 0)
        {
            return;
        }
        _currentIndex = idx;
        for (var i = 0; i < LyricPanel.Children.Count; i++)
        {
            if (LyricPanel.Children[i] is TextBlock tb)
            {
                tb.FontWeight = i == idx ? Microsoft.UI.Text.FontWeights.SemiBold : Microsoft.UI.Text.FontWeights.Normal;
                tb.Foreground = i == idx
                    ? (Brush)Application.Current.Resources["AccentTextFillColorPrimaryBrush"]
                    : (Brush)Application.Current.Resources["TextFillColorPrimaryBrush"];
                tb.FontSize = i == idx ? 18 : 14;
            }
        }
        // 自动滚动至当前行
        if (_currentIndex >= 0 && _currentIndex < LyricPanel.Children.Count)
        {
            var child = LyricPanel.Children[_currentIndex];
            var transform = child.TransformToVisual(LyricPanel);
            var positionInPanel = transform.TransformPoint(new Windows.Foundation.Point(0, 0));
            LyricScroll.ChangeView(null, positionInPanel.Y - 80, null);
        }
    }

    private void RenderEmpty(string hint = "播放任意歌曲后此处将显示歌词")
    {
        LyricPanel.Children.Clear();
        LyricPanel.Children.Add(new TextBlock
        {
            Text = hint,
            FontSize = 14,
            FontStyle = Windows.UI.Text.FontStyle.Italic,
            Foreground = (Brush)Application.Current.Resources["TextFillColorSecondaryBrush"],
        });
    }

    private void RenderLyric()
    {
        LyricPanel.Children.Clear();
        foreach (var line in _lines)
        {
            LyricPanel.Children.Add(new TextBlock
            {
                Text = line.Text,
                FontSize = 14,
                TextWrapping = TextWrapping.Wrap,
                Padding = new Windows.UI.Xaml.Thickness(0, 4, 0, 4),
            });
        }
    }
}
