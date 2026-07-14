using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Microsoft.UI.Xaml;
using Microsoft.UI.Xaml.Controls;
using MusicPlayerApp.Models;
using MusicPlayerApp.Services;
using Windows.Storage.Pickers;

namespace MusicPlayerApp.Views;

/// <summary>本地音乐页：管理本地扫描目录、触发重新扫描、展示扫描进度。</summary>
public sealed partial class LocalMusicPage : Page
{
    private readonly CoreService _core = CoreService.Instance;
    private readonly PlayerService _player;

    public ObservableCollection<SongViewModel> Songs { get; } = new();

    public LocalMusicPage()
    {
        InitializeComponent();
        _player = App.Player;
        _ = RefreshProgressAsync();
    }

    private async void AddDirButton_Click(object sender, RoutedEventArgs e)
    {
        var picker = new FolderPicker
        {
            SuggestedStartLocation = PickerLocationId.MusicLibrary,
        };
        picker.FileTypeFilter.Add("*");

        var hwnd = WinRT.Interop.WindowNative.GetWindowForCurrentThread();
        WinRT.Interop.InitializeWithWindow.Initialize(picker, hwnd);

        var folder = await picker.PickSingleFolderAsync();
        if (folder is null)
        {
            return;
        }
        try
        {
            ScanRing.IsActive = true;
            ProgressText.Text = $"正在扫描 {folder.Path} ...";
            await _core.LocalAddDirAsync(folder.Path);
            await RefreshProgressAsync();
        }
        catch (Exception ex)
        {
            await ShowErrorAsync($"添加目录失败: {ex.Message}");
        }
        finally
        {
            ScanRing.IsActive = false;
        }
    }

    private async void RescanButton_Click(object sender, RoutedEventArgs e)
    {
        ScanRing.IsActive = true;
        ProgressText.Text = "正在重新扫描...";
        try
        {
            await _core.LocalRescanAsync();
            await RefreshProgressAsync();
        }
        catch (Exception ex)
        {
            await ShowErrorAsync($"重新扫描失败: {ex.Message}");
        }
        finally
        {
            ScanRing.IsActive = false;
        }
    }

    private async Task RefreshProgressAsync()
    {
        try
        {
            var p = await _core.LocalProgressAsync();
            ProgressText.Text = p.Scanning
                ? $"扫描中... 已索引 {p.CurrentCount} 首"
                : $"本地音乐库就绪：共 {p.CurrentCount} 首";
        }
        catch (Exception ex)
        {
            ProgressText.Text = $"进度查询失败: {ex.Message}";
        }
    }

    private void LocalList_ItemClick(object sender, ItemClickEventArgs e)
    {
        if (e.ClickedItem is SongViewModel vm)
        {
            _player.PlaySingle(vm.Source);
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
