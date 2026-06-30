//! DLNA 协议客户端（占位实现）。
//!
//! DLNA 基于 UPnP，涉及 SSDP 设备发现、SOAP 内容目录浏览（Browse 动作）
//! 与媒体传输控制，协议栈复杂。当前为骨架实现，调用时返回明确错误。
//!
//! 生产实现路径：集成 `dlna-rs` 或类似 UPnP/DLNA 库，完成
//! SSDP 发现 → 获取 ContentDirectory 服务 → Browse 动作 → 解析 DIDL-Lite。
//! 该过程需要异步 SSDP 多播与 SOAP HTTP，生态库选型与稳定性需谨慎评估。

use async_trait::async_trait;

use crate::error::{CoreError, Result};
use crate::protocols::ProtocolClient;

/// 占位提示信息，含“占位”关键词，便于测试断言。
const PLACEHOLDER_MSG: &str =
    "DLNA 协议需集成 dlna-rs/UPnP 库（SSDP 发现 + SOAP 控制），当前为占位实现";

/// DLNA 协议客户端（占位）。
pub struct DlnaClient {
    /// 设备控制 URL
    pub control_url: String,
}

impl DlnaClient {
    /// 构造 DLNA 客户端。
    pub fn new(control_url: String) -> Self {
        Self { control_url }
    }
}

#[async_trait]
impl ProtocolClient for DlnaClient {
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

    fn client() -> DlnaClient {
        DlnaClient::new("http://dlna.local:8200/MediaServer/Control".into())
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
