//! 本地音乐源：基于文件系统的音乐库索引与监听。
//!
//! 通过递归扫描用户指定的根目录，使用 lofty 读取音频元数据并持久化到 SQLite；
//! 同时使用 notify 监听目录变化，增量维护索引。实现 [`Source`](crate::sources::Source)
//! trait，将本地库接入聚合搜索与播放。
//!
//! ## 字段说明
//! - `db`：SQLite 连接。设计文档字面为 `Mutex<rusqlite::Connection>`，这里使用
//!   `Arc<Mutex<Connection>>` 包裹——notify watcher 回调需为 `Send + 'static`，
//!   必须以 Arc 共享所有权才能在回调线程内访问数据库。功能上仍是单连接、单 Mutex
//!   保护，Arc 仅为共享所有权的机械需要。
//! - `id`：固定为 `"local"`。
//! - `root_dirs`：已纳入扫描的根目录列表。
//! - `watchers`：每个根目录对应的 notify watcher，drop 即停止监听。
//! - `scan_count` / `scanning`：扫描进度跟踪。
//!
//! ## 子能力
//! - 15.1 目录递归扫描 + 扩展名过滤
//! - 15.2 元数据解析（lofty），缺失字段回退
//! - 15.3 SQLite 持久化索引（upsert / 查询）
//! - 15.4 文件夹监听增量更新（notify）
//! - 15.5 本地音乐作为内置源接入聚合搜索/播放列表/收藏夹
//! - 15.6/15.7 扫描进度

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use async_trait::async_trait;
use lofty::file::{AudioFile, TaggedFileExt};
use lofty::tag::Accessor;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use rusqlite::{params, Connection};
use uuid::Uuid;

use crate::error::{CoreError, Result};
use crate::models::{Leaderboard, Lyric, SearchResult, Song, SongOrigin};
use crate::sources::Source;

/// 本地音乐支持的扩展名（小写，不含点）。
const SUPPORTED_EXT: &[&str] = &["mp3", "flac", "m4a", "ape", "ogg", "wav", "aac"];

/// 扫描进度快照。
#[derive(Debug, Clone, Copy)]
pub struct ScanProgress {
    /// 当前已索引记录数（扫描期间累加）。
    pub current_count: usize,
    /// 是否正在扫描。
    pub scanning: bool,
}

/// lofty 解析得到的中间结构（已应用回退值）。
///
/// 按设计，`parse_file_metadata` 在 lofty 解析失败时也返回回退值，不返回 Err，
/// 因此这里直接以 `ParsedTrack`（而非 `Result<ParsedTrack>`）承载结果。
struct ParsedTrack {
    title: String,
    artists: Vec<String>,
    album: Option<String>,
    cover: Option<Vec<u8>>,
    duration_ms: Option<u64>,
}

/// 本地文件系统音乐源
pub struct LocalSource {
    /// SQLite 连接（Arc 包裹以便 watcher 回调线程共享）
    db: Arc<Mutex<Connection>>,
    /// 音源标识（固定 "local"）
    id: &'static str,
    /// 已纳入扫描的根目录
    root_dirs: Mutex<Vec<PathBuf>>,
    /// 每个根目录对应的 notify watcher
    watchers: Mutex<HashMap<PathBuf, RecommendedWatcher>>,
    /// 已索引记录数（扫描时累加）
    scan_count: AtomicUsize,
    /// 是否正在扫描
    scanning: AtomicBool,
}

