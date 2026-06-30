//! FFI 接口规范占位。
//!
//! 该模块定义面向 C/移动端的 C-friendly 函数签名规范，供桌面（如 Qt）、
//! 移动（Kotlin/JNI、Swift）等宿主通过动态库（cdylib）调用核心能力。
//!
//! ## 内存所有权约定
//! - 凡返回 `*mut c_char` 的函数，调用方必须使用 `music_core_free_string` 释放。
//! - 传入核心的指针所有权不转移，核心不负责释放外部内存。
//! - 所有 FFI 入口均为 `extern "C"`，并以 `#[no_mangle]` 导出稳定符号。

use std::ffi::CString;
use std::os::raw::c_char;

/// 返回核心库版本字符串。
///
/// 调用方负责用 `music_core_free_string` 释放返回的指针。
///
/// 该函数不会 panic：版本号取自 `CARGO_PKG_VERSION`，由 Cargo 在编译期校验为
/// 合法 semver（不含 NUL）。即便极端情况下版本号含 NUL，也会回退为空 C 字符串
/// 而非触发 panic（panic 跨 FFI 边界是未定义行为）。
#[no_mangle]
pub extern "C" fn music_core_version() -> *mut c_char {
    let version = env!("CARGO_PKG_VERSION");
    // Cargo 保证 version 为合法 semver，不含 NUL；防御性回退为空串以避免 FFI panic
    let c_string = CString::new(version).unwrap_or_default();
    // 将所有权转移给调用方
    c_string.into_raw()
}

/// 释放由 `music_core_*` 系列函数返回的 C 字符串。
///
/// 传入 NULL 是安全空操作。调用方不应再使用已释放的指针。
///
/// # Safety
/// `ptr` 必须为 NULL 或由 `CString::into_raw` 产生（即本模块 `music_core_version`
/// 等返回的指针）。传入任何其他指针（包括由 `malloc`、栈地址或其它分配器产生的指针）
/// 均会导致未定义行为。函数被标记为 `unsafe` 是为了在 Rust 侧强制调用方显式承诺
/// 该前置条件；C 侧调用本身不需额外的 unsafe 语法（C 无此概念），但语义上等价。
#[no_mangle]
pub unsafe extern "C" fn music_core_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    // 安全：调用方约定 ptr 来自本模块的 into_raw，重新构造并丢弃以回收内存
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}
