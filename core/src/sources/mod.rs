//! 音源抽象层。
//!
//! 定义统一的 `Source` trait，屏蔽不同音源（本地、JSON 配置、社区适配）的差异，
//! 上层通过该 trait 面向音源编程。使用 `async_trait` 保证 trait 对象安全，
//! 便于 `Box<dyn Source>` 动态分发。

use std::sync::Arc;

use async_trait::async_trait;

use crate::error::Result;
use crate::models::{Leaderboard, Lyric, SearchResult, Song};

pub mod community;
pub mod json;
pub mod local;
pub mod schema;

/// 音源统一抽象。
///
/// 所有具体音源需实现该 trait，提供搜索、元数据、播放 URL、歌词、排行榜等能力。
/// 使用 `#[async_trait]` 将 async 方法脱糖为返回 `Box<dyn Future>` 的形式，
/// 使 trait 对象（`Box<dyn Source>`）可用，从而支持动态分发。
#[async_trait]
pub trait Source: Send + Sync {
    /// 音源标识
    fn id(&self) -> &str;
    /// 音源展示名
    fn name(&self) -> &str;
    /// 关键字搜索
    async fn search(&self, keyword: &str, page: u32, page_size: u32) -> Result<SearchResult>;
    /// 获取歌曲元数据
    async fn get_metadata(&self, song_id: &str) -> Result<Song>;
    /// 获取可播放 URL
    async fn get_play_url(&self, song_id: &str) -> Result<String>;
    /// 获取歌词
    async fn get_lyric(&self, song_id: &str) -> Result<Lyric>;
    /// 获取该音源提供的排行榜列表
    async fn get_leaderboards(&self) -> Result<Vec<Leaderboard>>;
}

/// 音源管理器：加载/启用/禁用/优先级管理。
///
/// 内部维护 `Entry { source, enabled, priority }` 列表，
/// `ordered` 为按 priority 降序、仅含启用项的缓存视图，便于上层
/// 通过 `ordered_sources()` 顺序遍历调用。
pub struct SourceManager {
    entries: Vec<Entry>,
    ordered: Vec<Arc<dyn Source>>,
}

struct Entry {
    source: Arc<dyn Source>,
    enabled: bool,
    priority: i32,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            ordered: Vec::new(),
        }
    }

    /// 追加一个音源，默认启用，按给定优先级参与排序。
    pub fn add_source(&mut self, source: Arc<dyn Source>, priority: i32) {
        self.entries.push(Entry {
            source,
            enabled: true,
            priority,
        });
        self.rebuild_ordered();
    }

    /// 按 id 移除音源；不存在则无操作。
    pub fn remove_source(&mut self, id: &str) {
        self.entries.retain(|e| e.source.id() != id);
        self.rebuild_ordered();
    }

    /// 启用指定 id 的音源；不存在则无操作。
    pub fn enable(&mut self, id: &str) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.source.id() == id) {
            if !e.enabled {
                e.enabled = true;
                self.rebuild_ordered();
            }
        }
    }

    /// 禁用指定 id 的音源；不存在则无操作。
    pub fn disable(&mut self, id: &str) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.source.id() == id) {
            if e.enabled {
                e.enabled = false;
                self.rebuild_ordered();
            }
        }
    }

    /// 列出所有音源：(id, name, enabled, priority)。
    pub fn list(&self) -> Vec<(&str, &str, bool, i32)> {
        self.entries
            .iter()
            .map(|e| (e.source.id(), e.source.name(), e.enabled, e.priority))
            .collect()
    }

    /// 返回按 priority 降序的启用音源视图。
    pub fn ordered_sources(&self) -> &[Arc<dyn Source>] {
        &self.ordered
    }

    fn rebuild_ordered(&mut self) {
        let mut enabled: Vec<&Entry> =
            self.entries.iter().filter(|e| e.enabled).collect();
        enabled.sort_by(|a, b| b.priority.cmp(&a.priority));
        self.ordered = enabled.into_iter().map(|e| Arc::clone(&e.source)).collect();
    }
}

impl Default for SourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod source_manager_tests {
    use super::*;
    use crate::error::Result;
    use crate::models::{Leaderboard, Lyric, SearchResult};
    use async_trait::async_trait;

    /// 测试用最小 Source 实现：仅 id/name 不同，trait 方法不被调用。
    struct MockSource {
        id: String,
        name: String,
    }

    #[async_trait]
    impl Source for MockSource {
        fn id(&self) -> &str {
            &self.id
        }
        fn name(&self) -> &str {
            &self.name
        }
        async fn search(&self, _: &str, _: u32, _: u32) -> Result<SearchResult> {
            Ok(SearchResult {
                keyword: String::new(),
                songs: Vec::new(),
                albums: Vec::new(),
                artists: Vec::new(),
                total: 0,
                page: 0,
                page_size: 0,
            })
        }
        async fn get_metadata(&self, _: &str) -> Result<crate::models::Song> {
            unimplemented!()
        }
        async fn get_play_url(&self, _: &str) -> Result<String> {
            unimplemented!()
        }
        async fn get_lyric(&self, _: &str) -> Result<Lyric> {
            unimplemented!()
        }
        async fn get_leaderboards(&self) -> Result<Vec<Leaderboard>> {
            Ok(Vec::new())
        }
    }

    fn mock(id: &str, name: &str) -> Arc<dyn Source> {
        Arc::new(MockSource {
            id: id.into(),
            name: name.into(),
        })
    }

    #[test]
    fn source_manager_empty() {
        let sm = SourceManager::new();
        assert!(sm.list().is_empty());
        assert!(sm.ordered_sources().is_empty());
    }

    #[test]
    fn source_manager_orders_by_priority_desc() {
        let mut sm = SourceManager::new();
        sm.add_source(mock("a", "A"), 1);
        sm.add_source(mock("b", "B"), 5);
        sm.add_source(mock("c", "C"), 3);

        let ordered = sm.ordered_sources();
        assert_eq!(ordered.len(), 3);
        assert_eq!(ordered[0].id(), "b");
        assert_eq!(ordered[1].id(), "c");
        assert_eq!(ordered[2].id(), "a");

        let list = sm.list();
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn source_manager_enable_disable() {
        let mut sm = SourceManager::new();
        sm.add_source(mock("a", "A"), 1);
        sm.add_source(mock("b", "B"), 5);

        sm.disable("b");
        let ordered = sm.ordered_sources();
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].id(), "a");

        // b 不存在时禁用/启用无副作用
        sm.disable("missing");
        sm.enable("missing");
        assert_eq!(sm.ordered_sources().len(), 1);

        sm.enable("b");
        let ordered = sm.ordered_sources();
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0].id(), "b");
    }

    #[test]
    fn source_manager_remove() {
        let mut sm = SourceManager::new();
        sm.add_source(mock("a", "A"), 1);
        sm.add_source(mock("b", "B"), 5);
        sm.add_source(mock("c", "C"), 3);

        sm.remove_source("c");
        let ordered = sm.ordered_sources();
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0].id(), "b");
        assert_eq!(ordered[1].id(), "a");

        // 再次删除已不存在的音源不应出错
        sm.remove_source("c");
        assert_eq!(sm.ordered_sources().len(), 2);
    }
}