impl LocalSource {
    /// 创建本地音乐源：打开/创建 SQLite 数据库并建表。
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(db_path.as_ref())
            .map_err(|e| CoreError::Source(e.to_string()))?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS local_tracks (
                id TEXT PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                artist TEXT NOT NULL,
                album TEXT,
                cover BLOB,
                duration_ms INTEGER,
                mtime INTEGER
            );
            CREATE INDEX IF NOT EXISTS idx_local_title ON local_tracks(title);
            CREATE INDEX IF NOT EXISTS idx_local_artist ON local_tracks(artist);
            CREATE INDEX IF NOT EXISTS idx_local_album ON local_tracks(album);",
        )
        .map_err(|e| CoreError::Source(e.to_string()))?;

        Ok(Self {
            db: Arc::new(Mutex::new(conn)),
            id: "local",
            root_dirs: Mutex::new(Vec::new()),
            watchers: Mutex::new(HashMap::new()),
            scan_count: AtomicUsize::new(0),
            scanning: AtomicBool::new(false),
        })
    }

    /// 添加扫描目录：加入 root_dirs，递归扫描所有 SUPPORTED_EXT 文件并 upsert 入库，
    /// 随后启动 notify watcher 监听该目录的后续变化。
    pub fn add_directory(&self, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref().to_path_buf();
        self.scanning.store(true, Ordering::SeqCst);
        self.scan_count.store(0, Ordering::SeqCst);
        // 任意错误路径都需复位 scanning 标志，避免 scan_progress 永远报告 scanning=true
        let result = self.add_directory_inner(&dir);
        self.scanning.store(false, Ordering::SeqCst);
        result
    }

    /// `add_directory` 的实际实现；调用方负责 `scanning` 标志的置位与复位。
    fn add_directory_inner(&self, dir: &Path) -> Result<()> {
        // 记录根目录（去重）
        {
            let mut roots = self
                .root_dirs
                .lock()
                .map_err(|e| CoreError::Source(e.to_string()))?;
            if !roots.iter().any(|p| p == &dir) {
                roots.push(dir.to_path_buf());
            }
        }

        // 递归扫描并入库
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let mut count = 0usize;
        let mut cb = |path: &Path| {
            if is_supported_ext(path) {
                let _ = upsert_track_path(&conn, path);
                count += 1;
                self.scan_count.store(count, Ordering::SeqCst);
            }
        };
        scan_dir_recursive(dir, &mut cb);
        drop(conn);

        // 启动 watcher（若失败则传播错误，已扫描的数据保留）
        self.start_watch(dir)?;

        Ok(())
    }

    /// 移除扫描目录：从 root_dirs 移除，停止该目录 watcher，
    /// 删除库中 path LIKE 该目录% 的记录。
    pub fn remove_directory(&self, dir: impl AsRef<Path>) -> Result<()> {
        let dir = dir.as_ref().to_path_buf();

        // 从 root_dirs 移除
        {
            let mut roots = self
                .root_dirs
                .lock()
                .map_err(|e| CoreError::Source(e.to_string()))?;
            roots.retain(|p| p != &dir);
        }

        // 停止 watcher：从 HashMap 取出后先释放锁，再 drop watcher。
        // RecommendedWatcher::drop 会 join notify 内部线程，若在锁内 drop
        // 而回调正在等待 db 锁，会长时间持有 watchers 锁阻塞其他操作。
        let _removed_watcher = {
            let mut watchers = self
                .watchers
                .lock()
                .map_err(|e| CoreError::Source(e.to_string()))?;
            watchers.remove(&dir)
        };

        // 删除库中该目录下所有记录。
        // 用「查询全部 + Rust 侧 Path::starts_with 过滤」而非 SQL LIKE：
        // 1) 跨平台一致（Windows 反斜杠与 SQL LIKE 默认转义符冲突，LIKE 会失效）；
        // 2) 与 rescan 的删除逻辑保持一致。
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let paths_to_delete: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT path FROM local_tracks")
                .map_err(|e| CoreError::Source(e.to_string()))?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| CoreError::Source(e.to_string()))?;
            let mut v = Vec::new();
            for r in rows {
                if let Ok(p) = r {
                    if Path::new(&p).starts_with(&dir) {
                        v.push(p);
                    }
                }
            }
            v
        };
        for p in paths_to_delete {
            let _ = conn.execute(
                "DELETE FROM local_tracks WHERE path = ?1",
                params![&p],
            );
        }
        Ok(())
    }

    /// 重新扫描所有 root_dirs（增量：重新解析所有现存文件并 upsert，
    /// 最后删除磁盘上已不存在的记录）。
    pub fn rescan(&self) -> Result<()> {
        self.scanning.store(true, Ordering::SeqCst);
        self.scan_count.store(0, Ordering::SeqCst);
        // 任意错误路径都需复位 scanning 标志（与 add_directory 同源问题），
        // 否则 scan_progress 会永远报告 scanning=true。
        let result = self.rescan_inner();
        self.scanning.store(false, Ordering::SeqCst);
        result
    }

    /// `rescan` 的实际实现；调用方负责 `scanning` 标志的置位与复位。
    fn rescan_inner(&self) -> Result<()> {
        let roots: Vec<PathBuf> = {
            let roots = self
                .root_dirs
                .lock()
                .map_err(|e| CoreError::Source(e.to_string()))?;
            roots.clone()
        };

        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let mut count = 0usize;
        for root in &roots {
            let mut cb = |path: &Path| {
                if is_supported_ext(path) {
                    let _ = upsert_track_path(&conn, path);
                    count += 1;
                    self.scan_count.store(count, Ordering::SeqCst);
                }
            };
            scan_dir_recursive(root, &mut cb);
        }

        // 删除库中磁盘上已不存在的记录
        let all_paths: Vec<String> = {
            let mut stmt = conn
                .prepare("SELECT path FROM local_tracks")
                .map_err(|e| CoreError::Source(e.to_string()))?;
            let rows = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| CoreError::Source(e.to_string()))?;
            let mut v = Vec::new();
            for r in rows {
                if let Ok(p) = r {
                    v.push(p);
                }
            }
            v
        };
        for p in all_paths {
            if !Path::new(&p).exists() {
                let _ = conn.execute(
                    "DELETE FROM local_tracks WHERE path = ?1",
                    params![&p],
                );
            }
        }
        drop(conn);

        Ok(())
    }

    /// 返回当前库中所有歌曲（供 UI 浏览）。
    pub fn list_all(&self) -> Result<Vec<Song>> {
        self.query_all()
    }

    /// 返回扫描进度快照。
    pub fn scan_progress(&self) -> ScanProgress {
        ScanProgress {
            current_count: self.scan_count.load(Ordering::SeqCst),
            scanning: self.scanning.load(Ordering::SeqCst),
        }
    }

    /// 启动 notify watcher 监听目录变化。
    ///
    /// 回调在 notify 的内部线程中执行（sync 闭包），通过 `Arc::clone(&self.db)`
    /// 拿到数据库连接并加锁。事件处理采用统一策略：对每个受影响路径，
    /// 若文件仍存在则 upsert（覆盖 Created/Modified/Renamed-to），若已不存在则删除
    /// （覆盖 Removed/Renamed-from）。
    fn start_watch(&self, dir: &Path) -> Result<()> {
        let db_clone = Arc::clone(&self.db);
        let callback = move |res: std::result::Result<Event, notify::Error>| {
            let event = match res {
                Ok(e) => e,
                Err(_) => return,
            };
            // 仅关心 Create/Modify/Remove；Access/Other 忽略
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {}
                _ => return,
            }
            for path in &event.paths {
                if !is_supported_ext(path) {
                    continue;
                }
                if path.exists() {
                    if let Ok(conn) = db_clone.lock() {
                        let _ = upsert_track_path(&conn, path);
                    }
                } else if let Ok(conn) = db_clone.lock() {
                    let _ = delete_track_path(&conn, path);
                }
            }
        };

        let mut watcher = notify::recommended_watcher(callback)
            .map_err(|e| CoreError::Source(e.to_string()))?;
        watcher
            .watch(dir, RecursiveMode::Recursive)
            .map_err(|e| CoreError::Source(e.to_string()))?;

        // 若已存在旧 watcher，insert 会返回旧的；先释放锁再 drop 旧的，
        // 避免 drop（join notify 线程）时长时间持有 watchers 锁。
        let _old_watcher = {
            let mut watchers = self
                .watchers
                .lock()
                .map_err(|e| CoreError::Source(e.to_string()))?;
            watchers.insert(dir.to_path_buf(), watcher)
        };
        Ok(())
    }

    /// 查询库中所有歌曲（DB 访问 helper）。
    fn query_all(&self) -> Result<Vec<Song>> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        query_all(&conn)
    }

    /// 关键字搜索（title/artist/album 任一 LIKE %keyword%）（DB 访问 helper）。
    fn search_db(&self, keyword: &str) -> Result<Vec<Song>> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let like = format!("%{}%", keyword);
        let mut stmt = conn
            .prepare(
                "SELECT id,path,title,artist,album,cover,duration_ms FROM local_tracks \
                 WHERE title LIKE ?1 OR artist LIKE ?1 OR album LIKE ?1 \
                 ORDER BY title",
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let songs = stmt
            .query_map(params![&like], row_to_song)
            .map_err(|e| CoreError::Source(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();
        Ok(songs)
    }
}

#[async_trait]
impl Source for LocalSource {
    fn id(&self) -> &str {
        self.id
    }

    fn name(&self) -> &str {
        "本地音乐"
    }

    async fn search(&self, keyword: &str, page: u32, page_size: u32) -> Result<SearchResult> {
        let all = self.search_db(keyword)?;
        let total = all.len() as u64;
        let page = page.max(1);
        let page_size = (page_size as usize).max(1);
        let offset = ((page - 1) as usize) * page_size;
        let songs: Vec<Song> = all.into_iter().skip(offset).take(page_size).collect();
        Ok(SearchResult {
            keyword: keyword.to_string(),
            songs,
            albums: Vec::new(),
            artists: Vec::new(),
            total,
            page,
            page_size: page_size as u32,
        })
    }

    async fn get_metadata(&self, song_id: &str) -> Result<Song> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        match conn.query_row(
            "SELECT id,path,title,artist,album,cover,duration_ms FROM local_tracks WHERE id = ?1",
            params![song_id],
            row_to_song,
        ) {
            Ok(s) => Ok(s),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(CoreError::NotFound(format!("本地歌曲不存在: {song_id}")))
            }
            Err(e) => Err(CoreError::Source(e.to_string())),
        }
    }

    async fn get_play_url(&self, song_id: &str) -> Result<String> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        match conn.query_row(
            "SELECT path FROM local_tracks WHERE id = ?1",
            params![song_id],
            |row| row.get::<_, String>(0),
        ) {
            Ok(p) => Ok(p),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                Err(CoreError::NotFound(format!("本地歌曲不存在: {song_id}")))
            }
            Err(e) => Err(CoreError::Source(e.to_string())),
        }
    }

    async fn get_lyric(&self, _song_id: &str) -> Result<Lyric> {
        // 本地音乐暂无歌词
        Err(CoreError::NotFound("本地音乐暂无歌词".into()))
    }

    async fn get_leaderboards(&self) -> Result<Vec<Leaderboard>> {
        // 本地音源无排行榜
        Ok(Vec::new())
    }
}

