//! NFS 客户端（占位：接入原生实现）

use async_trait::async_trait;
use crate::models::{NfsConfig, ProtocolEntry};
use crate::{CoreError, CoreResult};

pub struct NfsClient {
    cfg: NfsConfig,
}

impl NfsClient {
    pub fn new(cfg: NfsConfig) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl super::ProtocolClient for NfsClient {
    fn kind(&self) -> &'static str { "nfs" }

    async fn list(&self, path: &str) -> CoreResult<Vec<ProtocolEntry>> {
        Err(CoreError::Protocol {
            protocol: "nfs".into(),
            message: format!("NFS 需接入原生实现 ({}:{}/{}) path={}", self.cfg.host, self.cfg.export, self.cfg.path, path),
        })
    }
}
