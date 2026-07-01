//! 音源抽象层。
//!
//! 定义统一的 `Source` trait，屏蔽不同音源（本地、JSON 配置、社区适配）的差异，
//! 上层通过该 trait 面向音源编程。使用 `async_trait` 保证 trait 对象安全，
//! 便于 `Box<dyn Source>` 动态分发。

use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::{CoreError, Result};
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

/// 音源展示信息：用于上层（桌面 Tauri command / 移动 FFI）查询音源列表。
///
/// 字段 `source_type` 取值：`"json"`（标准配置）/ `"community"`（社区格式适配而来）/
/// `"local"`（本地音乐源）。`description` 可选。
#[derive(Debug, Clone, Serialize)]
pub struct SourceInfo {
    /// 音源唯一标识
    pub id: String,
    /// 展示名
    pub name: String,
    /// 语义化版本号
    pub version: String,
    /// 是否启用
    pub enabled: bool,
    /// 音源类型："json" / "community" / "local"
    pub source_type: String,
    /// 优先级（数值越大越靠前）
    pub priority: i32,
    /// 描述（可选）
    pub description: Option<String>,
}

/// 持久化的音源运行时状态（仅 id/enabled/priority，便于跨重启恢复顺序与启停）。
///
/// `SourceManager` 自身持有 `Arc<dyn Source>` 无法序列化，故仅持久化顺序与启停状态；
/// 应用启动时重新加载各音源配置后，通过 [`SourceManager::apply_state`] 恢复顺序。
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SourceState {
    id: String,
    enabled: bool,
    priority: i32,
}

/// 音源管理器：加载/启用/禁用/优先级管理。
///
/// 内部维护 `Entry { source, info }` 列表，`info` 持有展示用的 `SourceInfo`
/// （其中 enabled/priority 是顺序与启停的唯一来源）。`ordered` 为按 priority 降序、
/// 仅含启用项的缓存视图，便于上层通过 `ordered_sources()` 顺序遍历调用。
///
/// 持久化：`persist_path` 设置后，所有变更（增删/排序/启停）会写回该路径指向的
/// `sources_state.json`（内容为 `Vec<SourceState>`）。应用启动时调用
/// [`SourceManager::set_persistence_path`] 即可加载并应用上次保存的顺序与启停。
pub struct SourceManager {
    entries: Vec<Entry>,
    ordered: Vec<Arc<dyn Source>>,
    persist_path: Option<PathBuf>,
}

