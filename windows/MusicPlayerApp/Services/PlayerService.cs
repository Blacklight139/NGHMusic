using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Threading;
using System.Threading.Tasks;
using MusicPlayerApp.Models;
using Windows.Media;
using Windows.Media.Playback;
using Windows.Storage;

namespace MusicPlayerApp.Services;

/// <summary>
/// 播放服务：在 <see cref="Windows.Media.Playback.MediaPlayer"/> 之上封装
/// 播放队列、播放模式与状态持久化。所有 UI 绑定通过事件通知。
/// </summary>
public sealed class PlayerService : IDisposable
{
    private readonly MediaPlayer _player = new();
    private readonly List<Song> _queue = new();
    private int _currentIndex = -1;
    private PlayMode _mode = PlayMode.Sequential;
    private readonly string _stateFilePath;
    private CancellationTokenSource? _positionTimerCts;
    private bool _disposed;

    /// <summary>当前播放队列（只读视图）。</summary>
    public IReadOnlyList<Song> Queue => _queue;

    /// <summary>当前播放索引（-1 表示未选中）。</summary>
    public int CurrentIndex => _currentIndex;

    /// <summary>当前播放歌曲（可能为 null）。</summary>
    public Song? CurrentSong => _currentIndex >= 0 && _currentIndex < _queue.Count ? _queue[_currentIndex] : null;

    /// <summary>播放模式。</summary>
    public PlayMode Mode
    {
        get => _mode;
        set
        {
            if (_mode != value)
            {
                _mode = value;
                ModeChanged?.Invoke(this, value);
                _ = PersistAsync();
            }
        }
    }

    /// <summary>音量（0~1）。</summary>
    public double Volume
    {
        get => _player.Volume;
        set
        {
            _player.Volume = Math.Clamp(value, 0, 1);
            VolumeChanged?.Invoke(this, _player.Volume);
            _ = PersistAsync();
        }
    }

    /// <summary>当前是否正在播放。</summary>
    public bool IsPlaying => _player.PlaybackSession.PlaybackState == MediaPlaybackState.Playing;

    /// <summary>当前播放位置（秒）。</summary>
    public TimeSpan Position => _player.PlaybackSession.Position;

    /// <summary>当前媒体总时长。</summary>
    public TimeSpan NaturalDuration => _player.PlaybackSession.NaturalDuration;

    public event EventHandler<Song?>? CurrentSongChanged;
    public event EventHandler<bool>? IsPlayingChanged;
    public event EventHandler<TimeSpan>? PositionChanged;
    public event EventHandler<PlayMode>? ModeChanged;
    public event EventHandler<double>? VolumeChanged;
    public event EventHandler? MediaEnded;

    public PlayerService()
    {
        _stateFilePath = Path.Combine(ApplicationData.Current.LocalFolder.Path, "player_state.json");
        _player.AudioCategory = MediaPlayerAudioCategory.Media;
        _player.PlaybackSession.PositionChanged += OnPlaybackPositionChanged;
        _player.PlaybackSession.PlaybackStateChanged += OnPlaybackStateChanged;
        _player.MediaEnded += OnMediaEnded;
        _player.MediaFailed += OnMediaFailed;

        _ = LoadAsync();
        StartPositionTimer();
    }

    // =============================================================
    // 队列管理
    // =============================================================

    /// <summary>替换整个播放队列并以 index 开始播放。</summary>
    public void PlayAll(IEnumerable<Song> songs, int startIndex = 0)
    {
        _queue.Clear();
        _queue.AddRange(songs);
        _currentIndex = Math.Clamp(startIndex, 0, Math.Max(0, _queue.Count - 1));
        CurrentSongChanged?.Invoke(this, CurrentSong);
        _ = PlayCurrentAsync();
    }

    /// <summary>追加到队列末尾（不切换当前播放）。</summary>
    public void Enqueue(IEnumerable<Song> songs)
    {
        var wasEmpty = _queue.Count == 0;
        _queue.AddRange(songs);
        if (wasEmpty && _queue.Count > 0)
        {
            _currentIndex = 0;
            CurrentSongChanged?.Invoke(this, CurrentSong);
            _ = PlayCurrentAsync();
        }
    }

    /// <summary>立即播放单首歌曲（清空并替换队列）。</summary>
    public void PlaySingle(Song song)
    {
        _queue.Clear();
        _queue.Add(song);
        _currentIndex = 0;
        CurrentSongChanged?.Invoke(this, CurrentSong);
        _ = PlayCurrentAsync();
    }

    public void ClearQueue()
    {
        _player.Pause();
        var oldSource = _player.Source;
        _player.Source = null;
        oldSource?.Dispose();
        _queue.Clear();
        _currentIndex = -1;
        CurrentSongChanged?.Invoke(this, null);
        _ = PersistAsync();
    }

