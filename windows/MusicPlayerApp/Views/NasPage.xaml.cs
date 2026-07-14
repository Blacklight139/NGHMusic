using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using MusicPlayerApp.Models;
using MusicPlayerApp.Services;

namespace MusicPlayerApp.Views;

/// <summary>
/// NAS / 协议音源浏览页：
/// - 飞牛 NAS：健康检查、登录、目录浏览、点击音频文件经 feiniu_stream 拉流播放。
/// - 协议源：选择已添加协议源，浏览目录，点击音频文件经 protocol_stream 拉流播放。
/// 协议源的添加 / 删除在设置页管理；本页仅做浏览与播放。
/// 重试策略遵循 docs：502/504 指数退避（1s/2s/4s，最多 3 次），401 提示重新登录，404 修正路径，501 占位提示。
/// </summary>
public sealed partial class NasPage : Page
{
    private readonly CoreService _core = CoreService.Instance;
    private readonly PlayerService _player;

    public ObservableCollection<NasFileItem> FeiniuFiles { get; } = new();
    public ObservableCollection<ProtocolEntry> ProtocolEntries { get; } = new();
    public ObservableCollection<ProtocolSourceInfo> ProtocolSources { get; } = new();

    private string _currentPath = "/";
    private string _protocolPath = "/";
    private string? _selectedProtocolId;

    public NasPage()
    {
        InitializeComponent();
        _player = App.Player;
        ModeSelector.SelectedIndex = 0;
        _ = InitializeAsync();
    }

    private async Task InitializeAsync()
    {
        try { await _core.InitializeAsync(); } catch { /* 忽略初始化失败 */ }
        await RefreshProtocolSourcesAsync();
    }

