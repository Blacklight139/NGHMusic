//! 元数据 API 客户端：聚合从各音源获取元数据，并对外提供统一接口。

use crate::models::*;
use crate::sources::SourceEngine;
use crate::CoreResult;

pub struct MetadataClient<'a> {
    pub engine: &'a SourceEngine,
}

impl<'a> MetadataClient<'a> {
    pub fn new(engine: &'a SourceEngine) -> Self {
        Self { engine }
    }

    /// 获取歌曲完整元数据
    pub async fn get_song(&self, song_ref: &SongRef) -> CoreResult<Song> {
        self.engine.fetch_song(song_ref).await
    }

    /// 批量预取（顺序执行，避免对音源造成过大压力）
    pub async fn get_songs(&self, refs: &[SongRef]) -> CoreResult<Vec<Song>> {
        let mut out = Vec::with_capacity(refs.len());
        for r in refs {
            match self.engine.fetch_song(r).await {
                Ok(s) => out.push(s),
                Err(e) => {
                    tracing::warn!(?e, "预取歌曲失败");
                }
            }
        }
        Ok(out)
    }
}
