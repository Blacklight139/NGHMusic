//! SMB 客户端（占位：实际接入原生实现或 pavao/smb-rs）

use async_trait::async_trait;
use crate::models::{SmbConfig, ProtocolEntry};
use crate::{CoreError, CoreResult};

pub struct SmbClient {
    cfg: SmbConfig,
}

impl SmbClient {
    pub fn new(cfg: SmbConfig) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl super::ProtocolClient for SmbClient {
    fn kind(&self) -> &'static str { "smb" }

    async fn list(&self, path: &str) -> CoreResult<Vec<ProtocolEntry>> {
        // SMB 实际需原生库；此处返回占位错误，提示接入原生绑定。
        Err(CoreError::Protocol {
            protocol: "smb".into(),
            message: format!(
                "SMB 需接入原生实现 ({}:{}) share={} path={}",
                self.cfg.host,
                self.cfg.port.unwrap_or(445),
                self.cfg.share,
                path
            ),
        })
    }
}
