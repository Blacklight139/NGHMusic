//! 收藏夹：多分组、添加/移除、导入/导出。

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use crate::models::*;

pub struct FavoriteStore {
    inner: Arc<RwLock<HashMap<String, FavoriteGroup>>>,
}

impl Default for FavoriteStore {
    fn default() -> Self {
        Self::new()
    }
}

impl FavoriteStore {
    pub fn new() -> Self {
        let mut map = HashMap::new();
        let default = FavoriteGroup::new("我喜欢的音乐");
        map.insert(default.id.clone(), default);
        Self {
            inner: Arc::new(RwLock::new(map)),
        }
    }

    pub fn create_group(&self, name: impl Into<String>) -> FavoriteGroup {
        let g = FavoriteGroup::new(name);
        self.inner.write().insert(g.id.clone(), g.clone());
        g
    }

    pub fn list_groups(&self) -> Vec<FavoriteGroup> {
        self.inner.read().values().cloned().collect()
    }

    pub fn delete_group(&self, id: &str) -> bool {
        self.inner.write().remove(id).is_some()
    }

    pub fn add(&self, group_id: &str, song: SongRef) -> Option<bool> {
        let mut g = self.inner.write();
        let grp = g.get_mut(group_id)?;
        if grp.songs.contains(&song) {
            return Some(false);
        }
        grp.songs.push(song);
        Some(true)
    }

    pub fn remove(&self, group_id: &str, song: &SongRef) -> Option<bool> {
        let mut g = self.inner.write();
        let grp = g.get_mut(group_id)?;
        let before = grp.songs.len();
        grp.songs.retain(|s| s != song);
        Some(grp.songs.len() != before)
    }

    /// 导出全部为 JSON
    pub fn export(&self) -> serde_json::Value {
        let groups: Vec<FavoriteGroup> = self.list_groups();
        serde_json::to_value(groups).unwrap_or(serde_json::Value::Null)
    }

    /// 从 JSON 导入（覆盖）
    pub fn import(&self, json: &str) -> CoreResult<()> {
        let groups: Vec<FavoriteGroup> = serde_json::from_str(json)?;
        let mut g = self.inner.write();
        g.clear();
        for grp in groups {
            g.insert(grp.id.clone(), grp);
        }
        Ok(())
    }
}

use crate::CoreResult;
