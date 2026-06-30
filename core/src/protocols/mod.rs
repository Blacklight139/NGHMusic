//! NAS/远程协议抽象层。
//!
//! 统一封装 SMB、WebDAV、FTP、DLNA、NFS 等远程文件协议，
//! 上层通过 `ProtocolClient` trait 访问，屏蔽具体协议差异。
//! 各子模块为对应协议的占位实现，后续接入具体库。

use async_trait::async_trait;

use crate::error::Result;

pub mod dlna;
pub mod ftp;
pub mod nfs;
pub mod smb;
pub mod webdav;

/// 远程协议客户端统一抽象
#[async_trait]
pub trait ProtocolClient: Send + Sync {
    /// 列出指定路径下的条目名称
    async fn list(&self, path: &str) -> Result<Vec<String>>;
    /// 读取指定路径文件内容为字节
    async fn read(&self, path: &str) -> Result<Vec<u8>>;
    /// 生成可流式播放的 URL
    async fn stream_url(&self, path: &str) -> Result<String>;
}
