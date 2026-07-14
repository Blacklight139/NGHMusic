using System;
using System.Collections.Generic;
using System.IO;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using MusicCore.Native;
using MusicPlayerApp.Models;
using Windows.Storage;

namespace MusicPlayerApp.Services;

/// <summary>
/// 应用层核心服务封装：在 <see cref="MusicCoreNative"/> P/Invoke 之上提供
/// 强类型、异步、JSON 反序列化的接口。所有方法均通过后台线程调用原生代码，
/// 返回类型化模型；失败时抛出 <see cref="MusicCoreException"/>。
/// </summary>
public sealed class CoreService
{
    /// <summary>JSON 序列化选项（驼峰属性名兼容，反序列化时忽略大小写与多余字段）。</summary>
    private static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNameCaseInsensitive = true,
        ReadCommentHandling = JsonCommentHandling.Skip,
        AllowTrailingCommas = true,
    };

    /// <summary>单例实例（应用启动时由 App.xaml.cs 注入或懒加载）。</summary>
    public static CoreService Instance { get; } = new CoreService();

    private bool _initialized;

    private CoreService()
    {
    }

    /// <summary>
    /// 初始化本地索引库与缓存目录到 <see cref="ApplicationData.Current.LocalFolder"/>。
    /// 必须在调用本地 / 缓存相关接口之前调用一次。重复调用安全。
    /// </summary>
    public async Task InitializeAsync()
    {
        if (_initialized)
        {
            return;
        }

        var localFolder = ApplicationData.Current.LocalFolder;
        var dbPath = Path.Combine(localFolder.Path, "local.db");
        var cacheDir = Path.Combine(localFolder.Path, "cache");
        Directory.CreateDirectory(cacheDir);

        await Task.Run(() =>
        {
            MusicCoreNative.LocalInit(dbPath);
            MusicCoreNative.CacheInit(cacheDir, maxBytes: 1024L * 1024 * 1024); // 1GB
        });

        _initialized = true;
    }

    // =============================================================
    // 内部反序列化辅助
    // =============================================================

    private static T Deserialize<T>(string json) =>
        JsonSerializer.Deserialize<T>(json, JsonOptions)
            ?? throw new MusicCoreException("Json", $"反序列化失败: {typeof(T).Name}");

    private static async Task<T> RunAsync<T>(Func<T> fn, CancellationToken ct = default)
    {
        return await Task.Run(() =>
        {
            try
            {
                return fn();
            }
            catch (MusicCoreException)
            {
                throw;
            }
            catch (Exception ex)
            {
                throw new MusicCoreException("Ffi", $"FFI 调用失败: {ex.Message}");
            }
        }, ct);
    }

    // =============================================================
    // 版本
    // =============================================================

    /// <summary>返回核心库版本字符串。</summary>
    public Task<string> GetVersionAsync(CancellationToken ct = default) =>
        RunAsync(() => MusicCoreNative.Version(), ct);

    // =============================================================
    // 音源管理
    // =============================================================

    /// <summary>导入音源 JSON（标准 / 社区格式），返回导入报告对象。</summary>
    public Task<JsonDocument> ImportSourceAsync(string json, CancellationToken ct = default) =>
        RunAsync(() =>
        {
            var resultJson = MusicCoreNative.SourceImport(json);
            return JsonDocument.Parse(resultJson);
        }, ct);

    /// <summary>校验音源 JSON，返回 (valid, errors)。</summary>
    public async Task<(bool Valid, List<string> Errors)> ValidateSourceAsync(string json, CancellationToken ct = default)
    {
        var resultJson = await RunAsync(() => MusicCoreNative.SourceValidate(json), ct);
        using var doc = JsonDocument.Parse(resultJson);
        var root = doc.RootElement;
        var valid = root.TryGetProperty("valid", out var v) && v.GetBoolean();
        var errors = new List<string>();
        if (root.TryGetProperty("errors", out var arr) && arr.ValueKind == JsonValueKind.Array)
        {
            foreach (var item in arr.EnumerateArray())
            {
                if (item.ValueKind == JsonValueKind.String)
                {
                    errors.Add(item.GetString() ?? string.Empty);
                }
            }
        }
        return (valid, errors);
    }

    /// <summary>列出所有音源（按 priority 降序）。</summary>
    public async Task<List<SourceInfo>> ListSourcesAsync(CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.SourceList(), ct);
        using var doc = JsonDocument.Parse(json);
        var root = doc.RootElement;
        // 兼容直接返回数组或 {"sources":[...]} 两种形态
        List<SourceInfo> list = new();
        if (root.ValueKind == JsonValueKind.Array)
        {
            AddRange(root.EnumerateArray());
        }
        else if (root.TryGetProperty("sources", out var s) && s.ValueKind == JsonValueKind.Array)
        {
            AddRange(s.EnumerateArray());
        }
        return list;

        void AddRange(JsonElement.ArrayEnumerator en)
        {
            foreach (var item in en)
            {
                list.Add(new SourceInfo
                {
                    Id = item.TryGetProperty("id", out var id) ? id.GetString() ?? string.Empty : string.Empty,
                    Name = item.TryGetProperty("name", out var n) ? n.GetString() ?? string.Empty : string.Empty,
                    Enabled = item.TryGetProperty("enabled", out var e) && e.GetBoolean(),
                    Priority = item.TryGetProperty("priority", out var p) ? p.GetInt32() : 0,
                });
            }
        }
    }

    public Task EnableSourceAsync(string id, CancellationToken ct = default) =>
        RunAsync(() => MusicCoreNative.SourceEnable(id), ct);

    public Task DisableSourceAsync(string id, CancellationToken ct = default) =>
        RunAsync(() => MusicCoreNative.SourceDisable(id), ct);

    public Task DeleteSourceAsync(string id, CancellationToken ct = default) =>
        RunAsync(() => MusicCoreNative.SourceDelete(id), ct);

    // =============================================================
    // 搜索与歌曲
    // =============================================================

    public Task<SearchResult> SearchAsync(string keyword, uint page = 1, uint pageSize = 20, CancellationToken ct = default) =>
        RunAsync(() => Deserialize<SearchResult>(MusicCoreNative.Search(keyword, page, pageSize)), ct);

    public Task<Song> GetMetadataAsync(string sourceId, string songId, CancellationToken ct = default) =>
        RunAsync(() => Deserialize<Song>(MusicCoreNative.GetMetadata(sourceId, songId)), ct);

    public async Task<PlayUrlResult> GetPlayUrlAsync(string sourceId, string songId, CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.GetPlayUrl(sourceId, songId), ct);
        var result = Deserialize<PlayUrlResult>(json);
        // 兼容 cached URL 字段优先级：url > play_url
        result.Url ??= result.PlayUrl;
        return result;
    }

    public Task<Lyric> GetLyricAsync(string sourceId, string songId, CancellationToken ct = default) =>
        RunAsync(() => Deserialize<Lyric>(MusicCoreNative.GetLyric(sourceId, songId)), ct);

    public async Task<List<Leaderboard>> GetLeaderboardsAsync(string sourceId, CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.GetLeaderboards(sourceId), ct);
        // 排行榜返回裸数组
        return JsonSerializer.Deserialize<List<Leaderboard>>(json, JsonOptions)
            ?? new List<Leaderboard>();
    }

    // =============================================================
    // 飞牛 NAS
    // =============================================================

    public async Task<FeiniuLoginResult> FeiniuLoginAsync(string baseUrl, string username, string password, CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.FeiniuLogin(baseUrl, username, password), ct);
        return Deserialize<FeiniuLoginResult>(json);
    }

    public async Task<FeiniuFilesResult> FeiniuListFilesAsync(string path, CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.FeiniuListFiles(path), ct);
        return Deserialize<FeiniuFilesResult>(json);
    }

    public async Task<string> FeiniuStreamAsync(string path, CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.FeiniuStream(path), ct);
        using var doc = JsonDocument.Parse(json);
        return doc.RootElement.TryGetProperty("url", out var u) ? u.GetString() ?? string.Empty : string.Empty;
    }

    public async Task<bool> FeiniuHealthAsync(CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.FeiniuHealth(), ct);
        using var doc = JsonDocument.Parse(json);
        return doc.RootElement.TryGetProperty("healthy", out var h) && h.GetBoolean();
    }

    // =============================================================
    // 协议源
    // =============================================================

    public async Task<ProtocolSourceInfo> ProtocolAddAsync(string configJson, CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.ProtocolAdd(configJson), ct);
        return Deserialize<ProtocolSourceInfo>(json);
    }

    public async Task<List<ProtocolSourceInfo>> ProtocolListAsync(CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.ProtocolList(), ct);
        using var doc = JsonDocument.Parse(json);
        var list = new List<ProtocolSourceInfo>();
        if (doc.RootElement.TryGetProperty("sources", out var arr) && arr.ValueKind == JsonValueKind.Array)
        {
            foreach (var item in arr.EnumerateArray())
            {
                list.Add(JsonSerializer.Deserialize<ProtocolSourceInfo>(item.GetRawText(), JsonOptions)
                    ?? new ProtocolSourceInfo());
            }
        }
        return list;
    }

    public Task ProtocolDeleteAsync(string id, CancellationToken ct = default) =>
        RunAsync(() => MusicCoreNative.ProtocolDelete(id), ct);

    public async Task<ProtocolFilesResult> ProtocolListFilesAsync(string id, string path, CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.ProtocolListFiles(id, path), ct);
        return Deserialize<ProtocolFilesResult>(json);
    }

    public async Task<string> ProtocolReadAsync(string id, string path, CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.ProtocolRead(id, path), ct);
        using var doc = JsonDocument.Parse(json);
        return doc.RootElement.TryGetProperty("data_base64", out var d) ? d.GetString() ?? string.Empty : string.Empty;
    }

    public async Task<string> ProtocolStreamAsync(string id, string path, CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.ProtocolStream(id, path), ct);
        using var doc = JsonDocument.Parse(json);
        return doc.RootElement.TryGetProperty("url", out var u) ? u.GetString() ?? string.Empty : string.Empty;
    }

    // =============================================================
    // 本地音乐
    // =============================================================

    public Task LocalAddDirAsync(string dir, CancellationToken ct = default) =>
        RunAsync(() => MusicCoreNative.LocalAddDir(dir), ct);

    public Task LocalRescanAsync(CancellationToken ct = default) =>
        RunAsync(() => MusicCoreNative.LocalRescan(), ct);

    public async Task<LocalProgressResult> LocalProgressAsync(CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.LocalProgress(), ct);
        return Deserialize<LocalProgressResult>(json);
    }

    // =============================================================
    // 缓存
    // =============================================================

    public async Task<CacheStatsResult> CacheStatsAsync(CancellationToken ct = default)
    {
        var json = await RunAsync(() => MusicCoreNative.CacheStats(), ct);
        return Deserialize<CacheStatsResult>(json);
    }

    public Task CacheClearAsync(CancellationToken ct = default) =>
        RunAsync(() => MusicCoreNative.CacheClear(), ct);
}

