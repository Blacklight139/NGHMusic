//! 播放器逻辑（不含音频后端）：维护当前播放列表、索引、播放模式、状态。
//!
//! 音频后端由各端原生实现，通过回调注入。

use parking_lot::Mutex;
use std::sync::Arc;
use crate::models::*;

pub struct Player {
    inner: Arc<Mutex<PlayerInner>>,
}

struct PlayerInner {
    playlist: Playlist,
    current_index: Option<usize>,
    state: PlayerState,
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

impl Player {
    pub fn new() -> Self {
        let playlist = Playlist::new("current");
        let id = playlist.id.clone();
        Self {
            inner: Arc::new(Mutex::new(PlayerInner {
                playlist,
                current_index: None,
                state: PlayerState {
                    playlist_id: Some(id),
                    volume: 1.0,
                    mode: PlayMode::Sequence,
                    ..Default::default()
                },
            })),
        }
    }

    pub fn snapshot(&self) -> PlayerState {
        self.inner.lock().state.clone()
    }

    pub fn playlist(&self) -> Playlist {
        self.inner.lock().playlist.clone()
    }

    pub fn set_playlist(&self, mut playlist: Playlist) {
        let mut g = self.inner.lock();
        // 保留当前歌曲索引（若仍在列表中）
        if let Some(cur) = g.state.current.clone() {
            g.current_index = playlist.songs.iter().position(|s| s == &cur);
        }
        if playlist.songs.is_empty() {
            g.current_index = None;
            g.state.playing = false;
        }
        g.state.playlist_id = Some(playlist.id.clone());
        std::mem::swap(&mut g.playlist, &mut playlist);
    }

    pub fn add(&self, song: SongRef) {
        let mut g = self.inner.lock();
        g.playlist.songs.push(song);
    }

    pub fn remove(&self, idx: usize) {
        let mut g = self.inner.lock();
        if idx >= g.playlist.songs.len() {
            return;
        }
        g.playlist.songs.remove(idx);
        if let Some(ci) = g.current_index {
            if idx < ci {
                g.current_index = Some(ci - 1);
            } else if idx == ci {
                g.current_index = if g.playlist.songs.is_empty() {
                    None
                } else {
                    Some(ci.min(g.playlist.songs.len() - 1))
                };
            }
        }
    }

    pub fn clear(&self) {
        let mut g = self.inner.lock();
        g.playlist.songs.clear();
        g.current_index = None;
        g.state.current = None;
        g.state.playing = false;
        g.state.position_sec = 0.0;
    }

    pub fn move_item(&self, from: usize, to: usize) {
        let mut g = self.inner.lock();
        if from >= g.playlist.songs.len() || to >= g.playlist.songs.len() {
            return;
        }
        let song = g.playlist.songs.remove(from);
        g.playlist.songs.insert(to, song);
        if let Some(ci) = g.current_index {
            if ci == from {
                g.current_index = Some(to);
            } else if from < ci && to >= ci {
                g.current_index = Some(ci - 1);
            } else if from > ci && to <= ci {
                g.current_index = Some(ci + 1);
            }
        }
    }

    pub fn play_at(&self, idx: usize) -> Option<SongRef> {
        let mut g = self.inner.lock();
        if idx >= g.playlist.songs.len() {
            return None;
        }
        g.current_index = Some(idx);
        let s = g.playlist.songs[idx].clone();
        g.state.current = Some(s.clone());
        g.state.playing = true;
        g.state.position_sec = 0.0;
        Some(s)
    }

    pub fn next(&self) -> Option<SongRef> {
        let mut g = self.inner.lock();
        let len = g.playlist.songs.len();
        if len == 0 {
            return None;
        }
        let idx = match g.current_index {
            None => 0,
            Some(i) => match g.state.mode {
                PlayMode::Shuffle => {
                    use rand_chrono;
                    let next = rand_chrono(len);
                    next
                }
                PlayMode::RepeatOne => i,
                PlayMode::Sequence => (i + 1) % len,
            },
        };
        drop(g);
        self.play_at(idx)
    }

    pub fn prev(&self) -> Option<SongRef> {
        let g = self.inner.lock();
        let len = g.playlist.songs.len();
        if len == 0 {
            return None;
        }
        let i = g.current_index.unwrap_or(0);
        let prev = (i + len - 1) % len;
        drop(g);
        self.play_at(prev)
    }

    pub fn set_position(&self, sec: f64) {
        self.inner.lock().state.position_sec = sec;
    }

    pub fn set_volume(&self, vol: f32) {
        self.inner.lock().state.volume = vol.clamp(0.0, 1.0);
    }

    pub fn set_mode(&self, mode: PlayMode) {
        self.inner.lock().state.mode = mode;
    }

    pub fn set_playing(&self, playing: bool) {
        self.inner.lock().state.playing = playing;
    }

    pub fn current(&self) -> Option<SongRef> {
        self.inner.lock().state.current.clone()
    }
}

fn rand_chrono(_len: usize) -> usize {
    // 简化伪随机：基于系统时间
    use std::time::{SystemTime, UNIX_EPOCH};
    let n = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as usize)
        .unwrap_or(0);
    n % _len.max(1)
}