    public void RemoveAt(int index)
    {
        if (index < 0 || index >= _queue.Count)
        {
            return;
        }
        _queue.RemoveAt(index);
        if (index < _currentIndex)
        {
            _currentIndex--;
        }
        else if (index == _currentIndex)
        {
            if (_currentIndex >= _queue.Count)
            {
                _currentIndex = _queue.Count - 1;
            }
            _ = PlayCurrentAsync();
        }
        CurrentSongChanged?.Invoke(this, CurrentSong);
    }

    // =============================================================
    // 播放控制
    // =============================================================

    public void Play()
    {
        if (_player.Source is null && _queue.Count > 0 && _currentIndex < 0)
        {
            _currentIndex = 0;
        }
        if (_player.Source is null)
        {
            _ = PlayCurrentAsync();
            return;
        }
        _player.Play();
    }

    public void Pause() => _player.Pause();

    public void TogglePlayPause()
    {
        if (IsPlaying)
        {
            Pause();
        }
        else
        {
            Play();
        }
    }

    public void Next()
    {
        if (_queue.Count == 0)
        {
            return;
        }
        _currentIndex = _mode switch
        {
            PlayMode.Random => RandomNext(),
            _ => (_currentIndex + 1) % _queue.Count,
        };
        CurrentSongChanged?.Invoke(this, CurrentSong);
        _ = PlayCurrentAsync();
    }

    public void Previous()
    {
        if (_queue.Count == 0)
        {
            return;
        }
        _currentIndex = _mode switch
        {
            PlayMode.Random => RandomNext(),
            _ => (_currentIndex - 1 + _queue.Count) % _queue.Count,
        };
        CurrentSongChanged?.Invoke(this, CurrentSong);
        _ = PlayCurrentAsync();
    }

    /// <summary>跳转播放进度（秒）。</summary>
    public void Seek(TimeSpan position)
    {
        _player.PlaybackSession.Position = position;
    }

    /// <summary>循环切换播放模式。</summary>
    public PlayMode CycleMode()
    {
        Mode = _mode switch
        {
            PlayMode.Sequential => PlayMode.SingleLoop,
            PlayMode.SingleLoop => PlayMode.Random,
            _ => PlayMode.Sequential,
        };
        return Mode;
    }

    // =============================================================
    // 内部播放逻辑
    // =============================================================

    private async Task PlayCurrentAsync()
    {
        if (_currentIndex < 0 || _currentIndex >= _queue.Count)
        {
            var staleSource = _player.Source;
            _player.Source = null;
            staleSource?.Dispose();
            return;
        }

        // 记录请求时的索引，await 恢复后检查是否仍为当前索引，
        // 避免快速切歌时旧请求覆盖新请求（竞态条件）。
        var requestedIndex = _currentIndex;
        var song = _queue[requestedIndex];
        try
        {
            var url = await ResolvePlayableUrlAsync(song);
            // await 期间索引可能已被 Next/Previous 等改变，放弃本次过期的请求
            if (_currentIndex != requestedIndex)
            {
                return;
            }
            if (string.IsNullOrEmpty(url))
            {
                // 跳过无可用 URL 的曲目
                Next();
                return;
            }

            MediaSource? newSource;
            if (Uri.TryCreate(url, UriKind.Absolute, out var uri))
            {
                newSource = MediaSource.CreateFromUri(uri);
            }
            else
            {
                // 本地路径：尝试通过 StorageFile 打开
                try
                {
                    var file = await StorageFile.GetFileFromPathAsync(url);
                    var stream = await file.OpenAsync(FileAccessMode.Read);
                    newSource = MediaSource.CreateFromStream(stream, file.ContentType);
                }
                catch
                {
                    newSource = MediaSource.CreateFromUri(new Uri(url, UriKind.RelativeOrAbsolute));
                }
            }

            // 替换并释放旧的 MediaSource，避免原生资源泄漏
            var previousSource = _player.Source;
            _player.Source = newSource;
            previousSource?.Dispose();

            _player.Play();
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[PlayerService] 播放失败: {ex.Message}");
            if (_currentIndex == requestedIndex)
            {
                Next();
            }
        }

        _ = PersistAsync();
    }

    /// <summary>解析歌曲可播放 URL：play_url &gt; local_path &gt; OnlineOrigin.play_url。</summary>
    private static async Task<string?> ResolvePlayableUrlAsync(Song song)
    {
        if (!string.IsNullOrEmpty(song.PlayUrl))
        {
            return song.PlayUrl;
        }
        if (!string.IsNullOrEmpty(song.LocalPath))
        {
            return song.LocalPath;
        }
        if (song.Origin is OnlineOrigin online && !string.IsNullOrEmpty(online.PlayUrl))
        {
            return online.PlayUrl;
        }
        if (!string.IsNullOrEmpty(song.SourceId) && !string.IsNullOrEmpty(song.Id))
        {
            try
            {
                var result = await CoreService.Instance.GetPlayUrlAsync(song.SourceId, song.Id);
                return result.Url ?? result.PlayUrl;
            }
            catch
            {
                return null;
            }
        }
        return null;
    }

