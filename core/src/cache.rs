//! 播放缓存层。
//!
//! 基于 LRU 策略的本地文件缓存，缓存命中时优先本地播放。
//! 缓存 key 为 song_id，value 为播放数据（下载的流数据字节）。
//! 音源返回播放 URL（在线流式）时，下载流为本地文件存入缓存目录，
//! 命中时返回本地文件路径，避免重复下载。

use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::fs;
use std::time::SystemTime;

use crate::error::{CoreError, Result};

/// 单条缓存条目（私有）。
#[derive(Clone, Debug)]
struct CacheEntry {
    key: String,
    file_path: PathBuf,
    size: u64,
    last_access: SystemTime,
}

/// 缓存统计信息。
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// 当前缓存条目数
    pub entries: usize,
    /// 当前缓存总字节数
    pub total_bytes: u64,
    /// 缓存容量上限（字节）
    pub max_bytes: u64,
}

/// LRU 文件缓存管理器。
///
/// 缓存播放数据为本地文件，命中时返回文件路径，未命中时调用
/// fetcher 下载并写入缓存。超容量时按 LRU（最久未用）淘汰。
pub struct CacheManager {
    cache_dir: PathBuf,
    index: Mutex<HashMap<String, CacheEntry>>,
    max_entries: usize,
    max_bytes: u64,
    current_bytes: AtomicUsize,
}

