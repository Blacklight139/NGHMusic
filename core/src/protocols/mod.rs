//! 网络协议客户端：SMB / WebDAV / FTP / DLNA / NFS
//!
//! 为各端提供统一的浏览接口。HTTP 类协议（WebDAV）基于 reqwest；
//! 其它协议抽象为 trait，实际绑定可按需接入原生实现。

use async_trait::async_trait;
use crate::models::{ProtocolSource, ProtocolEntry, SmbConfig, WebDavConfig, FtpConfig, DlnaConfig, NfsConfig};
use crate::{CoreError, CoreResult};

pub mod webdav;
pub mod smb;
pub mod ftp;
pub mod dlna;
pub mod nfs;

#[async_trait]
pub trait ProtocolClient: Send + Sync {
    /// 列出目录条目
    async fn list(&self, path: &str) -> CoreResult<Vec<ProtocolEntry>>;
    /// 协议标识
    fn kind(&self) -> &'static str;
}

/// 根据配置创建协议客户端
pub fn build(cfg: ProtocolSource) -> CoreResult<Box<dyn ProtocolClient>> {
    match cfg {
        ProtocolSource::Smb(c) => Ok(Box::new(smb::SmbClient::new(c))),
        ProtocolSource::WebDav(c) => Ok(Box::new(webdav::WebDavClient::new(c))),
        ProtocolSource::Ftp(c) => Ok(Box::new(ftp::FtpClient::new(c))),
        ProtocolSource::Dlna(c) => Ok(Box::new(dlna::DlnaClient::new(c))),
        ProtocolSource::Nfs(c) => Ok(Box::new(nfs::NfsClient::new(c))),
    }
}
