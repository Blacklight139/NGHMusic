using System;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using MusicPlayerApp.Models;
using MusicPlayerApp.Services;

namespace MusicPlayerApp.Views;

/// <summary>底部播放控制条：播放/暂停、上一曲/下一曲、进度条、音量、播放模式。</summary>
public sealed partial class PlaybackBar : UserControl
{
    private readonly PlayerService _player;
    private bool _suppressSliderEvent;
    private bool _isMuted;
    private double _volumeBeforeMute = 0.8;

    public PlaybackBar()
    {
        InitializeComponent();
        _player = App.Player;
        _player.CurrentSongChanged += OnCurrentSongChanged;
        _player.IsPlayingChanged += OnIsPlayingChanged;
        _player.PositionChanged += OnPositionChanged;
        _player.ModeChanged += OnModeChanged;
        _player.VolumeChanged += OnVolumeChanged;
        Unloaded += OnControlUnloaded;

        VolumeSlider.Value = _player.Volume;
        UpdateModeIcon(_player.Mode);
        OnCurrentSongChanged(this, _player.CurrentSong);
    }

    private void OnControlUnloaded(object sender, RoutedEventArgs e)
    {
        // 控件从可视化树移除时取消订阅，避免 PlayerService 单例持有控件引用
        _player.CurrentSongChanged -= OnCurrentSongChanged;
        _player.IsPlayingChanged -= OnIsPlayingChanged;
        _player.PositionChanged -= OnPositionChanged;
        _player.ModeChanged -= OnModeChanged;
        _player.VolumeChanged -= OnVolumeChanged;
    }

    private void OnCurrentSongChanged(object? sender, Song? song)
    {
        _ = DispatcherQueue.TryEnqueue(() =>
        {
            if (song is null)
            {
                SongTitleText.Text = "未播放";
                SongArtistText.Text = string.Empty;
                DurationText.Text = "00:00";
                CurrentTimeText.Text = "00:00";
                ProgressSlider.Maximum = 1;
                _suppressSliderEvent = true;
                ProgressSlider.Value = 0;
                _suppressSliderEvent = false;
                return;
            }
            SongTitleText.Text = song.Title;
            SongArtistText.Text = song.Artists.Count > 0 ? string.Join(" / ", song.Artists) : "未知艺术家";
            var dur = song.DurationMs.HasValue
                ? TimeSpan.FromMilliseconds(song.DurationMs.Value)
                : _player.NaturalDuration;
            DurationText.Text = Format(dur);
            ProgressSlider.Maximum = dur.TotalSeconds > 0 ? dur.TotalSeconds : 1;
        });
    }

    private void OnIsPlayingChanged(object? sender, bool isPlaying)
    {
        _ = DispatcherQueue.TryEnqueue(() =>
        {
            PlayPauseIcon.Glyph = isPlaying ? "\uE769" : "\uE768";
        });
    }

    private void OnPositionChanged(object? sender, TimeSpan position)
    {
        _ = DispatcherQueue.TryEnqueue(() =>
        {
            CurrentTimeText.Text = Format(position);
            _suppressSliderEvent = true;
            try
            {
                ProgressSlider.Value = position.TotalSeconds;
            }
            finally
            {
                _suppressSliderEvent = false;
            }
        });
    }

    private void OnModeChanged(object? sender, PlayMode mode)
    {
        _ = DispatcherQueue.TryEnqueue(() => UpdateModeIcon(mode));
    }

    private void OnVolumeChanged(object? sender, double volume)
    {
        _ = DispatcherQueue.TryEnqueue(() =>
        {
            _suppressSliderEvent = true;
            try
            {
                VolumeSlider.Value = volume;
            }
            finally
            {
                _suppressSliderEvent = false;
            }
            UpdateVolumeIcon(volume);
        });
    }

    private void UpdateModeIcon(PlayMode mode)
    {
        ModeIcon.Glyph = mode switch
        {
            PlayMode.Sequential => "\uE1CE",   // Repeat（顺序）
            PlayMode.SingleLoop => "\uE1CD",   // RepeatOne
            PlayMode.Random => "\uE8B1",       // Shuffle
            _ => "\uE1CE",
        };
        ModeButton.ToolTipService.ToolTip = mode switch
        {
            PlayMode.Sequential => "顺序播放",
            PlayMode.SingleLoop => "单曲循环",
            PlayMode.Random => "随机播放",
            _ => "播放模式",
        };
    }

    private void UpdateVolumeIcon(double volume)
    {
        if (volume <= 0.001)
        {
            VolumeIcon.Glyph = "\uE74F"; // Mute
        }
        else
        {
            VolumeIcon.Glyph = "\uE767"; // Volume2（低/高音量统一使用该图标）
        }
    }

    private void PlayPauseButton_Click(object sender, RoutedEventArgs e)
    {
        _player.TogglePlayPause();
    }

    private void PrevButton_Click(object sender, RoutedEventArgs e)
    {
        _player.Previous();
    }

    private void NextButton_Click(object sender, RoutedEventArgs e)
    {
        _player.Next();
    }

    private void ModeButton_Click(object sender, RoutedEventArgs e)
    {
        _player.CycleMode();
    }

    private void ProgressSlider_ValueChanged(object sender, RangeBaseValueChangedEventArgs e)
    {
        if (_suppressSliderEvent)
        {
            return;
        }
        _player.Seek(TimeSpan.FromSeconds(ProgressSlider.Value));
    }

    private void VolumeSlider_ValueChanged(object sender, RangeBaseValueChangedEventArgs e)
    {
        if (_suppressSliderEvent)
        {
            return;
        }
        _player.Volume = VolumeSlider.Value;
        _isMuted = VolumeSlider.Value <= 0.001;
    }

    private void MuteButton_Click(object sender, RoutedEventArgs e)
    {
        if (_isMuted)
        {
            _player.Volume = _volumeBeforeMute > 0 ? _volumeBeforeMute : 0.5;
            _isMuted = false;
        }
        else
        {
            _volumeBeforeMute = _player.Volume;
            _player.Volume = 0;
            _isMuted = true;
        }
    }

    private static string Format(TimeSpan ts)
    {
        if (ts.TotalSeconds < 0)
        {
            ts = TimeSpan.Zero;
        }
        return ts.ToString(@"mm\:ss");
    }
}
