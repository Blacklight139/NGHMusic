//! 跨音源聚合搜索

use crate::models::*;
use crate::sources::SourceEngine;
use crate::sources::http_source;
use crate::CoreResult;

pub struct SearchService<'a> {
    pub engine: &'a SourceEngine,
}

impl<'a> SearchService<'a> {
    pub fn new(engine: &'a SourceEngine) -> Self {
        Self { engine }
    }

    /// 跨已启用音源并发搜索
    pub async fn search(
        &self,
        keyword: &str,
        ty: SearchType,
        page: Page,
    ) -> CoreResult<Paged<SearchResult>> {
        let handles = self.engine.enabled_sources();
        let mut tasks = Vec::with_capacity(handles.len());
        for h in handles {
            let ep = h.config.endpoints.search.clone();
            if let Some(ep) = ep {
                let http = self.engine.http_client();
                let cfg = h.config.clone();
                let kw = keyword.to_string();
                tasks.push(tokio::spawn(async move {
                    let subs: [(&str, &str); 2] = [
                        ("keyword", kw.as_str()),
                        ("page", &page.offset.to_string()),
                    ];
                    match http_source::request_endpoint(&http, &cfg, &ep, &subs).await {
                        Ok(val) => {
                            let arr = match val {
                                serde_json::Value::Array(a) => a,
                                serde_json::Value::Object(_) => vec![val],
                                _ => Vec::new(),
                            };
                            let mut items = Vec::with_capacity(arr.len());
                            for v in arr {
                                match ty {
                                    SearchType::Song => {
                                        if let Ok(s) = crate::sources::engine::map_song(
                                            &cfg.manifest.id,
                                            &cfg.field_mapping,
                                            &v,
                                        ) {
                                            items.push(SearchResult {
                                                source_id: cfg.manifest.id.clone(),
                                                source_name: cfg.manifest.name.clone(),
                                                item: SearchItem::Song(s),
                                            });
                                        }
                                    }
                                    SearchType::Album => {
                                        if let Some(name) = v.get("name").and_then(|x| x.as_str()) {
                                            items.push(SearchResult {
                                                source_id: cfg.manifest.id.clone(),
                                                source_name: cfg.manifest.name.clone(),
                                                item: SearchItem::Album(Album {
                                                    source_id: cfg.manifest.id.clone(),
                                                    album_id: v
                                                        .get("id")
                                                        .and_then(|x| x.as_str())
                                                        .unwrap_or("")
                                                        .into(),
                                                    name: name.into(),
                                                    ..Default::default()
                                                }),
                                            });
                                        }
                                    }
                                    SearchType::Artist => {
                                        if let Some(name) = v.get("name").and_then(|x| x.as_str()) {
                                            items.push(SearchResult {
                                                source_id: cfg.manifest.id.clone(),
                                                source_name: cfg.manifest.name.clone(),
                                                item: SearchItem::Artist(Artist {
                                                    source_id: cfg.manifest.id.clone(),
                                                    artist_id: v
                                                        .get("id")
                                                        .and_then(|x| x.as_str())
                                                        .unwrap_or("")
                                                        .into(),
                                                    name: name.into(),
                                                    ..Default::default()
                                                }),
                                            });
                                        }
                                    }
                                }
                            }
                            (cfg.manifest.id.clone(), items)
                        }
                        Err(e) => {
                            tracing::warn!(?e, "音源搜索失败");
                            (cfg.manifest.id.clone(), Vec::new())
                        }
                    }
                }));
            }
        }
        let mut merged = Vec::new();
        for t in tasks {
            if let Ok((_id, items)) = t.await {
                merged.extend(items);
            }
        }
        // 分页（合并后再分页）
        let total = merged.len() as u32;
        let start = (page.offset as usize).min(merged.len());
        let end = (start + page.limit as usize).min(merged.len());
        let items: Vec<SearchResult> = merged.drain(start..end).collect();
        Ok(Paged {
            items,
            total,
            offset: page.offset,
            limit: page.limit,
        })
    }
}

impl SourceEngine {
    pub fn http_client(&self) -> reqwest::Client {
        self.http.clone()
    }
}