// =============================================================
// 服务返回的辅助 DTO（与 Rust JSON 对齐，snake_case）
// =============================================================

/// <summary>音源信息（ListSources 返回）。</summary>
public sealed class SourceInfo
{
    public string Id { get; set; } = string.Empty;
    public string Name { get; set; } = string.Empty;
    public bool Enabled { get; set; }
    public int Priority { get; set; }
}

/// <summary>播放 URL 接口返回（兼容 url / play_url / cached）。</summary>
public sealed class PlayUrlResult
{
    public string? Url { get; set; }

    public string? PlayUrl { get; set; }

    public bool Cached { get; set; }
}

/// <summary>飞牛登录返回。</summary>
public sealed class FeiniuLoginResult
{
    public string? Token { get; set; }
    public string? BaseUrl { get; set; }
}

/// <summary>飞牛 / 协议源 列目录条目。</summary>
public sealed class NasFileEntry
{
    public string? Name { get; set; }
    public bool IsDir { get; set; }
    public ulong Size { get; set; }
    public string? Modified { get; set; }
}

/// <summary>飞牛列目录返回。</summary>
public sealed class FeiniuFilesResult
{
    public string? Path { get; set; }
    public List<NasFileEntry> Files { get; set; } = new();
}

/// <summary>协议源信息。</summary>
public sealed class ProtocolSourceInfo
{
    public string Id { get; set; } = string.Empty;
    public string? Protocol { get; set; }
    public string? Root { get; set; }
    public bool Enabled { get; set; }
    public bool Placeholder { get; set; }
}

/// <summary>协议源列目录返回。</summary>
public sealed class ProtocolFilesResult
{
    public string? Path { get; set; }
    public List<string> Entries { get; set; } = new();
}

/// <summary>本地扫描进度。</summary>
public sealed class LocalProgressResult
{
    public ulong CurrentCount { get; set; }
    public bool Scanning { get; set; }
}

/// <summary>缓存统计。</summary>
public sealed class CacheStatsResult
{
    public ulong Entries { get; set; }
    public ulong TotalBytes { get; set; }
    public ulong MaxBytes { get; set; }
}
