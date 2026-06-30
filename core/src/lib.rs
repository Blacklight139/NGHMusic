//! music-core: 跨平台音乐播放器共享核心
//!
//! 包含音源引擎、元数据 API 客户端、聚合搜索、播放缓存、
//! 协议客户端（SMB/WebDAV/FTP/DLNA/NFS）、飞牛 API 客户端，
//! 并经 FFI 暴露给桌面(Tauri)/iOS/Android/HarmonyOS 各端。

pub mod error;
pub mod models;
pub mod sources;
pub mod metadata;
pub mod search;
pub mod cache;
pub mod protocols;
pub mod feiniu;
pub mod player;
pub mod playlist;
pub mod favorites;
pub mod lyrics;
pub mod ranking;
pub mod storage;
pub mod ffi;

pub use error::{CoreError, CoreResult};