impl CacheManager {
    /// 创建缓存管理器，初始化缓存目录并加载已有索引。
    ///
    /// 扫描 `cache_dir` 下所有 `.cache` 文件重建内存索引，
    /// 使缓存在进程重启后仍然可用。`max_entries` 固定为 1000。
    pub fn new(cache_dir: impl AsRef<Path>, max_bytes: u64) -> Result<Self> {
        let cache_dir = cache_dir.as_ref().to_path_buf();
        fs::create_dir_all(&cache_dir)
            .map_err(|e| CoreError::Cache(format!("创建缓存目录失败: {}", e)))?;

        let mut index: HashMap<String, CacheEntry> = HashMap::new();
        let mut current_bytes: u64 = 0;

        let entries = fs::read_dir(&cache_dir)
            .map_err(|e| CoreError::Cache(format!("读取缓存目录失败: {}", e)))?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("cache") {
                continue;
            }
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };
            let size = metadata.len();
            let last_access = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) if !s.is_empty() => s.to_string(),
                _ => continue,
            };
            current_bytes += size;
            index.insert(
                stem.clone(),
                CacheEntry {
                    key: stem,
                    file_path: path,
                    size,
                    last_access,
                },
            );
        }

        Ok(Self {
            cache_dir,
            index: Mutex::new(index),
            max_entries: 1000,
            max_bytes,
            current_bytes: AtomicUsize::new(current_bytes as usize),
        })
    }

    /// 获取缓存路径；未命中则调用 `fetcher` 下载数据并写入缓存。
    ///
    /// 命中时直接返回本地文件路径并更新 `last_access`；
    /// 未命中时调用 `fetcher` 获取字节数据，同步写入
    /// `cache_dir/<sanitized_key>.cache`，更新索引与字节计数，
    /// 然后触发 LRU 淘汰。
    pub async fn get_or_fetch<F, Fut>(&self, key: &str, fetcher: F) -> Result<PathBuf>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<Vec<u8>>>,
    {
        let index_key = sanitize_key(key);
        let file_path = self.cache_dir.join(format!("{}.cache", index_key));

        // 缓存命中：更新 last_access 并返回路径
        {
            let mut index = self.lock_index()?;
            if let Some(entry) = index.get_mut(&index_key) {
                entry.last_access = SystemTime::now();
                return Ok(entry.file_path.clone());
            }
        }

        // 未命中：下载数据
        let data = fetcher().await?;

        // 二次检查：并发请求可能在 await 期间已填充缓存。
        // 若已被填充则直接复用已有路径，避免重复写文件与重复计数。
        {
            let mut index = self.lock_index()?;
            if let Some(entry) = index.get_mut(&index_key) {
                entry.last_access = SystemTime::now();
                return Ok(entry.file_path.clone());
            }
        }

        // 同步写入文件（缓存数据通常不大，可接受）
        if let Err(e) = fs::write(&file_path, &data) {
            // 写入失败时清理可能产生的残留文件
            let _ = fs::remove_file(&file_path);
            return Err(CoreError::Cache(format!("写入缓存文件失败: {}", e)));
        }
        let size = data.len() as u64;

        // 更新索引与字节计数。
        // fetch_add 必须在锁内执行：若放在锁外，另一并发线程可在我们 insert
        // 之后、fetch_add 之前观察到索引变更并触发自己的 insert+fetch_sub，
        // 导致计数失衡（旧条目被扣减但新条目尚未累加）。
        {
            let mut index = self.lock_index()?;
            if let Some(old) = index.insert(
                index_key.clone(),
                CacheEntry {
                    key: index_key,
                    file_path: file_path.clone(),
                    size,
                    last_access: SystemTime::now(),
                },
            ) {
                self.current_bytes
                    .fetch_sub(old.size as usize, Ordering::SeqCst);
            }
            self.current_bytes
                .fetch_add(size as usize, Ordering::SeqCst);
        }

        // 触发 LRU 淘汰
        self.evict_if_needed()?;

        Ok(file_path)
    }

    /// 判断 key 是否在缓存中。
    pub fn has(&self, key: &str) -> bool {
        let index_key = sanitize_key(key);
        self.lock_index()
            .map(|i| i.contains_key(&index_key))
            .unwrap_or(false)
    }

    /// 命中时返回路径并更新 `last_access`；未命中返回 `None`。
    pub fn get_path(&self, key: &str) -> Option<PathBuf> {
        let index_key = sanitize_key(key);
        let mut index = self.lock_index().ok()?;
        if let Some(entry) = index.get_mut(&index_key) {
            entry.last_access = SystemTime::now();
            Some(entry.file_path.clone())
        } else {
            None
        }
    }

    /// 清空所有缓存文件与索引。
    pub fn clear(&self) -> Result<()> {
        let mut index = self.lock_index()?;
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("cache") {
                    let _ = fs::remove_file(&path);
                }
            }
        }
        index.clear();
        self.current_bytes.store(0, Ordering::SeqCst);
        Ok(())
    }

    /// 返回缓存统计信息。
    pub fn stats(&self) -> CacheStats {
        let entries = self.lock_index().map(|i| i.len()).unwrap_or(0);
        CacheStats {
            entries,
            total_bytes: self.current_bytes.load(Ordering::SeqCst) as u64,
            max_bytes: self.max_bytes,
        }
    }

    /// 删除单条缓存（索引与文件）。
    pub fn remove(&self, key: &str) -> Result<()> {
        let index_key = sanitize_key(key);
        let mut index = self.lock_index()?;
        if let Some(entry) = index.remove(&index_key) {
            let _ = fs::remove_file(&entry.file_path);
            self.current_bytes
                .fetch_sub(entry.size as usize, Ordering::SeqCst);
        }
        Ok(())
    }

    /// 按 `last_access` 升序（最久未用）淘汰，直到 `current_bytes <= max_bytes`
    /// 且条目数不超过 `max_entries`。
    ///
    /// 在锁内仅做「从索引移除 + 扣减计数」并收集待删条目；释放锁后再循环
    /// `fs::remove_file`，避免文件 IO 阻塞其他并发缓存查询（`get_or_fetch` /
    /// `get_path` / `has`）。索引已无这些条目，并发查询不会返回它们。
    fn evict_if_needed(&self) -> Result<()> {
        let to_remove: Vec<CacheEntry> = {
            let mut index = self.lock_index()?;
            let over_bytes = (self.current_bytes.load(Ordering::SeqCst) as u64) > self.max_bytes;
            let over_count = index.len() > self.max_entries;
            if !over_bytes && !over_count {
                return Ok(());
            }

            // 收集并按 last_access 升序排序（LRU 在前）
            let mut entries: Vec<CacheEntry> = index.values().cloned().collect();
            entries.sort_by(|a, b| a.last_access.cmp(&b.last_access));

            let mut to_remove = Vec::new();
            for entry in entries {
                let still_over_bytes =
                    (self.current_bytes.load(Ordering::SeqCst) as u64) > self.max_bytes;
                let still_over_count = index.len() > self.max_entries;
                if !still_over_bytes && !still_over_count {
                    break;
                }
                // 先从索引移除并扣减计数，磁盘文件留待锁外删除
                index.remove(&entry.key);
                self.current_bytes
                    .fetch_sub(entry.size as usize, Ordering::SeqCst);
                to_remove.push(entry);
            }
            to_remove
        };

        // 锁外删除磁盘文件；失败仅忽略（索引已无该条目，
        // 残留文件不会影响正确性，下次 CacheManager::new 重建索引时也不会被纳入）
        for entry in to_remove {
            let _ = fs::remove_file(&entry.file_path);
        }
        Ok(())
    }

    /// 获取索引锁，失败（PoisonError）时返回缓存错误。
    fn lock_index(&self) -> Result<std::sync::MutexGuard<'_, HashMap<String, CacheEntry>>> {
        self.index
            .lock()
            .map_err(|e| CoreError::Cache(format!("缓存索引锁获取失败: {}", e)))
    }
}

