//! 排行榜

use crate::models::*;
use crate::sources::SourceEngine;
use crate::sources::http_source;
use crate::{CoreError, CoreResult};

pub struct RankingService<'a> {
    pub engine: &'a SourceEngine,
}

impl<'a> RankingService<'a> {
    pub fn new(engine: &'a SourceEngine) -> Self {
        Self { engine }
    }

    /// 列出所有音源提供的排行榜元数据（不含歌曲详情）
    pub async fn list(&self) -> CoreResult<Vec<Ranking>> {
        let handles = self.engine.enabled_sources();
        let mut out = Vec::new();
        for h in handles {
            let Some(ep) = h.config.endpoints.ranking.clone() else { continue };
            let subs: [(&str, &str); 0] = [];
            match http_source::request_endpoint(&self.engine.http_client(), &h.config, &ep, &subs).await {
                Ok(val) => {
                    let arr: Vec<serde_json::Value> = match val {
                        serde_json::Value::Array(a) => a,
                        serde_json::Value::Object(_) => vec![val],
                        _ => continue,
                    };
                    for v in arr {
                        let id = v.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
                        let name = v.get("name").and_then(|x| x.as_str()).unwrap_or("").to_string();
                        let cover_url = v.get("cover").or_else(|| v.get("coverUrl")).and_then(|x| x.as_str()).map(String::from);
                        if !id.is_empty() {
                            out.push(Ranking {
                                source_id: h.config.manifest.id.clone(),
                                ranking_id: id,
                                name,
                                cover_url,
                                update_time: None,
                                songs: Vec::new(),
                            });
                        }
                    }
                }
                Err(e) => tracing::warn!(?e, "排行榜列表失败"),
            }
        }
        Ok(out)
    }

    /// 获取某排行榜详情（含歌曲）
    pub async fn detail(&self, ranking: &Ranking) -> CoreResult<Ranking> {
        let handle = self.engine.get(&ranking.source_id)?;
        let ep = handle
            .config
            .endpoints
            .ranking
            .as_ref()
            .ok_or_else(|| CoreError::NotFound("排行榜端点缺失".into()))?;
        let id = ranking.ranking_id.clone();
        let subs: [(&str, &str); 1] = [("rankingId", id.as_str())];
        let val = http_source::request_endpoint(&self.engine.http_client(), &handle.config, ep, &subs).await?;
        // 期望返回数组或 { songs: [] }
        let songs_val = val
            .get("songs")
            .cloned()
            .unwrap_or(val);
        let arr = match songs_val {
            serde_json::Value::Array(a) => a,
            serde_json::Value::Object(_) => vec![songs_val],
            _ => Vec::new(),
        };
        let mut songs = Vec::with_capacity(arr.len());
        for v in arr {
            if let Ok(s) = crate::sources::engine::map_song(&ranking.source_id, &handle.config.field_mapping, &v) {
                songs.push(s);
            }
        }
        Ok(Ranking {
            source_id: ranking.source_id.clone(),
            ranking_id: ranking.ranking_id.clone(),
            name: ranking.name.clone(),
            cover_url: ranking.cover_url.clone(),
            update_time: ranking.update_time.clone(),
            songs,
        })
    }
}
