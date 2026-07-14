using System;

namespace MusicCore.Native;

/// <summary>
/// 音乐核心 FFI 调用失败时抛出的异常，携带核心返回的错误类型与消息。
/// </summary>
/// <remarks>
/// <see cref="Kind"/> 与 Rust 侧 <c>CoreError</c> 变体名一一对应：
/// Io / Json / Http / Source / Schema / NotFound / Cache / Protocol / Feiniu / Ffi。
/// </remarks>
public sealed class MusicCoreException : Exception
{
    /// <summary>核心错误类型（与 CoreError 变体名一致）。</summary>
    public string Kind { get; }

    public MusicCoreException(string kind, string message)
        : base(message)
    {
        Kind = kind;
    }

    public override string ToString()
    {
        return $"[{Kind}] {Message}";
    }
}
