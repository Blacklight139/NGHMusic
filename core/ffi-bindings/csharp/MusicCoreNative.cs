using System;
using System.Runtime.InteropServices;
using System.Text.Json;

namespace MusicCore.Native;

/// <summary>
/// Rust 核心（music_core cdylib）C ABI 的 C# P/Invoke 封装。
/// </summary>
/// <remarks>
/// 所有返回字符串的 C ABI 函数均通过 <c>CString::into_raw</c> 分配，
/// 调用方须用 <see cref="music_core_free_string"/> 释放。本封装在安全包装层
/// 完成读取与释放，调用方只需处理 <see cref="MusicCoreException"/>。
/// 返回值为核心序列化的 JSON 字符串；失败时抛出 <see cref="MusicCoreException"/>。
/// </remarks>
public static class MusicCoreNative
{
    private const string LibName = "music_core";

    // =============================================================
    // 原生导入（与 core/src/ffi.rs 的 #[no_mangle] 符号一一对应）
    // =============================================================

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_version();

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern void music_core_free_string(IntPtr ptr);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_source_import(IntPtr json);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_source_validate(IntPtr json);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_source_list();

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_source_enable(IntPtr id);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_source_disable(IntPtr id);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_source_delete(IntPtr id);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_search(IntPtr keyword, uint page, uint pageSize);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_get_metadata(IntPtr sourceId, IntPtr songId);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_get_play_url(IntPtr sourceId, IntPtr songId);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_get_lyric(IntPtr sourceId, IntPtr songId);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_get_leaderboards(IntPtr sourceId);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_feiniu_login(IntPtr baseUrl, IntPtr username, IntPtr password);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_feiniu_list_files(IntPtr path);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_feiniu_stream(IntPtr path);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_feiniu_health();

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_protocol_add(IntPtr configJson);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_protocol_list();

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_protocol_delete(IntPtr id);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_protocol_list_files(IntPtr id, IntPtr path);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_protocol_read(IntPtr id, IntPtr path);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_protocol_stream(IntPtr id, IntPtr path);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_local_init(IntPtr dbPath);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_local_add_dir(IntPtr dir);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_local_rescan();

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_local_progress();

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_cache_init(IntPtr cacheDir, ulong maxBytes);

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_cache_stats();

    [DllImport(LibName, CallingConvention = CallingConvention.Cdecl, CharSet = CharSet.Ansi, ExactSpelling = true)]
    private static extern IntPtr music_core_cache_clear();

    // =============================================================
    // 私有辅助
    // =============================================================

    /// <summary>分配 ANSI C 字符串；null 返回 <see cref="IntPtr.Zero"/>。</summary>
    private static IntPtr AllocCString(string? s) =>
        s is null ? IntPtr.Zero : Marshal.StringToHGlobalAnsi(s);

    /// <summary>读取核心返回的 C 字符串并立即释放其内存。</summary>
    private static string? ReadAndFree(IntPtr ptr)
    {
        if (ptr == IntPtr.Zero)
        {
            return null;
        }
        try
        {
            return Marshal.PtrToStringAnsi(ptr);
        }
        finally
        {
            music_core_free_string(ptr);
        }
    }

    /// <summary>
    /// 校验返回 JSON 是否为错误对象（<c>{"error":{"kind":...,"message":...}}</c>），
    /// 是则抛出 <see cref="MusicCoreException"/>；否则原样返回 JSON 字符串。
    /// </summary>
    private static string CheckError(string? json)
    {
        if (json is null)
        {
            throw new MusicCoreException("Ffi", "核心返回空指针（序列化失败）");
        }
        using var doc = JsonDocument.Parse(json);
        if (doc.RootElement.ValueKind == JsonValueKind.Object &&
            doc.RootElement.TryGetProperty("error", out var err) &&
            err.ValueKind == JsonValueKind.Object)
        {
            var kind = err.TryGetProperty("kind", out var k) && k.ValueKind == JsonValueKind.String
                ? k.GetString() ?? "Ffi"
                : "Ffi";
            var msg = err.TryGetProperty("message", out var m) && m.ValueKind == JsonValueKind.String
                ? m.GetString() ?? string.Empty
                : string.Empty;
            throw new MusicCoreException(kind, msg);
        }
        return json;
    }

    /// <summary>调用无参函数并完成读取/释放/错误检查。</summary>
    private static string InvokeNoArg(Func<IntPtr> fn) =>
        CheckError(ReadAndFree(fn()));

    // =============================================================
    // 安全包装：版本与内存
    // =============================================================

