//! 音乐库持久化层（SQLite）。
//!
//! 统一管理播放列表、收藏夹、播放状态的持久化存储。
//! 通过 `Mutex<rusqlite::Connection>` 提供线程安全的访问。
//! 播放列表/收藏夹仅存储歌曲的字符串 id 引用，实际 Song 数据
//! 由音源/本地源提供。

use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::error::{CoreError, Result};
use crate::models::*;
use chrono::Utc;
use uuid::Uuid;

/// 音乐库持久化层，封装 SQLite 连接，提供播放列表/收藏/播放状态的 CRUD。
pub struct Library {
    db: Mutex<Connection>,
}

impl Library {
    /// 打开指定路径的 SQLite 数据库并初始化表结构。
    pub fn new(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let conn = Connection::open(path).map_err(|e| CoreError::Source(e.to_string()))?;
        let lib = Self {
            db: Mutex::new(conn),
        };
        lib.init_schema()?;
        Ok(lib)
    }

    /// 使用内存数据库（主要用于测试）。
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| CoreError::Source(e.to_string()))?;
        let lib = Self {
            db: Mutex::new(conn),
        };
        lib.init_schema()?;
        Ok(lib)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS playlists (
              id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS playlist_songs (
              playlist_id TEXT NOT NULL, song_id TEXT NOT NULL, position INTEGER NOT NULL,
              PRIMARY KEY(playlist_id, position), FOREIGN KEY(playlist_id) REFERENCES playlists(id)
            );
            CREATE TABLE IF NOT EXISTS favorite_groups (
              id TEXT PRIMARY KEY, name TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS favorite_songs (
              group_id TEXT NOT NULL, song_id TEXT NOT NULL, position INTEGER NOT NULL,
              PRIMARY KEY(group_id, position), FOREIGN KEY(group_id) REFERENCES favorite_groups(id)
            );
            CREATE TABLE IF NOT EXISTS play_state (
              id INTEGER PRIMARY KEY CHECK(id=1),
              current_song_id TEXT, playlist_id TEXT, position INTEGER,
              position_ms INTEGER, duration_ms INTEGER, is_playing INTEGER, volume REAL, mode TEXT
            );
            "#,
        )
        .map_err(|e| CoreError::Source(e.to_string()))?;
        Ok(())
    }

    // ============== 播放列表 11.1 ==============

    /// 创建新播放列表，返回完整 Playlist（id/created_at 自动生成）。
    pub fn create_playlist(&self, name: &str) -> Result<Playlist> {
        let id = Uuid::new_v4().to_string();
        let created_at = Utc::now();
        let created_at_str = created_at.to_rfc3339();
        {
            let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
            conn.execute(
                "INSERT INTO playlists (id, name, created_at) VALUES (?1, ?2, ?3)",
                params![id, name, created_at_str],
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        }
        Ok(Playlist {
            id,
            name: name.to_string(),
            song_ids: Vec::new(),
            created_at,
        })
    }

    /// 列出所有播放列表（song_ids 按 position 排序）。
    pub fn list_playlists(&self) -> Result<Vec<Playlist>> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, name, created_at FROM playlists ORDER BY created_at ASC")
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let created_at_str: String = row.get(2)?;
                Ok((id, name, created_at_str))
            })
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            let (id, name, created_at_str) = row.map_err(|e| CoreError::Source(e.to_string()))?;
            let song_ids = fetch_playlist_song_ids(&conn, &id)?;
            let created_at = parse_dt(&created_at_str)?;
            result.push(Playlist {
                id,
                name,
                song_ids,
                created_at,
            });
        }
        Ok(result)
    }

    /// 获取单个播放列表（含按 position 排序的 song_ids）。
    pub fn get_playlist(&self, id: &str) -> Result<Playlist> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let (id, name, created_at_str): (String, String, String) = conn
            .query_row(
                "SELECT id, name, created_at FROM playlists WHERE id=?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let song_ids = fetch_playlist_song_ids(&conn, &id)?;
        let created_at = parse_dt(&created_at_str)?;
        Ok(Playlist {
            id,
            name,
            song_ids,
            created_at,
        })
    }

    /// 向播放列表追加一首歌曲（position 取当前最大+1）。
    pub fn add_to_playlist(&self, playlist_id: &str, song_id: &str) -> Result<()> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let max_pos: Option<i64> = conn
            .query_row(
                "SELECT MAX(position) FROM playlist_songs WHERE playlist_id=?1",
                params![playlist_id],
                |row| row.get(0),
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let next = max_pos.unwrap_or(-1) + 1;
        conn.execute(
            "INSERT INTO playlist_songs (playlist_id, song_id, position) VALUES (?1, ?2, ?3)",
            params![playlist_id, song_id, next],
        )
        .map_err(|e| CoreError::Source(e.to_string()))?;
        Ok(())
    }

    /// 从播放列表删除指定位置的歌曲并重排后续 position。
    pub fn remove_from_playlist(&self, playlist_id: &str, position: usize) -> Result<()> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let deleted = conn
            .execute(
                "DELETE FROM playlist_songs WHERE playlist_id=?1 AND position=?2",
                params![playlist_id, position as i64],
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        if deleted == 0 {
            return Err(CoreError::NotFound(format!(
                "playlist position not found: {}:{}",
                playlist_id, position
            )));
        }
        conn.execute(
            "UPDATE playlist_songs SET position=position-1 WHERE playlist_id=?1 AND position>?2",
            params![playlist_id, position as i64],
        )
        .map_err(|e| CoreError::Source(e.to_string()))?;
        Ok(())
    }

    /// 将播放列表中 from 位置的歌曲移动到 to 位置，并重排为连续 position。
    pub fn reorder_playlist(&self, playlist_id: &str, from: usize, to: usize) -> Result<()> {
        if from == to {
            return Ok(());
        }
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        // 读出当前歌曲顺序并在内存中重排，避免主键冲突
        let mut songs = fetch_playlist_song_ids(&conn, playlist_id)?;
        if from >= songs.len() {
            return Err(CoreError::NotFound(format!(
                "reorder from out of range: len={}, from={}",
                songs.len(),
                from
            )));
        }
        if to >= songs.len() {
            return Err(CoreError::NotFound(format!(
                "reorder to out of range: len={}, to={}",
                songs.len(),
                to
            )));
        }
        let song = songs.remove(from);
        songs.insert(to, song);
        conn.execute(
            "DELETE FROM playlist_songs WHERE playlist_id=?1",
            params![playlist_id],
        )
        .map_err(|e| CoreError::Source(e.to_string()))?;
        for (i, song_id) in songs.iter().enumerate() {
            conn.execute(
                "INSERT INTO playlist_songs (playlist_id, song_id, position) VALUES (?1, ?2, ?3)",
                params![playlist_id, song_id, i as i64],
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        }
        Ok(())
    }

    /// 清空播放列表中所有歌曲（保留播放列表本身）。
    pub fn clear_playlist(&self, playlist_id: &str) -> Result<()> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        conn.execute(
            "DELETE FROM playlist_songs WHERE playlist_id=?1",
            params![playlist_id],
        )
        .map_err(|e| CoreError::Source(e.to_string()))?;
        Ok(())
    }

    /// 删除播放列表（包括其歌曲引用）。
    pub fn delete_playlist(&self, id: &str) -> Result<()> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        conn.execute(
            "DELETE FROM playlist_songs WHERE playlist_id=?1",
            params![id],
        )
        .map_err(|e| CoreError::Source(e.to_string()))?;
        conn.execute("DELETE FROM playlists WHERE id=?1", params![id])
            .map_err(|e| CoreError::Source(e.to_string()))?;
        Ok(())
    }

    // ============== 收藏夹 12.1 ==============

    /// 创建新收藏分组。
    pub fn create_favorite_group(&self, name: &str) -> Result<FavoriteGroup> {
        let id = Uuid::new_v4().to_string();
        {
            let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
            conn.execute(
                "INSERT INTO favorite_groups (id, name) VALUES (?1, ?2)",
                params![id, name],
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        }
        Ok(FavoriteGroup {
            id,
            name: name.to_string(),
            song_ids: Vec::new(),
        })
    }

    /// 列出所有收藏分组（song_ids 按 position 排序）。
    pub fn list_favorite_groups(&self) -> Result<Vec<FavoriteGroup>> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, name FROM favorite_groups ORDER BY id ASC")
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                Ok((id, name))
            })
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let mut result = Vec::new();
        for row in rows {
            let (id, name) = row.map_err(|e| CoreError::Source(e.to_string()))?;
            let song_ids = fetch_favorite_song_ids(&conn, &id)?;
            result.push(FavoriteGroup {
                id,
                name,
                song_ids,
            });
        }
        Ok(result)
    }

    /// 向收藏分组追加一首歌曲。
    pub fn add_to_favorites(&self, group_id: &str, song_id: &str) -> Result<()> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let max_pos: Option<i64> = conn
            .query_row(
                "SELECT MAX(position) FROM favorite_songs WHERE group_id=?1",
                params![group_id],
                |row| row.get(0),
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let next = max_pos.unwrap_or(-1) + 1;
        conn.execute(
            "INSERT INTO favorite_songs (group_id, song_id, position) VALUES (?1, ?2, ?3)",
            params![group_id, song_id, next],
        )
        .map_err(|e| CoreError::Source(e.to_string()))?;
        Ok(())
    }

    /// 从收藏分组删除指定位置的歌曲并重排。
    pub fn remove_from_favorites(&self, group_id: &str, position: usize) -> Result<()> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let deleted = conn
            .execute(
                "DELETE FROM favorite_songs WHERE group_id=?1 AND position=?2",
                params![group_id, position as i64],
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        if deleted == 0 {
            return Err(CoreError::NotFound(format!(
                "favorite position not found: {}:{}",
                group_id, position
            )));
        }
        conn.execute(
            "UPDATE favorite_songs SET position=position-1 WHERE group_id=?1 AND position>?2",
            params![group_id, position as i64],
        )
        .map_err(|e| CoreError::Source(e.to_string()))?;
        Ok(())
    }

    /// 删除收藏分组（包括其歌曲引用）。
    pub fn delete_favorite_group(&self, id: &str) -> Result<()> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        conn.execute(
            "DELETE FROM favorite_songs WHERE group_id=?1",
            params![id],
        )
        .map_err(|e| CoreError::Source(e.to_string()))?;
        conn.execute("DELETE FROM favorite_groups WHERE id=?1", params![id])
            .map_err(|e| CoreError::Source(e.to_string()))?;
        Ok(())
    }

    /// 导出收藏分组为 JSON：`{group: FavoriteGroup, songs: [song_id...]}`。
    pub fn export_favorites(&self, group_id: &str) -> Result<serde_json::Value> {
        let group = self.get_favorite_group(group_id)?;
        Ok(serde_json::json!({
            "group": group,
            "songs": group.song_ids,
        }))
    }

    /// 将一组歌曲导入到收藏分组（追加到末尾）。
    pub fn import_favorites(&self, group_id: &str, songs: &[String]) -> Result<()> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let max_pos: Option<i64> = conn
            .query_row(
                "SELECT MAX(position) FROM favorite_songs WHERE group_id=?1",
                params![group_id],
                |row| row.get(0),
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let mut pos = max_pos.unwrap_or(-1) + 1;
        for song_id in songs {
            conn.execute(
                "INSERT INTO favorite_songs (group_id, song_id, position) VALUES (?1, ?2, ?3)",
                params![group_id, song_id, pos],
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
            pos += 1;
        }
        Ok(())
    }

    fn get_favorite_group(&self, group_id: &str) -> Result<FavoriteGroup> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let (id, name): (String, String) = conn
            .query_row(
                "SELECT id, name FROM favorite_groups WHERE id=?1",
                params![group_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let song_ids = fetch_favorite_song_ids(&conn, &id)?;
        Ok(FavoriteGroup {
            id,
            name,
            song_ids,
        })
    }

    // ============== 播放状态 10.1 ==============

    /// 保存播放状态（INSERT OR REPLACE 覆盖单行 id=1）。
    pub fn save_play_state(&self, state: &PlayState) -> Result<()> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let mode_str = mode_to_str(&state.mode);
        conn.execute(
            "INSERT OR REPLACE INTO play_state \
             (id, current_song_id, playlist_id, position, position_ms, duration_ms, is_playing, volume, mode) \
             VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                state.current_song_id,
                state.playlist_id,
                state.index.map(|i| i as i64),
                state.position_ms as i64,
                state.duration_ms as i64,
                state.is_playing as i64,
                state.volume as f64,
                mode_str,
            ],
        )
        .map_err(|e| CoreError::Source(e.to_string()))?;
        Ok(())
    }

    /// 读取播放状态；无记录返回 None。mode 字段反序列化为 PlayMode。
    pub fn load_play_state(&self) -> Result<Option<PlayState>> {
        let conn = self.db.lock().map_err(|e| CoreError::Source(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT current_song_id, playlist_id, position, position_ms, duration_ms, \
                 is_playing, volume, mode FROM play_state WHERE id=1",
            )
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let mut rows = stmt
            .query([])
            .map_err(|e| CoreError::Source(e.to_string()))?;
        let row = match rows
            .next()
            .map_err(|e| CoreError::Source(e.to_string()))?
        {
            None => return Ok(None),
            Some(r) => r,
        };
        let current_song_id: Option<String> =
            row.get(0).map_err(|e| CoreError::Source(e.to_string()))?;
        let playlist_id: Option<String> =
            row.get(1).map_err(|e| CoreError::Source(e.to_string()))?;
        let position: Option<i64> = row.get(2).map_err(|e| CoreError::Source(e.to_string()))?;
        let position_ms: i64 = row.get(3).map_err(|e| CoreError::Source(e.to_string()))?;
        let duration_ms: i64 = row.get(4).map_err(|e| CoreError::Source(e.to_string()))?;
        let is_playing: i64 = row.get(5).map_err(|e| CoreError::Source(e.to_string()))?;
        let volume: f64 = row.get(6).map_err(|e| CoreError::Source(e.to_string()))?;
        let mode: String = row.get(7).map_err(|e| CoreError::Source(e.to_string()))?;
        Ok(Some(PlayState {
            current_song_id,
            playlist_id,
            index: position.map(|p| p as usize),
            position_ms: position_ms as u64,
            duration_ms: duration_ms as u64,
            is_playing: is_playing != 0,
            volume: volume as f32,
            mode: str_to_mode(&mode)?,
        }))
    }
}