    private void ModeSelector_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        var tag = (ModeSelector.SelectedItem as SelectorItem)?.Tag as string;
        if (tag == "protocol")
        {
            FeiniuPanel.Visibility = Visibility.Collapsed;
            ProtocolPanel.Visibility = Visibility.Visible;
            _ = RefreshProtocolSourcesAsync();
        }
        else
        {
            FeiniuPanel.Visibility = Visibility.Visible;
            ProtocolPanel.Visibility = Visibility.Collapsed;
        }
    }

    // =============================================================
    // 飞牛 NAS
    // =============================================================

    private async void HealthCheckButton_Click(object sender, RoutedEventArgs e)
    {
        await CheckHealthAsync();
    }

    private async Task CheckHealthAsync()
    {
        HealthText.Text = "检测中…";
        HealthIcon.Glyph = "\uE9D9"; // 待定
        try
        {
            var ok = await RetryAsync(() => _core.FeiniuHealthAsync(), nameof(CheckHealthAsync));
            HealthIcon.Glyph = ok ? "\uEA18" : "\uE783"; // 健康对勾 / 错误
            HealthText.Text = ok ? "飞牛服务可达" : "飞牛服务不可达";
        }
        catch (Exception ex)
        {
            HealthIcon.Glyph = "\uE783";
            HealthText.Text = "健康检查失败";
            await ShowErrorAsync(FormatError(ex));
        }
    }

    private async void LoginButton_Click(object sender, RoutedEventArgs e)
    {
        var baseUrl = BaseUrlBox.Text?.Trim();
        var username = UsernameBox.Text?.Trim();
        var password = PasswordBox.Password;
        if (string.IsNullOrEmpty(baseUrl) || string.IsNullOrEmpty(username))
        {
            await ShowErrorAsync("请填写服务地址与用户名");
            return;
        }
        LoadingRing.IsActive = true;
        try
        {
            await _core.FeiniuLoginAsync(baseUrl, username, password);
            _currentPath = "/";
            await RefreshFeiniuFilesAsync();
            await ShowInfoAsync("登录成功");
        }
        catch (Exception ex)
        {
            // 401 不可重试，提示重新登录
            if (IsStatus(ex, 401))
            {
                await ShowErrorAsync("用户名或密码错误（401），请检查凭据后重试。");
            }
            else
            {
                await ShowErrorAsync("登录失败：" + FormatError(ex));
            }
        }
        finally
        {
            LoadingRing.IsActive = false;
        }
    }

    private async void RefreshButton_Click(object sender, RoutedEventArgs e) => await RefreshFeiniuFilesAsync();

    private async Task RefreshFeiniuFilesAsync()
    {
        PathText.Text = _currentPath;
        LoadingRing.IsActive = true;
        FeiniuFiles.Clear();
        try
        {
            var result = await RetryAsync(() => _core.FeiniuListFilesAsync(_currentPath), nameof(RefreshFeiniuFilesAsync));
            // 目录优先排序
            var ordered = result.Files.OrderBy(f => f.IsDir ? 0 : 1).ThenBy(f => f.Name);
            foreach (var f in ordered)
            {
                FeiniuFiles.Add(new NasFileItem(f, _currentPath));
            }
        }
        catch (Exception ex)
        {
            await ShowErrorAsync("列目录失败：" + FormatError(ex));
        }
        finally
        {
            LoadingRing.IsActive = false;
        }
    }

    private async void FilesList_ItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is not NasFileItem item) return;
        if (item.IsDir)
        {
            _currentPath = item.FullPath;
            await RefreshFeiniuFilesAsync();
            return;
        }
        if (!IsAudioFile(item.Name)) return;
        await PlayFeiniuFileAsync(item);
    }

    private async Task PlayFeiniuFileAsync(NasFileItem item)
    {
        LoadingRing.IsActive = true;
        try
        {
            var url = await RetryAsync(() => _core.FeiniuStreamAsync(item.FullPath), nameof(PlayFeiniuFileAsync));
            if (string.IsNullOrEmpty(url))
            {
                await ShowErrorAsync("未获取到播放地址");
                return;
            }
            var song = new Song
            {
                Id = "feiniu-" + item.Name,
                SourceId = "feiniu",
                Title = item.Name,
                Origin = new NasOrigin { Protocol = "feiniu", Url = url },
                PlayUrl = url,
            };
            _player.PlaySingle(song);
            await ShowInfoAsync("开始播放：" + item.Name);
        }
        catch (Exception ex)
        {
            await ShowErrorAsync("获取播放地址失败：" + FormatError(ex));
        }
        finally
        {
            LoadingRing.IsActive = false;
        }
    }

    private async void BackButton_Click(object sender, RoutedEventArgs e)
    {
        if (_currentPath == "/" || string.IsNullOrEmpty(_currentPath)) return;
        var idx = _currentPath.TrimEnd('/').LastIndexOf('/');
        _currentPath = idx <= 0 ? "/" : _currentPath[..idx];
        await RefreshFeiniuFilesAsync();
    }

    // =============================================================
    // 协议源浏览
    // =============================================================

    private async void RefreshSourcesButton_Click(object sender, RoutedEventArgs e) => await RefreshProtocolSourcesAsync();

    private async Task RefreshProtocolSourcesAsync()
    {
        var previousId = _selectedProtocolId;
        ProtocolSources.Clear();
        ProtocolSourceCombo.Items?.Clear();
        try
        {
            var list = await _core.ProtocolListAsync();
            foreach (var p in list)
            {
                ProtocolSources.Add(p);
                ProtocolSourceCombo.Items?.Add($"{p.Protocol} · {p.Id}" + (p.Placeholder ? "（占位）" : ""));
            }
            if (ProtocolSources.Count == 0)
            {
                ProtocolHintText.Text = "尚无协议源，请在设置页添加（WebDAV / FTP 可用，SMB / DLNA / NFS 为占位）";
                _selectedProtocolId = null;
                return;
            }
            ProtocolHintText.Text = "选择协议源以浏览文件（SMB / DLNA / NFS 为占位实现）";
            // 尽量保持原选择
            var restoreIdx = string.IsNullOrEmpty(previousId)
                ? -1
                : ProtocolSources.ToList().FindIndex(p => p.Id == previousId);
            ProtocolSourceCombo.SelectedIndex = restoreIdx >= 0 ? restoreIdx : 0;
        }
        catch (Exception ex)
        {
            ProtocolHintText.Text = "加载协议源失败：" + FormatError(ex);
        }
    }

    private void ProtocolSourceCombo_SelectionChanged(object sender, SelectionChangedEventArgs e)
    {
        if (ProtocolSourceCombo.SelectedIndex < 0 || ProtocolSourceCombo.SelectedIndex >= ProtocolSources.Count)
        {
            _selectedProtocolId = null;
            return;
        }
        var src = ProtocolSources[ProtocolSourceCombo.SelectedIndex];
        _selectedProtocolId = src.Id;
        if (src.Placeholder)
        {
            ProtocolHintText.Text = $"{src.Protocol} 为占位实现，浏览 / 拉流不可用（需启用对应 feature）。建议使用 WebDAV / FTP。";
            ProtocolEntries.Clear();
            return;
        }
        ProtocolHintText.Text = "已选择：" + src.Protocol + " · " + src.Id;
        _protocolPath = src.Root?.TrimEnd('/') ?? "/";
        if (string.IsNullOrEmpty(_protocolPath)) _protocolPath = "/";
        _ = RefreshProtocolEntriesAsync();
    }

    private async void ProtocolRefreshButton_Click(object sender, RoutedEventArgs e) => await RefreshProtocolEntriesAsync();

    private async Task RefreshProtocolEntriesAsync()
    {
        if (string.IsNullOrEmpty(_selectedProtocolId)) return;
        ProtocolPathText.Text = _protocolPath;
        ProtocolLoadingRing.IsActive = true;
        ProtocolEntries.Clear();
        try
        {
            var result = await RetryAsync(
                () => _core.ProtocolListFilesAsync(_selectedProtocolId, _protocolPath),
                nameof(RefreshProtocolEntriesAsync));
            // 协议源仅返回条目名称；以末尾分隔符判断目录
            var ordered = result.Entries.OrderBy(n => n.EndsWith('/') ? 0 : 1).ThenBy(n => n);
            foreach (var name in ordered)
            {
                ProtocolEntries.Add(new ProtocolEntry(name, _protocolPath));
            }
        }
        catch (Exception ex)
        {
            ProtocolHintText.Text = "浏览失败：" + FormatError(ex);
        }
        finally
        {
            ProtocolLoadingRing.IsActive = false;
        }
    }

    private async void ProtocolEntriesList_ItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is not ProtocolEntry entry) return;
        if (entry.IsDir)
        {
            _protocolPath = entry.FullPath;
            await RefreshProtocolEntriesAsync();
            return;
        }
        if (!IsAudioFile(entry.Name)) return;
        await PlayProtocolFileAsync(entry);
    }

    private async Task PlayProtocolFileAsync(ProtocolEntry entry)
    {
        ProtocolLoadingRing.IsActive = true;
        try
        {
            var url = await RetryAsync(
                () => _core.ProtocolStreamAsync(_selectedProtocolId, entry.FullPath),
                nameof(PlayProtocolFileAsync));
            if (string.IsNullOrEmpty(url))
            {
                await ShowErrorAsync("未获取到播放地址");
                return;
            }
            var song = new Song
            {
                Id = "proto-" + entry.Name,
                SourceId = "protocol",
                Title = entry.Name,
                Origin = new NasOrigin { Protocol = "protocol", Url = url },
                PlayUrl = url,
            };
            _player.PlaySingle(song);
            await ShowInfoAsync("开始播放：" + entry.Name);
        }
        catch (Exception ex)
        {
            await ShowErrorAsync("获取播放地址失败：" + FormatError(ex));
        }
        finally
        {
            ProtocolLoadingRing.IsActive = false;
        }
    }

    private async void ProtocolBackButton_Click(object sender, RoutedEventArgs e)
    {
        if (_protocolPath == "/" || string.IsNullOrEmpty(_protocolPath)) return;
        var idx = _protocolPath.TrimEnd('/').LastIndexOf('/');
        _protocolPath = idx <= 0 ? "/" : _protocolPath[..idx];
        await RefreshProtocolEntriesAsync();
    }

    // =============================================================
    // 重试与错误处理工具
    // =============================================================

    /// <summary>
    /// 对 502/504 类网络错误做指数退避重试（1s/2s/4s，最多 3 次）；
    /// 401 / 404 / 501 等不重试，直接抛出由调用方处理。
    /// </summary>
    private static async Task<T> RetryAsync<T>(Func<Task<T>> fn, string opName)
    {
        const int maxRetries = 3;
        var delays = new[] { 1000, 2000, 4000 };
        Exception? last = null;
        for (var attempt = 0; attempt <= maxRetries; attempt++)
        {
            try
            {
                return await fn();
            }
            catch (Exception ex)
            {
                last = ex;
                if (!IsRetryable(ex) || attempt == maxRetries) break;
                await Task.Delay(delays[attempt]);
            }
        }
        throw last!;
    }

    private static bool IsRetryable(Exception ex)
    {
        // 502 / 504 与网络不可达类错误可重试
        var msg = ex.Message ?? string.Empty;
        return msg.Contains("502") || msg.Contains("504") ||
               msg.Contains("不可达") || msg.Contains("请求失败");
    }

    private static bool IsStatus(Exception ex, int status) =>
        (ex.Message ?? string.Empty).Contains(status.ToString());

    private static string FormatError(Exception ex)
    {
        var msg = ex.Message ?? "未知错误";
        if (IsStatus(ex, 401)) return msg + "\n提示：未登录或 token 失效，请重新登录。";
        if (IsStatus(ex, 404)) return msg + "\n提示：路径不存在，请修正路径。";
        if (IsStatus(ex, 501)) return msg + "\n提示：该协议为占位实现，请使用 WebDAV / FTP 或启用对应 feature。";
        return msg;
    }

    private static bool IsAudioFile(string name)
    {
        if (string.IsNullOrEmpty(name)) return false;
        var exts = new[] { ".mp3", ".flac", ".wav", ".m4a", ".aac", ".ogg", ".opus", ".wma" };
        var lower = name.ToLowerInvariant();
        return exts.Any(lower.EndsWith);
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

    private async Task ShowInfoAsync(string message)
    {
        var dialog = new ContentDialog
        {
            XamlRoot = XamlRoot,
            Title = "逆光音乐",
            Content = message,
            CloseButtonText = "好",
        };
        await dialog.ShowAsync();
    }
}

