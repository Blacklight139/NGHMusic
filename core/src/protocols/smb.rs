//! SMB 协议客户端（占位实现）。
//!
//! 真实 SMB 访问需通过 `pavao` crate（libsmbclient 的 Rust 绑定）或
//! 直接调用系统 `smbclient`。两者均依赖系统级 libsmbclient 共享库，
//! 引入跨平台构建复杂度，故当前提供骨架实现并在调用时返回明确错误。
//!
//! 生产实现路径：
//! 1. 在 `Cargo.toml` 启用 `pavao` feature（依赖 libsmbclient 系统库）；
//! 2. 用 [`SmbClient::new`] 持有连接参数；
//! 3. `list`/`read` 通过 pavao 打开 share 并枚举/读取文件；
//! 4. `stream_url` 无标准 URL 方案，建议先 `read` 落盘或转 HTTP 中转。

use async_trait::async_trait;

use crate::error::{CoreError, Result};
use crate::protocols::ProtocolClient;

/// 占位提示信息，含“占位”与“需启用”关键词，便于测试断言。
const PLACEHOLDER_MSG: &str =
    "SMB 协议需启用 pavao feature（依赖 libsmbclient 系统库），当前为占位实现";

/// SMB 协议客户端（占位）。
pub struct SmbClient {
    /// 服务器地址
    pub host: String,
    /// 共享名
    pub share: String,
    /// 登录用户名
    pub username: String,
    /// 登录密码
    pub password: String,
}

impl SmbClient {
    /// 构造 SMB 客户端。
    pub fn new(host: String, share: String, username: String, password: String) -> Self {
        Self {
            host,
            share,
            username,
            password,
        }
    }
}

#[async_trait]
impl ProtocolClient for SmbClient {
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

    fn client() -> SmbClient {
        SmbClient::new("host".into(), "share".into(), "user".into(), "pass".into())
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

    #[tokio::test]
    async fn stream_url_returns_placeholder_error() {
        let err = client().stream_url("/a.mp3").await.unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("占位") || msg.contains("需启用"));
    }
}