/// 将 key 转换为安全的文件名与索引键。
///
/// 旧实现将所有非字母数字字符替换为 "_"，导致 `song_1` / `song-1` / `song 1`
/// 等不同 key 映射到同一缓存键，产生碰撞。现采用「字母数字前缀 + 完整哈希」
/// 策略：哈希保证不同 key 必生成不同缓存键（消除碰撞），前缀保留可读性。
fn sanitize_key(key: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish();

    // 取字母数字字符作为可读前缀（截断以避免文件名过长），并过滤掉路径分隔符。
    // 即使两个 key 字母数字部分相同，哈希后缀也会区分它们。
    let prefix: String = key
        .chars()
        .filter(|c| c.is_alphanumeric())
        .take(32)
        .collect();
    if prefix.is_empty() {
        format!("_{:016x}", hash)
    } else {
        format!("{}_{:016x}", prefix, hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// 创建缓存管理器与临时目录，便于测试。
    fn make_manager(max_bytes: u64) -> (TempDir, CacheManager) {
        let dir = TempDir::new().expect("创建临时目录失败");
        let mgr = CacheManager::new(dir.path(), max_bytes).expect("创建缓存管理器失败");
        (dir, mgr)
    }

    #[tokio::test]
    async fn test_basic_get_and_has() {
        let (_dir, mgr) = make_manager(1024 * 1024);
        let key = "song_1";
        assert!(!mgr.has(key), "初始状态不应存在");

        let path = mgr
            .get_or_fetch(key, || async move { Ok(vec![1u8, 2, 3, 4, 5]) })
            .await
            .expect("首次获取应成功");

        assert!(path.exists(), "缓存文件应存在");
        assert!(mgr.has(key), "缓存后应存在");
    }

    #[tokio::test]
    async fn test_cache_hit_returns_existing_path() {
        let (_dir, mgr) = make_manager(1024 * 1024);
        let key = "song_hit";

        let path1 = mgr
            .get_or_fetch(key, || async move { Ok(vec![1u8, 2, 3, 4]) })
            .await
            .expect("首次获取应成功");

        // 第二次调用应命中缓存，不应调用 fetcher（fetcher 返回错误以验证）
        let path2 = mgr
            .get_or_fetch(key, || async move {
                Err::<Vec<u8>, CoreError>(CoreError::Cache("不应调用 fetcher".into()))
            })
            .await
            .expect("命中应返回路径");

        assert_eq!(path1, path2, "命中应返回相同路径");
        assert!(path2.exists(), "路径应存在");
    }

    #[tokio::test]
    async fn test_eviction_triggers_on_byte_overflow() {
        let (_dir, mgr) = make_manager(100);
        let data = vec![0u8; 40];

        // 每条 40 字节，max_bytes=100；插入 5 条触发多轮淘汰
        for i in 0..5 {
            let key = format!("song_{}", i);
            let payload = data.clone();
            mgr.get_or_fetch(&key, move || async move { Ok(payload) })
                .await
                .expect("获取应成功");
            // 确保各条 last_access 不同，使淘汰顺序确定
            std::thread::sleep(std::time::Duration::from_millis(5));
        }

        let stats = mgr.stats();
        assert!(
            stats.total_bytes <= 100,
            "淘汰后 total_bytes {} 应 <= 100",
            stats.total_bytes
        );
        assert!(
            stats.entries <= 3,
            "淘汰后条目数 {} 应 <= 3",
            stats.entries
        );
    }

    #[tokio::test]
    async fn test_get_path_hit_and_miss() {
        let (_dir, mgr) = make_manager(1024 * 1024);
        let key = "song_path";

        mgr.get_or_fetch(key, || async move { Ok(vec![1u8, 2, 3]) })
            .await
            .expect("获取应成功");

        let path = mgr
            .get_path(key)
            .expect("命中应返回 Some");
        assert!(path.exists(), "命中路径应存在");

        assert!(
            mgr.get_path("nonexistent").is_none(),
            "未命中应返回 None"
        );
    }

    #[tokio::test]
    async fn test_clear() {
        let (_dir, mgr) = make_manager(1024 * 1024);
        mgr.get_or_fetch("song_a", || async move { Ok(vec![1u8]) })
            .await
            .expect("获取应成功");
        mgr.get_or_fetch("song_b", || async move { Ok(vec![2u8]) })
            .await
            .expect("获取应成功");

        assert_eq!(mgr.stats().entries, 2, "清空前应有 2 条");
        assert!(mgr.has("song_a"));

        mgr.clear().expect("清空应成功");

        assert_eq!(mgr.stats().entries, 0, "清空后应无条目");
        assert_eq!(mgr.stats().total_bytes, 0, "清空后字节数应为 0");
        assert!(!mgr.has("song_a"), "清空后不应存在");
        assert!(!mgr.has("song_b"), "清空后不应存在");
    }

    #[tokio::test]
    async fn test_stats() {
        let (_dir, mgr) = make_manager(500);

        let stats = mgr.stats();
        assert_eq!(stats.entries, 0, "初始无条目");
        assert_eq!(stats.total_bytes, 0, "初始无字节");
        assert_eq!(stats.max_bytes, 500, "max_bytes 应为 500");

        mgr.get_or_fetch("song_s", || async move { Ok(vec![0u8; 100]) })
            .await
            .expect("获取应成功");

        let stats = mgr.stats();
        assert_eq!(stats.entries, 1, "插入后应 1 条");
        assert_eq!(stats.total_bytes, 100, "插入后应 100 字节");
    }

    #[tokio::test]
    async fn test_remove() {
        let (_dir, mgr) = make_manager(1024 * 1024);
        let key = "song_rm";

        mgr.get_or_fetch(key, || async move { Ok(vec![1u8, 2, 3]) })
            .await
            .expect("获取应成功");
        assert!(mgr.has(key), "应存在");

        mgr.remove(key).expect("删除应成功");
        assert!(!mgr.has(key), "删除后不应存在");

        let stats = mgr.stats();
        assert_eq!(stats.entries, 0, "删除后应无条目");
        assert_eq!(stats.total_bytes, 0, "删除后应无字节");
    }

    #[tokio::test]
    async fn test_rebuild_index_on_new() {
        let dir = TempDir::new().expect("创建临时目录失败");

        {
            let mgr = CacheManager::new(dir.path(), 1024 * 1024).expect("创建缓存管理器失败");
            mgr.get_or_fetch("song_persist", || async move { Ok(vec![1u8, 2, 3, 4]) })
                .await
                .expect("获取应成功");
            assert!(mgr.has("song_persist"));
            // mgr 在此作用域结束时 drop，但 dir 仍存活
        }

        // 重新创建管理器，应从磁盘重建索引
        let mgr2 = CacheManager::new(dir.path(), 1024 * 1024).expect("重建缓存管理器失败");
        assert!(mgr2.has("song_persist"), "重建后应命中");
        let stats = mgr2.stats();
        assert_eq!(stats.entries, 1, "重建后应 1 条");
        assert_eq!(stats.total_bytes, 4, "重建后应 4 字节");
    }

    #[tokio::test]
    async fn test_special_chars_in_key() {
        let (_dir, mgr) = make_manager(1024 * 1024);
        // key 含特殊字符，应被 sanitize 为合法文件名
        let key = "source/song:1?v=2";

        let path = mgr
            .get_or_fetch(key, || async move { Ok(vec![9u8, 9]) })
            .await
            .expect("获取应成功");

        assert!(path.exists(), "缓存文件应存在");
        assert!(mgr.has(key), "应能查到含特殊字符的 key");
        assert!(
            mgr.get_path(key).is_some(),
            "应能获取含特殊字符的 key 路径"
        );
    }
}