// ============== 辅助函数 ==============

fn fetch_playlist_song_ids(conn: &Connection, playlist_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT song_id FROM playlist_songs WHERE playlist_id=?1 ORDER BY position ASC")
        .map_err(|e| CoreError::Source(e.to_string()))?;
    let songs = stmt
        .query_map(params![playlist_id], |row| row.get::<_, String>(0))
        .map_err(|e| CoreError::Source(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| CoreError::Source(e.to_string()))?;
    Ok(songs)
}

fn fetch_favorite_song_ids(conn: &Connection, group_id: &str) -> Result<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT song_id FROM favorite_songs WHERE group_id=?1 ORDER BY position ASC")
        .map_err(|e| CoreError::Source(e.to_string()))?;
    let songs = stmt
        .query_map(params![group_id], |row| row.get::<_, String>(0))
        .map_err(|e| CoreError::Source(e.to_string()))?
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| CoreError::Source(e.to_string()))?;
    Ok(songs)
}

fn parse_dt(s: &str) -> Result<chrono::DateTime<Utc>> {
    chrono::DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| CoreError::Source(e.to_string()))
}

fn mode_to_str(mode: &PlayMode) -> &'static str {
    match mode {
        PlayMode::Sequential => "sequential",
        PlayMode::SingleLoop => "single_loop",
        PlayMode::Random => "random",
    }
}

