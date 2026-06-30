//! FTP 客户端（占位：接入原生或 async-ftp 实现）

use async_trait::async_trait;
use crate::models::{FtpConfig, ProtocolEntry};
use crate::{CoreError, CoreResult};

pub struct FtpClient {
    cfg: FtpConfig,
}

impl FtpClient {
    pub fn new(cfg: FtpConfig) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl super::ProtocolClient for FtpClient {
    fn kind(&self) -> &'static str { "ftp" }

    async fn list(&self, path: &str) -> CoreResult<Vec<ProtocolEntry>> {
        Err(CoreError::Protocol {
            protocol: "ftp".into(),
            message: format!("FTP 需接入原生实现 ({}:{}) path={}", self.cfg.host, self.cfg.port.unwrap_or(21), path),
        })
    }
}
