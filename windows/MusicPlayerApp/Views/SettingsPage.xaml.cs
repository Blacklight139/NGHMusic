using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using MusicPlayerApp.Services;
using Windows.Storage.Pickers;

namespace MusicPlayerApp.Views;

/// <summary>
/// 设置页：包含音源导入（文件选择器 + LXMusic 风格列表管理）、协议源管理、缓存管理。
/// </summary>
public sealed partial class SettingsPage : Page
{
    private readonly CoreService _core = CoreService.Instance;

    public ObservableCollection<SourceInfo> Sources { get; } = new();
    public ObservableCollection<ProtocolSourceInfo> ProtocolSources { get; } = new();

    public SettingsPage()
    {
        InitializeComponent();
        _ = InitializeAsync();
    }

    private async Task InitializeAsync()
    {
        try
        {
            await _core.InitializeAsync();
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[SettingsPage] 核心初始化失败: {ex.Message}");
        }
        await RefreshSourcesAsync();
        await RefreshProtocolsAsync();
        await RefreshCacheAsync();
        await RefreshVersionAsync();
    }

    // =============================================================
    // 音源管理
    // =============================================================

    private async void ImportSourceButton_Click(object sender, RoutedEventArgs e)
    {
        var picker = new FileOpenPicker
        {
            ViewMode = PickerViewMode.List,
            SuggestedStartLocation = PickerLocationId.DocumentsLibrary,
        };
        picker.FileTypeFilter.Add(".json");
        picker.FileTypeFilter.Add("*");

        var hwnd = WinRT.Interop.WindowNative.GetWindowForCurrentThread();
        WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);

        var file = await picker.PickSingleFileAsync();
        if (file is null)
        {
            return;
        }
        try
        {
            var json = await File.ReadAllTextAsync(file.Path);
            // 先校验，再导入
            var validation = await _core.ValidateSourceAsync(json);
            if (!validation.Valid)
            {
                await ShowErrorAsync("校验未通过：\n" + string.Join("\n", validation.Errors));
                return;
            }
            await _core.ImportSourceAsync(json);
            await RefreshSourcesAsync();
        }
        catch (Exception ex)
        {
            await ShowErrorAsync($"导入失败: {ex.Message}");
        }
    }

    private async void RefreshSourcesButton_Click(object sender, RoutedEventArgs e)
    {
        await RefreshSourcesAsync();
    }

    private async Task RefreshSourcesAsync()
    {
        try
        {
            var list = await _core.ListSourcesAsync();
            Sources.Clear();
            foreach (var s in list)
            {
                Sources.Add(s);
            }
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[SettingsPage] 加载音源失败: {ex.Message}");
        }
    }

    private async void EnabledToggle_Toggled(object sender, RoutedEventArgs e)
    {
        if (sender is not ToggleSwitch ts || ts.Tag is not string id)
        {
            return;
        }
        try
        {
            if (ts.IsOn)
            {
                await _core.EnableSourceAsync(id);
            }
            else
            {
                await _core.DisableSourceAsync(id);
            }
        }
        catch (Exception ex)
        {
            await ShowErrorAsync($"切换状态失败: {ex.Message}");
            await RefreshSourcesAsync();
        }
    }

    private async void DeleteSourceButton_Click(object sender, RoutedEventArgs e)
    {
        if (sender is not Button btn || btn.Tag is not string id)
        {
            return;
        }
        try
        {
            await _core.DeleteSourceAsync(id);
            await RefreshSourcesAsync();
        }
        catch (Exception ex)
        {
            await ShowErrorAsync($"删除失败: {ex.Message}");
        }
    }

    // =============================================================
    // 协议源管理
    // =============================================================

    private async void AddProtocolButton_Click(object sender, RoutedEventArgs e)
    {
        var protocol = (ProtocolCombo.SelectedItem as string) ?? "webdav";
        var configText = ProtocolConfigBox.Text?.Trim() ?? string.Empty;
        if (string.IsNullOrEmpty(configText))
        {
            await ShowErrorAsync("请填写协议配置 JSON");
            return;
        }

        // 注入 protocol 字段（若用户未填）
        string finalJson;
        try
        {
            using var doc = System.Text.Json.JsonDocument.Parse(configText);
            var root = doc.RootElement;
            if (root.ValueKind == System.Text.Json.JsonValueKind.Object &&
                root.TryGetProperty("protocol", out _))
            {
                finalJson = configText;
            }
            else
            {
                // 简单包装注入 protocol 字段
                finalJson = configText.TrimStart('{').TrimEnd('}');
                finalJson = $"{{\"protocol\":\"{protocol}\",{finalJson}";
            }
        }
        catch (Exception ex)
        {
            await ShowErrorAsync($"配置 JSON 解析失败: {ex.Message}");
            return;
        }

        try
        {
            await _core.ProtocolAddAsync(finalJson);
            ProtocolConfigBox.Text = string.Empty;
            await RefreshProtocolsAsync();
        }
        catch (Exception ex)
        {
            await ShowErrorAsync($"添加协议源失败: {ex.Message}");
        }
    }

    private async void DeleteProtocolButton_Click(object sender, RoutedEventArgs e)
    {
        if (sender is not Button btn || btn.Tag is not string id)
        {
            return;
        }
        try
        {
            await _core.ProtocolDeleteAsync(id);
            await RefreshProtocolsAsync();
        }
        catch (Exception ex)
        {
            await ShowErrorAsync($"删除协议源失败: {ex.Message}");
        }
    }

    private async Task RefreshProtocolsAsync()
    {
        try
        {
            var list = await _core.ProtocolListAsync();
            ProtocolSources.Clear();
            foreach (var p in list)
            {
                ProtocolSources.Add(p);
            }
        }
        catch (Exception ex)
        {
            System.Diagnostics.Debug.WriteLine($"[SettingsPage] 加载协议源失败: {ex.Message}");
        }
    }

    // =============================================================
    // 缓存管理
    // =============================================================

    private async void ClearCacheButton_Click(object sender, RoutedEventArgs e)
    {
        try
        {
            await _core.CacheClearAsync();
            await RefreshCacheAsync();
        }
        catch (Exception ex)
        {
            await ShowErrorAsync($"清空缓存失败: {ex.Message}");
        }
    }

    private async void RefreshCacheButton_Click(object sender, RoutedEventArgs e)
    {
        await RefreshCacheAsync();
    }

    private async Task RefreshCacheAsync()
    {
        try
        {
            var s = await _core.CacheStatsAsync();
            CacheStatsText.Text = $"条目数：{s.Entries}  ·  已用：{FormatBytes(s.TotalBytes)}  ·  上限：{FormatBytes(s.MaxBytes)}";
        }
        catch (Exception ex)
        {
            CacheStatsText.Text = $"缓存信息获取失败: {ex.Message}";
        }
    }

    // =============================================================
    // 版本信息
    // =============================================================

    private async Task RefreshVersionAsync()
    {
        try
        {
            var ver = await _core.GetVersionAsync();
            CoreVersionText.Text = $"核心版本：{ver}";
        }
        catch (Exception ex)
        {
            CoreVersionText.Text = $"核心版本获取失败: {ex.Message}";
        }
    }

    private static string FormatBytes(ulong bytes) => bytes switch
    {
        < 1024 => $"{bytes} B",
        < 1024 * 1024 => $"{bytes / 1024.0:F1} KB",
        < 1024UL * 1024 * 1024 => $"{bytes / (1024.0 * 1024):F1} MB",
        _ => $"{bytes / (1024.0 * 1024 * 1024):F2} GB",
    };

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