fn str_to_mode(s: &str) -> Result<PlayMode> {
    match s {
        "sequential" => Ok(PlayMode::Sequential),
        "single_loop" => Ok(PlayMode::SingleLoop),
        "random" => Ok(PlayMode::Random),
        other => Err(CoreError::Source(format!("未知 PlayMode: {}", other))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建基于 tempfile 的 Library，并保持临时文件存活以避免
    /// SQLite 通过 access(path) 检查写入权限时因路径被删除而报 readonly。
    fn temp_library() -> (Library, tempfile::NamedTempFile) {
        let tmp = tempfile::NamedTempFile::new().expect("create tempfile");
        let lib = Library::new(tmp.path()).expect("open library");
        (lib, tmp)
    }

    #[test]
    fn test_playlist_add_and_list_order() {
        let (lib, _tmp) = temp_library();
        let pl = lib.create_playlist("我的列表").unwrap();
        lib.add_to_playlist(&pl.id, "song-a").unwrap();
        lib.add_to_playlist(&pl.id, "song-b").unwrap();
        lib.add_to_playlist(&pl.id, "song-c").unwrap();
        let got = lib.get_playlist(&pl.id).unwrap();
        assert_eq!(got.song_ids, vec!["song-a", "song-b", "song-c"]);
        let list = lib.list_playlists().unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].song_ids, vec!["song-a", "song-b", "song-c"]);
    }

    #[test]
    fn test_playlist_remove_reorders() {
        let (lib, _tmp) = temp_library();
        let pl = lib.create_playlist("pl").unwrap();
        for s in ["s1", "s2", "s3", "s4"] {
            lib.add_to_playlist(&pl.id, s).unwrap();
        }
        lib.remove_from_playlist(&pl.id, 1).unwrap();
        let got = lib.get_playlist(&pl.id).unwrap();
        assert_eq!(got.song_ids, vec!["s1", "s3", "s4"]);
    }

    #[test]
    fn test_playlist_reorder() {
        let (lib, _tmp) = temp_library();
        let pl = lib.create_playlist("pl").unwrap();
        for s in ["s1", "s2", "s3", "s4"] {
            lib.add_to_playlist(&pl.id, s).unwrap();
        }
        // 把第 0 项移到第 2 位：[s1,s2,s3,s4] -> [s2,s3,s1,s4]
        lib.reorder_playlist(&pl.id, 0, 2).unwrap();
        let got = lib.get_playlist(&pl.id).unwrap();
        assert_eq!(got.song_ids, vec!["s2", "s3", "s1", "s4"]);
        // 把第 3 项移到第 0 位：-> [s4,s2,s3,s1]
        lib.reorder_playlist(&pl.id, 3, 0).unwrap();
        let got = lib.get_playlist(&pl.id).unwrap();
        assert_eq!(got.song_ids, vec!["s4", "s2", "s3", "s1"]);
    }

    #[test]
    fn test_favorites_export_import_roundtrip() {
        let (lib, _tmp) = temp_library();
        let g = lib.create_favorite_group("收藏").unwrap();
        for s in ["fa", "fb", "fc"] {
            lib.add_to_favorites(&g.id, s).unwrap();
        }
        let exported = lib.export_favorites(&g.id).unwrap();
        assert_eq!(exported["songs"], serde_json::json!(["fa", "fb", "fc"]));
        // 导入到新分组验证往返
        let g2 = lib.create_favorite_group("新收藏").unwrap();
        let songs: Vec<String> = exported["songs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        lib.import_favorites(&g2.id, &songs).unwrap();
        let g2_loaded = lib
            .list_favorite_groups()
            .unwrap()
            .into_iter()
            .find(|x| x.id == g2.id)
            .unwrap();
        assert_eq!(g2_loaded.song_ids, vec!["fa", "fb", "fc"]);
    }

    #[test]
    fn test_play_state_roundtrip() {
        let (lib, _tmp) = temp_library();
        assert!(lib.load_play_state().unwrap().is_none());
        let state = PlayState {
            current_song_id: Some("song-x".into()),
            playlist_id: Some("pl-1".into()),
            index: Some(2),
            position_ms: 12_345,
            duration_ms: 180_000,
            is_playing: true,
            volume: 0.65,
            mode: PlayMode::SingleLoop,
        };
        lib.save_play_state(&state).unwrap();
        let loaded = lib.load_play_state().unwrap().unwrap();
        assert_eq!(loaded, state);
    }
}
