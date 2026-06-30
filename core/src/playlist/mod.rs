//! 播放列表管理

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use crate::models::*;

pub struct PlaylistStore {
    inner: Arc<RwLock<HashMap<String, Playlist>>>,
}

impl Default for PlaylistStore {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaylistStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create(&self, name: impl Into<String>) -> Playlist {
        let p = Playlist::new(name);
        self.inner.write().insert(p.id.clone(), p.clone());
        p
    }

    pub fn delete(&self, id: &str) -> bool {
        self.inner.write().remove(id).is_some()
    }

    pub fn rename(&self, id: &str, name: String) -> Option<()> {
        let mut g = self.inner.write();
        let p = g.get_mut(id)?;
        p.name = name;
        Some(())
    }

    pub fn list(&self) -> Vec<Playlist> {
        self.inner.read().values().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<Playlist> {
        self.inner.read().get(id).cloned()
    }

    pub fn add_song(&self, id: &str, song: SongRef) -> Option<()> {
        let mut g = self.inner.write();
        let p = g.get_mut(id)?;
        p.songs.push(song);
        Some(())
    }

    pub fn remove_song(&self, id: &str, idx: usize) -> Option<()> {
        let mut g = self.inner.write();
        let p = g.get_mut(id)?;
        if idx >= p.songs.len() {
            return None;
        }
        p.songs.remove(idx);
        Some(())
    }

    pub fn replace(&self, p: Playlist) {
        self.inner.write().insert(p.id.clone(), p);
    }
}
