//! 播放缓存层：LRU 策略，命中优先本地播放。

use parking_lot::Mutex;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use crate::models::SongRef;
use crate::{CoreError, CoreResult};

pub struct CacheStore {
    dir: PathBuf,
    /// 最大容量（字节）
    max_bytes: u64,
    state: Mutex<State>,
}

struct State {
    /// song key -> cache entry
    entries: HashMap<String, Entry>,
    /// 已用字节
    used: u64,
}

#[derive(Clone, Copy)]
struct Entry {
    size: u64,
    /// 最近访问时间（毫秒）
    last_access: u128,
}

impl CacheStore {
    pub async fn new(dir: PathBuf, max_bytes: u64) -> CoreResult<Self> {
        fs::create_dir_all(&dir).await.map_err(|e| CoreError::Cache(e.to_string()))?;
        Ok(Self {
            dir,
            max_bytes,
            state: Mutex::new(State {
                entries: HashMap::new(),
                used: 0,
            }),
        })
    }

    fn path_for(&self, song: &SongRef) -> PathBuf {
        let safe = song.key().replace(['/', '\\', ':', ' '], "_");
        self.dir.join(format!("{safe}.bin"))
    }

    /// 是否命中
    pub fn has(&self, song: &SongRef) -> bool {
        let key = song.key();
        let mut s = self.state.lock();
        if let Some(e) = s.entries.get_mut(&key) {
            e.last_access = now_ms();
            true
        } else {
            false
        }
    }

    /// 读取缓存到字节
    pub async fn read(&self, song: &SongRef) -> CoreResult<Vec<u8>> {
        let key = song.key();
        {
            let mut s = self.state.lock();
            if let Some(e) = s.entries.get_mut(&key) {
                e.last_access = now_ms();
            } else {
                return Err(CoreError::Cache(format!("未命中缓存: {key}")));
            }
        }
        let path = self.path_for(song);
        fs::read(&path)
            .await
            .map_err(|e| CoreError::Cache(format!("读取缓存失败: {e}")))
    }

    /// 写入缓存
    pub async fn write(&self, song: &SongRef, data: Vec<u8>) -> CoreResult<()> {
        let size = data.len() as u64;
        let path = self.path_for(song);
        fs::write(&path, &data)
            .await
            .map_err(|e| CoreError::Cache(format!("写入缓存失败: {e}")))?;
        let key = song.key();
        {
            let mut s = self.state.lock();
            if let Some(old) = s.entries.get(&key) {
                s.used = s.used.saturating_sub(old.size);
            }
            s.entries.insert(
                key.clone(),
                Entry {
                    size,
                    last_access: now_ms(),
                },
            );
            s.used += size;
        }
        self.evict().await?;
        Ok(())
    }

    /// 返回缓存文件路径（用于流式播放）
    pub fn cache_path(&self, song: &SongRef) -> PathBuf {
        self.path_for(song)
    }

    /// 当前已用 / 容量
    pub fn usage(&self) -> (u64, u64) {
        let s = self.state.lock();
        (s.used, self.max_bytes)
    }

    /// 清空所有缓存
    pub async fn clear(&self) -> CoreResult<()> {
        let keys: Vec<String> = {
            let mut s = self.state.lock();
            s.used = 0;
            s.entries.drain().map(|(k, _)| k).collect()
        };
        for k in keys {
            let _ = fs::remove_file(self.dir.join(format!("{}.bin", k.replace(['/', '\\', ':', ' '], "_")))).await;
        }
        Ok(())
    }

    /// LRU 淘汰到容量以内
    async fn evict(&self) -> CoreResult<()> {
        let to_remove: Vec<(String, Entry)> = {
            let s = self.state.lock();
            if s.used <= self.max_bytes {
                return Ok(());
            }
            let mut entries: Vec<(String, Entry)> =
                s.entries.iter().map(|(k, e)| (k.clone(), *e)).collect();
            entries.sort_by_key(|(_, e)| e.last_access);
            let mut freed = 0u64;
            let need = s.used.saturating_sub(self.max_bytes);
            entries
                .into_iter()
                .take_while(|(_, e)| {
                    let more = freed < need;
                    freed += e.size;
                    more
                })
                .map(|(k, e)| (k, e))
                .collect()
        };
        for (k, e) in to_remove {
            let path = self.dir.join(format!("{}.bin", k.replace(['/', '\\', ':', ' '], "_")));
            let _ = fs::remove_file(&path).await;
            let mut s = self.state.lock();
            s.entries.remove(&k);
            s.used = s.used.saturating_sub(e.size);
        }
        Ok(())
    }
}

fn now_ms() -> u128 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

pub type SharedCache = Arc<CacheStore>;
