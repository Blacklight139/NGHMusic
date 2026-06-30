//! 跨平台音乐播放器的共享 Rust 核心。
//!
//! 本 crate 提供数据模型、音源抽象、元数据、聚合搜索、缓存、
//! NAS 协议接入、飞牛 API 客户端以及 FFI 接口规范等共享能力，
//! 供各平台（桌面、移动、嵌入式）前端复用。

pub mod cache;
pub mod error;
pub mod feiniu;
pub mod ffi;
pub mod library;
pub mod lyric;
pub mod metadata;
pub mod models;
pub mod protocols;
pub mod search;
pub mod sources;

// 重新导出关键类型，便于外部一次性引用
pub use error::{CoreError, Result};
pub use models::*;