    private int RandomNext()
    {
        if (_queue.Count <= 1)
        {
            return 0;
        }
        var rnd = Random.Shared;
        int next;
        do
        {
            next = rnd.Next(_queue.Count);
        } while (next == _currentIndex);
        return next;
    }

    // =============================================================
    // 事件回调
    // =============================================================

    private void OnPlaybackStateChanged(MediaPlaybackSession sender, object args)
    {
        IsPlayingChanged?.Invoke(this, IsPlaying);
    }

    private void OnPlaybackPositionChanged(MediaPlaybackSession sender, object args)
    {
        PositionChanged?.Invoke(this, sender.Position);
    }

    private void OnMediaEnded(MediaPlayer sender, object args)
    {
        MediaEnded?.Invoke(this, EventArgs.Empty);
        if (_mode == PlayMode.SingleLoop && _currentIndex >= 0)
        {
            _ = PlayCurrentAsync();
            return;
        }

        // 顺序模式：播放到队列末尾时停止，不自动回绕到第一首。
        // Next() 使用取模回绕（% _queue.Count），若在末尾继续推进会从头重放，
        // 每首结束又触发 MediaEnded，从而形成无限循环重放整个队列。
        if (_mode == PlayMode.Sequential && _currentIndex >= _queue.Count - 1)
        {
            // 保留队列与索引，播放器自然停在末尾，用户可手动续播，避免 MediaEnded 反复触发。
            return;
        }

        Next();
    }

    private void OnMediaFailed(MediaPlayer sender, MediaPlayerFailedEventArgs args)
    {
        System.Diagnostics.Debug.WriteLine($"[PlayerService] 媒体失败: {args.Error} - {args.ErrorMessage}");
        // 跳到下一曲（避免卡死）
        Next();
    }

    private void StartPositionTimer()
    {
        _positionTimerCts = new CancellationTokenSource();
        var ct = _positionTimerCts.Token;
        _ = Task.Run(async () =>
        {
            using var timer = new PeriodicTimer(TimeSpan.FromMilliseconds(500));
            while (!ct.IsCancellationRequested)
            {
                try
                {
                    await timer.WaitForNextTickAsync(ct);
                    // Dispose 可能在此 tick 期间执行，检查标志避免访问已释放的播放器
                    if (_disposed) break;
                    if (IsPlaying)
                    {
                        PositionChanged?.Invoke(this, Position);
                    }
                }
                catch (OperationCanceledException)
                {
                    break;
                }
                catch
                {
                    // 播放器可能在 Dispose 过程中被释放，安全退出定时循环
                    break;
                }
            }
        }, ct);
    }

    // =============================================================
    // 持久化
    // =============================================================

    private PlayerStateDto BuildStateDto() => new()
    {
        Volume = _player.Volume,
        Mode = (int)_mode,
        Queue = _queue.Select(s => s.Id).ToList(),
        CurrentIndex = _currentIndex,
    };

    private async Task PersistAsync()
    {
        try
        {
            var dto = BuildStateDto();
            var json = JsonSerializer.Serialize(dto, StateJsonOptions);
            await File.WriteAllTextAsync(_stateFilePath, json);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[PlayerService] 持久化失败: {ex.Message}");
        }
    }

    private async Task LoadAsync()
    {
        try
        {
            if (!File.Exists(_stateFilePath))
            {
                return;
            }
            var json = await File.ReadAllTextAsync(_stateFilePath);
            var dto = JsonSerializer.Deserialize<PlayerStateDto>(json, StateJsonOptions);
            if (dto is null)
            {
                return;
            }
            _player.Volume = Math.Clamp(dto.Volume, 0, 1);
            _mode = (PlayMode)dto.Mode;
            ModeChanged?.Invoke(this, _mode);
            VolumeChanged?.Invoke(this, _player.Volume);
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[PlayerService] 加载状态失败: {ex.Message}");
        }
    }

    public void Dispose()
    {
        if (_disposed)
        {
            return;
        }
        _disposed = true;
        _positionTimerCts?.Cancel();
        _positionTimerCts?.Dispose();
        _player.PlaybackSession.PositionChanged -= OnPlaybackPositionChanged;
        _player.PlaybackSession.PlaybackStateChanged -= OnPlaybackStateChanged;
        _player.MediaEnded -= OnMediaEnded;
        _player.MediaFailed -= OnMediaFailed;
        // 释放当前 MediaSource 及播放器本身
        _player.Source?.Dispose();
        _player.Source = null;
        _player.Dispose();
    }

    private static readonly JsonSerializerOptions StateJsonOptions = new()
    {
        WriteIndented = false,
        DefaultIgnoreCondition = JsonIgnoreCondition.Never,
    };

    private sealed class PlayerStateDto
    {
        public double Volume { get; set; } = 0.8;
        public int Mode { get; set; }
        public List<string> Queue { get; set; } = new();
        public int CurrentIndex { get; set; } = -1;
    }
}