// ---------------------------------------------------------------------------
// 自由函数 helper
// ---------------------------------------------------------------------------

/// 递归扫描目录，对每个文件条目调用回调 `cb`。
///
/// walkdir 风格手写递归（不引入 walkdir 依赖），使用 `std::fs::read_dir`。
/// 读取失败的子目录会被跳过（不影响其余条目）。
///
/// 回调以 `&mut dyn FnMut` 传入，确保递归各层类型一致，
/// 避免 `impl FnMut` 泛型在递归调用中无限嵌套实例化。
fn scan_dir_recursive(dir: &Path, cb: &mut dyn FnMut(&Path)) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let ft = match entry.file_type() {
            Ok(t) => t,
            Err(_) => continue,
        };
        if ft.is_dir() {
            scan_dir_recursive(&path, cb);
        } else if ft.is_file() {
            cb(&path);
        }
    }
}

/// 判断路径扩展名是否为支持的音频格式（大小写不敏感）。
fn is_supported_ext(path: &Path) -> bool {
    let ext = match path.extension().and_then(|e| e.to_str()) {
        Some(s) => s,
        None => return false,
    };
    SUPPORTED_EXT
        .iter()
        .any(|&supported| supported.eq_ignore_ascii_case(ext))
}

/// 按 ";" 或 "/" 分割多艺术家字符串为列表。
fn split_artists(s: &str) -> Vec<String> {
    s.split(|c| c == ';' || c == '/')
        .map(|p| p.trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

/// 取文件 mtime（秒，UNIX 时间戳）。失败或早于 epoch 返回 None。
fn file_mtime_secs(path: &Path) -> Option<u64> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    modified
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

/// 用 lofty 解析音频文件元数据。
///
/// 解析失败（lofty 报错）也使用回退值：title=文件名 stem、artists=["Unknown Artist"]，
/// 不返回 Err，保证扫描健壮。字段缺失时同样回退。
fn parse_file_metadata(path: &Path) -> ParsedTrack {
    let file_stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let tagged_file = match lofty::read_from_path(path) {
        Ok(f) => f,
        Err(_) => {
            return ParsedTrack {
                title: file_stem,
                artists: vec!["Unknown Artist".to_string()],
                album: None,
                cover: None,
                duration_ms: None,
            };
        }
    };

    let tag_opt = tagged_file.primary_tag();

    let title = tag_opt
        .and_then(|t| t.title())
        .map(|c| c.to_string())
        .unwrap_or_else(|| file_stem.clone());

    let artists = tag_opt
        .and_then(|t| t.artist())
        .map(|c| split_artists(&c))
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| vec!["Unknown Artist".to_string()]);

    let album = tag_opt.and_then(|t| t.album()).map(|c| c.to_string());

    let cover = tag_opt
        .and_then(|t| t.pictures().first())
        .map(|p| p.data().to_vec());

    let duration = tagged_file.properties().duration();
    let duration_ms = if duration.is_zero() {
        None
    } else {
        Some(duration.as_millis() as u64)
    };

    ParsedTrack {
        title,
        artists,
        album,
        cover,
        duration_ms,
    }
}

/// 解析文件元数据并 upsert 入库。
///
/// 使用 `INSERT ... ON CONFLICT(path) DO UPDATE SET ...`：路径已存在时
/// 保留原 id，仅更新其余字段。
fn upsert_track_path(conn: &Connection, path: &Path) -> Result<()> {
    let parsed = parse_file_metadata(path);
    let mtime = file_mtime_secs(path).unwrap_or(0) as i64;
    let artist_joined = parsed.artists.join(" / ");
    let path_str = path.to_string_lossy().to_string();

    conn.execute(
        "INSERT INTO local_tracks(id,path,title,artist,album,cover,duration_ms,mtime) \
         VALUES(?1,?2,?3,?4,?5,?6,?7,?8) \
         ON CONFLICT(path) DO UPDATE SET \
         title=excluded.title, artist=excluded.artist, album=excluded.album, \
         cover=excluded.cover, duration_ms=excluded.duration_ms, mtime=excluded.mtime",
        params![
            Uuid::new_v4().to_string(),
            path_str,
            parsed.title,
            artist_joined,
            parsed.album,
            parsed.cover,
            parsed.duration_ms,
            mtime,
        ],
    )
    .map_err(|e| CoreError::Source(e.to_string()))?;
    Ok(())
}

/// 删除库中指定路径的记录。
fn delete_track_path(conn: &Connection, path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy().to_string();
    conn.execute(
        "DELETE FROM local_tracks WHERE path = ?1",
        params![path_str],
    )
    .map_err(|e| CoreError::Source(e.to_string()))?;
    Ok(())
}

/// 查询库中所有歌曲（按 title 排序）。
fn query_all(conn: &Connection) -> Result<Vec<Song>> {
    let mut stmt = conn
        .prepare("SELECT id,path,title,artist,album,cover,duration_ms FROM local_tracks ORDER BY title")
        .map_err(|e| CoreError::Source(e.to_string()))?;
    let songs = stmt
        .query_map([], row_to_song)
        .map_err(|e| CoreError::Source(e.to_string()))?
        .filter_map(|r| r.ok())
        .collect();
    Ok(songs)
}

/// rusqlite 行 → Song 转换。
///
/// - `source_id` 固定为 `"local"`。
/// - `cover_url` 设为 None（本地封面走 BLOB，不在 URL）。
/// - `artist` 以 `" / "` 连接，读取时 split 回列表。
/// - `local_path` 设为该路径，`origin` 为 `SongOrigin::Local { path }`。
fn row_to_song(row: &rusqlite::Row) -> rusqlite::Result<Song> {
    let id: String = row.get(0)?;
    let path: String = row.get(1)?;
    let title: String = row.get(2)?;
    let artist: String = row.get(3)?;
    let album: Option<String> = row.get(4)?;
    let _cover: Option<Vec<u8>> = row.get(5)?;
    let duration_ms: Option<i64> = row.get(6)?;

    let artists: Vec<String> = if artist.is_empty() {
        vec!["Unknown Artist".to_string()]
    } else {
        artist.split(" / ").map(|s| s.to_string()).collect()
    };

    Ok(Song {
        id,
        source_id: "local".to_string(),
        title,
        artists,
        album,
        cover_url: None,
        duration_ms: duration_ms.map(|v| v as u64),
        lyric_url: None,
        play_url: None,
        local_path: Some(PathBuf::from(&path)),
        origin: SongOrigin::Local {
            path: PathBuf::from(path),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// 构造一个临时 SQLite + 临时音乐目录，返回 (source, 临时根目录, 音乐子目录)。
    /// `_tmp` 需保持存活直至测试结束，否则目录会被回收。
    fn setup() -> (LocalSource, tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let db_path = tmp.path().join("test.db");
        let source = LocalSource::new(&db_path).unwrap();
        let music_dir = tmp.path().join("music");
        fs::create_dir_all(&music_dir).unwrap();
        (source, tmp, music_dir)
    }

    #[test]
    fn add_directory_indexes_empty_mp3_with_filename_title() {
        let (source, _tmp, music_dir) = setup();
        // 空 mp3：lofty 解析会失败，触发回退（title=文件名 stem, artists=["Unknown Artist"]）
        fs::write(music_dir.join("My Song.mp3"), b"").unwrap();

        source.add_directory(&music_dir).unwrap();

        let songs = source.list_all().unwrap();
        assert_eq!(songs.len(), 1, "应索引到 1 首歌曲");
        assert_eq!(songs[0].title, "My Song");
        assert_eq!(songs[0].artists, vec!["Unknown Artist".to_string()]);
        assert_eq!(songs[0].source_id, "local");
        assert!(songs[0].local_path.is_some());
        match &songs[0].origin {
            SongOrigin::Local { path } => {
                assert!(path.ends_with("My Song.mp3"));
            }
            _ => panic!("origin 应为 Local"),
        }
        assert_eq!(source.id(), "local");
        assert_eq!(source.name(), "本地音乐");
    }

    #[test]
    fn scan_progress_during_and_after_scan() {
        let (source, _tmp, music_dir) = setup();
        fs::write(music_dir.join("A.mp3"), b"").unwrap();
        fs::write(music_dir.join("B.flac"), b"").unwrap();

        // 扫描前
        let p = source.scan_progress();
        assert!(!p.scanning);
        assert_eq!(p.current_count, 0);

        source.add_directory(&music_dir).unwrap();

        // 扫描后
        let p = source.scan_progress();
        assert!(!p.scanning, "扫描结束后 scanning 应为 false");
        assert_eq!(p.current_count, 2);
    }

    #[tokio::test]
    async fn search_finds_indexed_track() {
        let (source, _tmp, music_dir) = setup();
        fs::write(music_dir.join("Hello World.mp3"), b"").unwrap();
        source.add_directory(&music_dir).unwrap();

        let result = source.search("Hello", 1, 10).await.unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.songs.len(), 1);
        assert_eq!(result.songs[0].title, "Hello World");
        assert_eq!(result.page, 1);
        assert_eq!(result.page_size, 10);
        assert!(result.albums.is_empty());
        assert!(result.artists.is_empty());
    }

    #[tokio::test]
    async fn search_pagination_works() {
        let (source, _tmp, music_dir) = setup();
        for i in 0..5 {
            fs::write(music_dir.join(format!("Track{i}.mp3")), b"").unwrap();
        }
        source.add_directory(&music_dir).unwrap();

        // 每页 2 条，取第 2 页 → 应有 2 条
        let result = source.search("Track", 2, 2).await.unwrap();
        assert_eq!(result.total, 5);
        assert_eq!(result.songs.len(), 2);
        assert_eq!(result.page, 2);
        assert_eq!(result.page_size, 2);
    }

    #[test]
    fn remove_directory_clears_index() {
        let (source, _tmp, music_dir) = setup();
        fs::write(music_dir.join("Track.mp3"), b"").unwrap();
        source.add_directory(&music_dir).unwrap();
        assert_eq!(source.list_all().unwrap().len(), 1);

        source.remove_directory(&music_dir).unwrap();
        assert_eq!(source.list_all().unwrap().len(), 0, "移除目录后索引应清空");
    }

    #[test]
    fn rescan_removes_deleted_records() {
        let (source, _tmp, music_dir) = setup();
        let file = music_dir.join("Gone.mp3");
        fs::write(&file, b"").unwrap();
        source.add_directory(&music_dir).unwrap();
        assert_eq!(source.list_all().unwrap().len(), 1);

        // 删除文件后 rescan，记录应被清除
        fs::remove_file(&file).unwrap();
        source.rescan().unwrap();
        assert_eq!(
            source.list_all().unwrap().len(),
            0,
            "rescan 应清除已删除文件的记录"
        );
    }

    #[tokio::test]
    async fn get_metadata_returns_song_and_not_found() {
        let (source, _tmp, music_dir) = setup();
        fs::write(music_dir.join("Meta.mp3"), b"").unwrap();
        source.add_directory(&music_dir).unwrap();

        let songs = source.list_all().unwrap();
        let id = songs[0].id.clone();
        let song = source.get_metadata(&id).await.unwrap();
        assert_eq!(song.title, "Meta");
        assert_eq!(song.source_id, "local");

        // 不存在的 id
        let err = source.get_metadata("nonexistent").await;
        assert!(
            matches!(err, Err(CoreError::NotFound(_))),
            "不存在的 id 应返回 NotFound"
        );
    }

    #[tokio::test]
    async fn get_play_url_returns_local_path() {
        let (source, _tmp, music_dir) = setup();
        fs::write(music_dir.join("Play.mp3"), b"").unwrap();
        source.add_directory(&music_dir).unwrap();

        let songs = source.list_all().unwrap();
        let id = songs[0].id.clone();
        let url = source.get_play_url(&id).await.unwrap();
        assert!(url.ends_with("Play.mp3"));

        // 不存在的 id
        let err = source.get_play_url("nonexistent").await;
        assert!(matches!(err, Err(CoreError::NotFound(_))));
    }

    #[tokio::test]
    async fn get_lyric_returns_not_found() {
        let (source, _tmp, _music_dir) = setup();
        let err = source.get_lyric("any").await;
        assert!(
            matches!(err, Err(CoreError::NotFound(_))),
            "本地音乐暂无歌词应返回 NotFound"
        );
    }

    #[tokio::test]
    async fn get_leaderboards_returns_empty() {
        let (source, _tmp, _music_dir) = setup();
        let lbs = source.get_leaderboards().await.unwrap();
        assert!(lbs.is_empty(), "本地音源无排行榜");
    }

    #[test]
    fn split_artists_handles_separators() {
        assert_eq!(split_artists("A; B/C"), vec!["A", "B", "C"]);
        assert_eq!(split_artists("Solo"), vec!["Solo"]);
        assert_eq!(split_artists(""), Vec::<String>::new());
        assert_eq!(split_artists(" ; / "), Vec::<String>::new());
    }

    #[test]
    fn is_supported_ext_case_insensitive() {
        assert!(is_supported_ext(Path::new("/x/a.MP3")));
        assert!(is_supported_ext(Path::new("/x/a.flac")));
        assert!(is_supported_ext(Path::new("/x/a.Wav")));
        assert!(!is_supported_ext(Path::new("/x/a.txt")));
        assert!(!is_supported_ext(Path::new("/x/noext")));
    }
}
