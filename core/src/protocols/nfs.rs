//! NFS 协议客户端（占位实现）。
//!
//! NFS 客户端在 Rust 生态中较弱（缺少成熟的纯 Rust NFS 客户端库），
//! 生产实现通常依赖系统挂载（`mount -t nfs`）或 ONC RPC 绑定。
//! 当前为骨架实现，调用时返回明确错误。
//!
//! 生产实现路径：
//! 1. 通过系统挂载 NFS 导出后按本地文件访问（最稳妥，但需 root/特权）；
//! 2. 或引入 NFSv3 ONC RPC 客户端库（Rust 生态不成熟，需谨慎评估）。

use async_trait::async_trait;

use crate::error::{CoreError, Result};
use crate::protocols::ProtocolClient;

/// 占位提示信息，含“占位”关键词，便于测试断言。
const PLACEHOLDER_MSG: &str =
    "NFS 协议需系统挂载或 ONC RPC 客户端库（Rust 生态较弱），当前为占位实现";

/// NFS 协议客户端（占位）。
pub struct NfsClient {
    /// 服务器地址
    pub server: String,
    /// 导出路径
    pub export: String,
}

impl NfsClient {
    /// 构造 NFS 客户端。
    pub fn new(server: String, export: String) -> Self {
        Self { server, export }
    }
}

#[async_trait]
impl ProtocolClient for NfsClient {
    async fn list(&self, _path: &str) -> Result<Vec<String>> {
        Err(CoreError::Protocol(PLACEHOLDER_MSG.into()))
    }

    async fn read(&self, _path: &str) -> Result<Vec<u8>> {
        Err(CoreError::Protocol(PLACEHOLDER_MSG.into()))
    }

    async fn stream_url(&self, _path: &str) -> Result<String> {
        Err(CoreError::Protocol(PLACEHOLDER_MSG.into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> NfsClient {
        NfsClient::new("nas.local".into(), "/export/music".into())
    }

    #[tokio::test]
    async fn list_returns_placeholder_error() {
        let err = client().list("/").await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("占位") || msg.contains("需启用"));
    }

    #[tokio::test]
    async fn read_returns_placeholder_error() {
        let err = client().read("/a.mp3").await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("占位") || msg.contains("需启用"));
    }
}