    /// <summary>返回核心库版本字符串（JSON：随核心实现，通常为版本号字面量）。</summary>
    public static string Version() => InvokeNoArg(music_core_version);

    // =============================================================
    // 安全包装：音源管理
    // =============================================================

    /// <summary>导入音源 JSON（社区格式自动适配），返回导入后的 SourceInfo JSON。</summary>
    public static string SourceImport(string json)
    {
        var arg = AllocCString(json);
        try
        {
            return CheckError(ReadAndFree(music_core_source_import(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>校验音源 JSON 是否符合标准 Schema，返回 {"valid":bool,"errors":[...]}。</summary>
    public static string SourceValidate(string json)
    {
        var arg = AllocCString(json);
        try
        {
            return CheckError(ReadAndFree(music_core_source_validate(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>列出所有音源（按 priority 降序），返回 SourceInfo 数组 JSON。</summary>
    public static string SourceList() => InvokeNoArg(music_core_source_list);

    /// <summary>启用指定 id 的音源，返回 {"id":...,"enabled":true}。</summary>
    public static string SourceEnable(string id)
    {
        var arg = AllocCString(id);
        try
        {
            return CheckError(ReadAndFree(music_core_source_enable(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>禁用指定 id 的音源，返回 {"id":...,"enabled":false}。</summary>
    public static string SourceDisable(string id)
    {
        var arg = AllocCString(id);
        try
        {
            return CheckError(ReadAndFree(music_core_source_disable(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>删除指定 id 的音源，返回 {"id":...,"deleted":true}。</summary>
    public static string SourceDelete(string id)
    {
        var arg = AllocCString(id);
        try
        {
            return CheckError(ReadAndFree(music_core_source_delete(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    // =============================================================
    // 安全包装：搜索与歌曲
    // =============================================================

    /// <summary>聚合搜索，返回 SearchResult JSON。</summary>
    public static string Search(string keyword, uint page, uint pageSize)
    {
        var arg = AllocCString(keyword);
        try
        {
            return CheckError(ReadAndFree(music_core_search(arg, page, pageSize)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>获取指定音源下歌曲的完整元数据，返回 Song JSON。</summary>
    public static string GetMetadata(string sourceId, string songId)
    {
        var a1 = AllocCString(sourceId);
        var a2 = AllocCString(songId);
        try
        {
            return CheckError(ReadAndFree(music_core_get_metadata(a1, a2)));
        }
        finally
        {
            if (a1 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a1);
            }
            if (a2 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a2);
            }
        }
    }

    /// <summary>获取指定音源下歌曲的可播放 URL，返回 {"url":...,"cached":false}。</summary>
    public static string GetPlayUrl(string sourceId, string songId)
    {
        var a1 = AllocCString(sourceId);
        var a2 = AllocCString(songId);
        try
        {
            return CheckError(ReadAndFree(music_core_get_play_url(a1, a2)));
        }
        finally
        {
            if (a1 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a1);
            }
            if (a2 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a2);
            }
        }
    }

    /// <summary>获取指定音源下歌曲的歌词，返回 Lyric JSON。</summary>
    public static string GetLyric(string sourceId, string songId)
    {
        var a1 = AllocCString(sourceId);
        var a2 = AllocCString(songId);
        try
        {
            return CheckError(ReadAndFree(music_core_get_lyric(a1, a2)));
        }
        finally
        {
            if (a1 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a1);
            }
            if (a2 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a2);
            }
        }
    }

    /// <summary>获取指定音源的排行榜列表，返回 Leaderboard 数组 JSON。</summary>
    public static string GetLeaderboards(string sourceId)
    {
        var arg = AllocCString(sourceId);
        try
        {
            return CheckError(ReadAndFree(music_core_get_leaderboards(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    // =============================================================
    // 安全包装：飞牛 NAS
    // =============================================================

    /// <summary>登录飞牛 NAS，返回 {"token":...,"base_url":...}。</summary>
    public static string FeiniuLogin(string baseUrl, string username, string password)
    {
        var a1 = AllocCString(baseUrl);
        var a2 = AllocCString(username);
        var a3 = AllocCString(password);
        try
        {
            return CheckError(ReadAndFree(music_core_feiniu_login(a1, a2, a3)));
        }
        finally
        {
            if (a1 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a1);
            }
            if (a2 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a2);
            }
            if (a3 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a3);
            }
        }
    }

    /// <summary>列出飞牛 NAS 指定路径下的文件，返回 {"path":...,"files":[...]}。</summary>
    public static string FeiniuListFiles(string path)
    {
        var arg = AllocCString(path);
        try
        {
            return CheckError(ReadAndFree(music_core_feiniu_list_files(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>生成飞牛 NAS 文件的可流式播放 URL，返回 {"url":...}。</summary>
    public static string FeiniuStream(string path)
    {
        var arg = AllocCString(path);
        try
        {
            return CheckError(ReadAndFree(music_core_feiniu_stream(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>飞牛服务健康检查，返回 {"healthy":bool,"base_url":...}。</summary>
    public static string FeiniuHealth() => InvokeNoArg(music_core_feiniu_health);

    // =============================================================
    // 安全包装：协议源（SMB/WebDAV/FTP/DLNA/NFS）
    // =============================================================

    /// <summary>添加一个远程协议源，返回 {"id":...,"protocol":...,"root":...,"enabled":true,"placeholder":bool}。</summary>
    public static string ProtocolAdd(string configJson)
    {
        var arg = AllocCString(configJson);
        try
        {
            return CheckError(ReadAndFree(music_core_protocol_add(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>列出已加载的协议源，返回 {"sources":[...]}。</summary>
    public static string ProtocolList() => InvokeNoArg(music_core_protocol_list);

    /// <summary>删除指定 id 的协议源，返回 {"id":...,"deleted":bool}。</summary>
    public static string ProtocolDelete(string id)
    {
        var arg = AllocCString(id);
        try
        {
            return CheckError(ReadAndFree(music_core_protocol_delete(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>列出协议源下指定路径的条目名称，返回 {"path":...,"entries":[...]}。</summary>
    public static string ProtocolListFiles(string id, string path)
    {
        var a1 = AllocCString(id);
        var a2 = AllocCString(path);
        try
        {
            return CheckError(ReadAndFree(music_core_protocol_list_files(a1, a2)));
        }
        finally
        {
            if (a1 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a1);
            }
            if (a2 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a2);
            }
        }
    }

    /// <summary>读取协议源下指定文件为字节，返回 {"size":N,"data_base64":"..."}。</summary>
    public static string ProtocolRead(string id, string path)
    {
        var a1 = AllocCString(id);
        var a2 = AllocCString(path);
        try
        {
            return CheckError(ReadAndFree(music_core_protocol_read(a1, a2)));
        }
        finally
        {
            if (a1 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a1);
            }
            if (a2 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a2);
            }
        }
    }

    /// <summary>生成协议源下指定文件的可流式播放 URL，返回 {"url":...}。</summary>
    public static string ProtocolStream(string id, string path)
    {
        var a1 = AllocCString(id);
        var a2 = AllocCString(path);
        try
        {
            return CheckError(ReadAndFree(music_core_protocol_stream(a1, a2)));
        }
        finally
        {
            if (a1 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a1);
            }
            if (a2 != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(a2);
            }
        }
    }

    // =============================================================
    // 安全包装：本地音乐
    // =============================================================

    /// <summary>初始化本地音乐源（打开/创建 SQLite 索引库），返回 {"ok":true}。</summary>
    public static string LocalInit(string dbPath)
    {
        var arg = AllocCString(dbPath);
        try
        {
            return CheckError(ReadAndFree(music_core_local_init(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>添加本地扫描目录并递归扫描入库，返回 {"ok":true}。</summary>
    public static string LocalAddDir(string dir)
    {
        var arg = AllocCString(dir);
        try
        {
            return CheckError(ReadAndFree(music_core_local_add_dir(arg)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>重新扫描所有已添加目录（增量更新），返回 {"ok":true}。</summary>
    public static string LocalRescan() => InvokeNoArg(music_core_local_rescan);

    /// <summary>返回本地扫描进度，返回 {"current_count":N,"scanning":bool}。</summary>
    public static string LocalProgress() => InvokeNoArg(music_core_local_progress);

    // =============================================================
    // 安全包装：缓存
    // =============================================================

    /// <summary>初始化播放缓存管理器，返回 {"ok":true}。</summary>
    public static string CacheInit(string cacheDir, ulong maxBytes)
    {
        var arg = AllocCString(cacheDir);
        try
        {
            return CheckError(ReadAndFree(music_core_cache_init(arg, maxBytes)));
        }
        finally
        {
            if (arg != IntPtr.Zero)
            {
                Marshal.FreeHGlobal(arg);
            }
        }
    }

    /// <summary>返回缓存统计，返回 {"entries":N,"total_bytes":N,"max_bytes":N}。</summary>
    public static string CacheStats() => InvokeNoArg(music_core_cache_stats);

    /// <summary>清空所有缓存文件与索引，返回 {"ok":true}。</summary>
    public static string CacheClear() => InvokeNoArg(music_core_cache_clear);
}