/// <summary>飞牛文件条目视图模型，附加图标 / 尺寸 / 元信息 / 完整路径。</summary>
public sealed class NasFileItem
{
    public string Name { get; }
    public bool IsDir { get; }
    public ulong Size { get; }
    public string? Modified { get; }
    public string FullPath { get; }
    public string IconGlyph => IsDir ? "\uE8B7" : "\uE8D6"; // 文件夹 / 音频
    public string SizeText => IsDir ? string.Empty : FormatBytes(Size);
    public string MetaText => (string.IsNullOrEmpty(Modified) ? "" : Modified) + (IsDir ? "  ·  文件夹" : "");

    public NasFileItem(NasFileEntry entry, string parentPath)
    {
        Name = entry.Name ?? string.Empty;
        IsDir = entry.IsDir;
        Size = entry.Size;
        Modified = entry.Modified;
        FullPath = JoinPath(parentPath, Name, IsDir);
    }

    private static string JoinPath(string parent, string name, bool isDir)
    {
        var p = parent.TrimEnd('/');
        var full = (p.Length == 0 ? "" : p + "/") + name.TrimStart('/');
        return isDir && !full.EndsWith('/') ? full + "/" : full;
    }

    private static string FormatBytes(ulong bytes) => bytes switch
    {
        < 1024 => $"{bytes} B",
        < 1024 * 1024 => $"{bytes / 1024.0:F1} KB",
        < 1024UL * 1024 * 1024 => $"{bytes / (1024.0 * 1024):F1} MB",
        _ => $"{bytes / (1024.0 * 1024 * 1024):F2} GB",
    };
}

/// <summary>协议源条目视图模型（协议 list 仅返回条目名称）。</summary>
public sealed class ProtocolEntry
{
    public string Name { get; }
    public bool IsDir { get; }
    public string FullPath { get; }
    public string IconGlyph => IsDir ? "\uE8B7" : "\uE8D6";

    public ProtocolEntry(string rawName, string parentPath)
    {
        // 名称可能带末尾分隔符表示目录
        var name = rawName ?? string.Empty;
        IsDir = name.EndsWith('/');
        Name = name.TrimEnd('/');
        FullPath = JoinPath(parentPath, Name, IsDir);
    }

    private static string JoinPath(string parent, string name, bool isDir)
    {
        var p = parent.TrimEnd('/');
        var full = (p.Length == 0 ? "/" : p + "/") + name.TrimStart('/');
        return isDir && !full.EndsWith('/') ? full + "/" : full;
    }
}
