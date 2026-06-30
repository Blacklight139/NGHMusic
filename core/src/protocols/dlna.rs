//! DLNA 客户端（占位：接入原生或 UPnP 库实现）

use async_trait::async_trait;
use crate::models::{DlnaConfig, ProtocolEntry};
use crate::{CoreError, CoreResult};

pub struct DlnaClient {
    cfg: DlnaConfig,
}

impl DlnaClient {
    pub fn new(cfg: DlnaConfig) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl super::ProtocolClient for DlnaClient {
    fn kind(&self) -> &'static str { "dlna" }

    async fn list(&self, path: &str) -> CoreResult<Vec<ProtocolEntry>> {
        Err(CoreError::Protocol {
            protocol: "dlna".into(),
            message: format!("DLNA 需接入原生实现 ({}) path={}", self.cfg.device_url, path),
        })
    }
}