struct Entry {
    source: Arc<dyn Source>,
    info: SourceInfo,
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            ordered: Vec::new(),
            persist_path: None,
        }
    }

    /// 追加一个音源，默认启用，按给定优先级参与排序。
    ///
    /// 该方法不携带 version/source_type/description 元信息，故 `SourceInfo`
    /// 中 version 置空、source_type 置 `"local"`、description 置 `None`。
    /// 如需完整元信息，使用 [`SourceManager::add_source_with_info`]。
    pub fn add_source(&mut self, source: Arc<dyn Source>, priority: i32) {
        let info = SourceInfo {
            id: source.id().to_string(),
            name: source.name().to_string(),
            version: String::new(),
            enabled: true,
            source_type: "local".to_string(),
            priority,
            description: None,
        };
        self.add_source_with_info(source, info);
    }

    /// 追加一个音源并附带完整展示元信息（用于 JSON/社区音源导入）。
    pub fn add_source_with_info(&mut self, source: Arc<dyn Source>, info: SourceInfo) {
        self.entries.push(Entry { source, info });
        self.rebuild_ordered();
        let _ = self.persist();
    }

    /// 按 id 移除音源；不存在则无操作。（兼容旧调用方，不返回错误）
    pub fn remove_source(&mut self, id: &str) {
        self.entries.retain(|e| e.source.id() != id);
        self.rebuild_ordered();
        let _ = self.persist();
    }

    /// 启用指定 id 的音源；不存在则无操作。
    pub fn enable(&mut self, id: &str) {
        let _ = self.set_source_enabled(id, true);
    }

    /// 禁用指定 id 的音源；不存在则无操作。
    pub fn disable(&mut self, id: &str) {
        let _ = self.set_source_enabled(id, false);
    }

    /// 设置音源启停状态；id 不存在返回 `NotFound` 错误。变更会持久化。
    pub fn set_source_enabled(&mut self, id: &str, enabled: bool) -> Result<()> {
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.source.id() == id)
            .ok_or_else(|| CoreError::NotFound(format!("音源不存在: {id}")))?;
        if entry.info.enabled != enabled {
            entry.info.enabled = enabled;
            self.rebuild_ordered();
        }
        self.persist()
    }

    /// 更新单个音源优先级；id 不存在返回 `NotFound` 错误。变更会持久化。
    pub fn update_source_priority(&mut self, id: &str, new_priority: i32) -> Result<()> {
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.source.id() == id)
            .ok_or_else(|| CoreError::NotFound(format!("音源不存在: {id}")))?;
        entry.info.priority = new_priority;
        self.rebuild_ordered();
        self.persist()
    }

    /// 按给定 id 顺序重排音源优先级（越靠前优先级越高）。任一 id 不存在返回错误。变更会持久化。
    ///
    /// 未出现在 `ordered_ids` 中的音源保持原优先级不变。
    pub fn reorder_sources(&mut self, ordered_ids: &[String]) -> Result<()> {
        let len = ordered_ids.len() as i32;
        for (i, id) in ordered_ids.iter().enumerate() {
            let priority = len - i as i32;
            let entry = self
                .entries
                .iter_mut()
                .find(|e| &e.info.id == id)
                .ok_or_else(|| CoreError::NotFound(format!("音源不存在: {id}")))?;
            entry.info.priority = priority;
        }
        self.rebuild_ordered();
        self.persist()
    }

    /// 删除指定 id 的音源（从内存与持久化存储移除）；id 不存在返回 `NotFound` 错误。
    pub fn delete_source(&mut self, id: &str) -> Result<()> {
        let before = self.entries.len();
        self.entries.retain(|e| e.source.id() != id);
        if self.entries.len() == before {
            return Err(CoreError::NotFound(format!("音源不存在: {id}")));
        }
        self.rebuild_ordered();
        self.persist()
    }

    /// 从 JSON 字符串导入音源：社区格式适配 → schema 严格校验 → 构造 `JsonSource` → 注册。
    ///
    /// 若 id 已存在则先移除旧的同 id 音源（视为覆盖导入）。新导入的音源默认启用，
    /// 优先级取现有最大优先级 + 1（即列表最前），便于导入后立即可用。
    /// 返回新音源的 `SourceInfo`。变更会持久化。
    pub fn import_source_from_json(&mut self, json_str: &str) -> Result<SourceInfo> {
        let raw: serde_json::Value = serde_json::from_str(json_str)?;
        let report = crate::sources::community::adapt_with_report(&raw)?;
        crate::sources::schema::SoundSourceConfig::validate_strict(&report.config)?;
        let config_str = serde_json::to_string(&report.config)?;
        let config = crate::sources::schema::SoundSourceConfig::from_json(&config_str)?;

        let source_type = match report.source_format.as_str() {
            "community-a" | "community-b" => "community",
            _ => "json",
        };

        let id = config.manifest.id.clone();
        // 覆盖同 id 旧音源
        self.entries.retain(|e| e.source.id() != id);

        let priority = self.next_priority();
        let info = SourceInfo {
            id: config.manifest.id.clone(),
            name: config.manifest.name.clone(),
            version: config.manifest.version.clone(),
            enabled: true,
            source_type: source_type.to_string(),
            priority,
            description: config.manifest.description.clone(),
        };
        let source: Arc<dyn Source> =
            Arc::new(crate::sources::json::JsonSource::new(config)?);
        self.entries.push(Entry {
            source,
            info: info.clone(),
        });
        self.rebuild_ordered();
        self.persist()?;
        Ok(info)
    }

    /// 列出所有音源：(id, name, enabled, priority)。
    pub fn list(&self) -> Vec<(&str, &str, bool, i32)> {
        self.entries
            .iter()
            .map(|e| (e.info.id.as_str(), e.info.name.as_str(), e.info.enabled, e.info.priority))
            .collect()
    }

    /// 返回按 priority 降序排列的全部音源信息（含启停项），用于上层展示音源列表。
    pub fn list_sources_ordered(&self) -> Vec<SourceInfo> {
        let mut all: Vec<&SourceInfo> = self.entries.iter().map(|e| &e.info).collect();
        all.sort_by(|a, b| b.priority.cmp(&a.priority));
        all.into_iter().cloned().collect()
    }

    /// 返回按 priority 降序的启用音源视图。
    pub fn ordered_sources(&self) -> &[Arc<dyn Source>] {
        &self.ordered
    }

    /// 设置持久化路径并加载已保存的顺序/启停状态应用到当前已注册音源。
    ///
    /// 应用启动流程：先 `add_source` / `import_source_from_json` 注册各音源，
    /// 再调用本方法传入持久化文件路径，即可恢复上次的顺序与启停。后续任何变更
    /// 都会自动写回该文件。文件不存在或解析失败时按空状态处理（不报错）。
    pub fn set_persistence_path(&mut self, path: PathBuf) {
        self.persist_path = Some(path.clone());
        if let Ok(states) = Self::load_state(&path) {
            self.apply_state(&states);
        }
    }

    /// 从文件加载持久化状态。文件不存在返回空列表。
    fn load_state(path: &Path) -> Result<Vec<SourceState>> {
        if !path.exists() {
            return Ok(Vec::new());
        }
        let text = std::fs::read_to_string(path)?;
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }
        let states: Vec<SourceState> = serde_json::from_str(&text)?;
        Ok(states)
    }

    /// 将持久化状态应用到当前已注册音源：按 id 匹配，覆盖 enabled/priority；
    /// 未匹配的 state 项忽略，未在 state 中的音源保持原状。
    fn apply_state(&mut self, states: &[SourceState]) {
        for state in states {
            if let Some(e) = self.entries.iter_mut().find(|e| e.info.id == state.id) {
                e.info.enabled = state.enabled;
                e.info.priority = state.priority;
            }
        }
        self.rebuild_ordered();
    }

    /// 返回下一个可用优先级（现有最大优先级 + 1，空时为 1）。
    fn next_priority(&self) -> i32 {
        self.entries
            .iter()
            .map(|e| e.info.priority)
            .max()
            .unwrap_or(0)
            + 1
    }

    fn rebuild_ordered(&mut self) {
        let mut enabled: Vec<&Entry> =
            self.entries.iter().filter(|e| e.info.enabled).collect();
        enabled.sort_by(|a, b| b.info.priority.cmp(&a.info.priority));
        self.ordered = enabled.into_iter().map(|e| Arc::clone(&e.source)).collect();
    }

    /// 将当前顺序/启停状态写入持久化路径（未设置路径则无操作）。
    fn persist(&self) -> Result<()> {
        let path = match &self.persist_path {
            Some(p) => p,
            None => return Ok(()),
        };
        let states: Vec<SourceState> = self
            .entries
            .iter()
            .map(|e| SourceState {
                id: e.info.id.clone(),
                enabled: e.info.enabled,
                priority: e.info.priority,
            })
            .collect();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&states)?;
        std::fs::write(path, json)?;
        Ok(())
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

    /// 构造一份可通过 schema 严格校验的标准音源配置 JSON。
    fn standard_config_json(id: &str, name: &str) -> String {
        serde_json::json!({
            "manifest": {
                "id": id,
                "name": name,
                "version": "1.0.0",
                "author": "tester",
                "description": "desc"
            },
            "endpoints": {
                "search": { "url": "https://example.com/s" },
                "metadata": { "url": "https://example.com/m" },
                "playUrl": { "url": "https://example.com/p" }
            },
            "fieldMapping": {
                "song": { "id": "id", "title": "title" },
                "album": { "id": "id", "name": "name" },
                "artist": { "id": "id", "name": "name" },
                "lyric": { "lines": [{ "timeMs": "t", "text": "x" }] }
            }
        })
        .to_string()
    }

    /// 完整流程：导入 → 列表 → 排序 → 单独更新优先级 → 删除，并校验错误路径。
    #[test]
    fn source_manager_import_list_reorder_delete() {
        let mut sm = SourceManager::new();

        let info = sm
            .import_source_from_json(&standard_config_json("a", "Alpha"))
            .expect("导入标准音源应成功");
        assert_eq!(info.id, "a");
        assert_eq!(info.name, "Alpha");
        assert_eq!(info.version, "1.0.0");
        assert_eq!(info.source_type, "json");
        assert_eq!(info.description.as_deref(), Some("desc"));
        assert!(info.enabled);

        // 第二个导入优先级更高（max+1），排在列表首位
        sm.import_source_from_json(&standard_config_json("b", "Bravo"))
            .expect("导入第二个音源应成功");
        let ordered = sm.list_sources_ordered();
        assert_eq!(ordered.len(), 2);
        assert_eq!(ordered[0].id, "b");
        assert_eq!(ordered[1].id, "a");

        // 重排：把 a 放到最前
        sm.reorder_sources(&["a".to_string(), "b".to_string()])
            .expect("重排应成功");
        let ordered = sm.list_sources_ordered();
        assert_eq!(ordered[0].id, "a");
        assert_eq!(ordered[1].id, "b");

        // 单独更新优先级使 b 回到首位
        sm.update_source_priority("b", 100)
            .expect("更新优先级应成功");
        let ordered = sm.list_sources_ordered();
        assert_eq!(ordered[0].id, "b");

        // 启停：禁用 a 后启用视图仅剩 b
        sm.set_source_enabled("a", false).expect("禁用应成功");
        let ordered = sm.list_sources_ordered();
        let a = ordered.iter().find(|i| i.id == "a").unwrap();
        assert!(!a.enabled);
        assert_eq!(sm.ordered_sources().len(), 1);

        // 删除 b
        sm.delete_source("b").expect("删除应成功");
        let ordered = sm.list_sources_ordered();
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered[0].id, "a");

        // 错误路径：不存在的 id 应返回错误
        assert!(sm.delete_source("missing").is_err());
        assert!(sm.update_source_priority("missing", 1).is_err());
        assert!(sm.set_source_enabled("missing", true).is_err());
        assert!(sm
            .reorder_sources(&["missing".to_string()])
            .is_err());
    }

    /// 社区格式 B 导入：source_type 应为 "community"。
    #[test]
    fn source_manager_import_community_format() {
        let mut sm = SourceManager::new();
        let json = serde_json::json!({
            "name": "网易云",
            "search_url": "https://music.example.com/search",
            "song_url": "https://music.example.com/song"
        })
        .to_string();
        let info = sm
            .import_source_from_json(&json)
            .expect("社区格式应导入成功");
        assert_eq!(info.source_type, "community");
        // 中文名 slug 回退为 "source" 以满足 schema pattern
        assert_eq!(info.id, "source");
        assert_eq!(sm.list_sources_ordered().len(), 1);

        // 重复导入同 id 视为覆盖，不新增
        sm.import_source_from_json(&json)
            .expect("重复导入应覆盖成功");
        assert_eq!(sm.list_sources_ordered().len(), 1);

        // 非法 JSON 应返回错误
        assert!(sm.import_source_from_json("not json").is_err());
    }

    /// 持久化往返：第一次会话写入顺序/启停，重启后新建 manager 应恢复。
    #[test]
    fn source_manager_persistence_roundtrip() {
        let tmp = tempfile::tempdir().expect("创建临时目录失败");
        let state_path = tmp.path().join("sources_state.json");

        // 第一次会话：注册音源 + 重排 + 禁用 + 设持久化路径 + 触发一次写入
        let mut sm = SourceManager::new();
        sm.import_source_from_json(&standard_config_json("a", "Alpha"))
            .unwrap();
        sm.import_source_from_json(&standard_config_json("b", "Bravo"))
            .unwrap();
        // 此时 b 在前；把 a 提到最前并禁用 b
        sm.reorder_sources(&["a".to_string(), "b".to_string()])
            .unwrap();
        sm.set_source_enabled("b", false).unwrap();
        sm.set_persistence_path(state_path.clone());
        // set_persistence_path 仅加载（文件不存在→空状态），需触发一次变更以写入文件
        sm.update_source_priority("a", 9).unwrap();
        assert!(state_path.exists(), "持久化文件应已生成");

        // 模拟重启：新建 manager，重新导入音源，再应用持久化状态恢复顺序/启停
        let mut sm2 = SourceManager::new();
        sm2.import_source_from_json(&standard_config_json("a", "Alpha"))
            .unwrap();
        sm2.import_source_from_json(&standard_config_json("b", "Bravo"))
            .unwrap();
        // 重启前 b 后导入优先级更高；应用持久化后应恢复 a 在前 + b 禁用
        sm2.set_persistence_path(state_path.clone());

        let ordered = sm2.list_sources_ordered();
        let a = ordered.iter().find(|i| i.id == "a").unwrap();
        let b = ordered.iter().find(|i| i.id == "b").unwrap();
        assert!(a.enabled, "a 应为启用");
        assert!(!b.enabled, "b 应为禁用");
        assert_eq!(a.priority, 9);
        assert!(
            a.priority > b.priority,
            "a 应排在 b 之前: a={} b={}",
            a.priority,
            b.priority
        );
    }
}
