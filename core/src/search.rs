//! 聚合搜索。
//!
//! `Aggregator` 持有多个音源，顺序查询后合并结果，
//! 单个音源失败将被记录为警告并跳过，不影响整体结果。

use crate::error::Result;
use crate::models::SearchResult;
use crate::sources::Source;

/// 多音源聚合搜索器
pub struct Aggregator {
    sources: Vec<Box<dyn Source>>,
}

impl Aggregator {
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
        }
    }

    /// 追加一个音源
    pub fn add_source(&mut self, source: Box<dyn Source>) {
        self.sources.push(source);
    }

    /// 聚合搜索：顺序查询各音源并合并结果。
    ///
    /// 单个音源失败将被记录为警告并跳过，不影响整体结果。
    pub async fn search(&self, keyword: &str, page: u32, page_size: u32) -> Result<SearchResult> {
        let mut songs = Vec::new();
        let mut albums = Vec::new();
        let mut artists = Vec::new();
        let mut total: u64 = 0;

        for source in &self.sources {
            match source.search(keyword, page, page_size).await {
                Ok(result) => {
                    songs.extend(result.songs);
                    albums.extend(result.albums);
                    artists.extend(result.artists);
                    total = total.saturating_add(result.total);
                }
                Err(e) => {
                    log::warn!("音源 {} 搜索失败，已跳过: {}", source.id(), e);
                }
            }
        }

        Ok(SearchResult {
            keyword: keyword.to_string(),
            songs,
            albums,
            artists,
            total,
            page,
            page_size,
        })
    }
}

impl Default for Aggregator {
    fn default() -> Self {
        Self::new()
    }
}
