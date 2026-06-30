//! 持久化存储：将播放器状态、播放列表、收藏夹、音源配置持久化为 JSON 文件。

use std::path::PathBuf;
use tokio::fs;
use crate::models::*;
use crate::{CoreError, CoreResult};

pub struct Storage {
    dir: PathBuf,
}

impl Storage {
    pub async fn new(dir: PathBuf) -> CoreResult<Self> {
        fs::create_dir_all(&dir).await.map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(Self { dir })
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(format!("{name}.json"))
    }

    pub async fn save<T: serde::Serialize>(&self, name: &str, v: &T) -> CoreResult<()> {
        let data = serde_json::to_vec_pretty(v)?;
        fs::write(self.path(name), &data)
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))
    }

    pub async fn load<T: serde::de::DeserializeOwned>(&self, name: &str) -> CoreResult<Option<T>> {
        match fs::read(self.path(name)).await {
            Ok(data) => {
                let v = serde_json::from_slice(&data)?;
                Ok(Some(v))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(CoreError::Storage(e.to_string())),
        }
    }

    pub async fn save_player_state(&self, s: &PlayerState) -> CoreResult<()> {
        self.save("player_state", s).await
    }
    pub async fn load_player_state(&self) -> CoreResult<Option<PlayerState>> {
        self.load("player_state").await
    }

    pub async fn save_playlists(&self, p: &[Playlist]) -> CoreResult<()> {
        self.save("playlists", &p).await
    }
    pub async fn load_playlists(&self) -> CoreResult<Vec<Playlist>> {
        Ok(self.load::<Vec<Playlist>>("playlists").await?.unwrap_or_default())
    }

    pub async fn save_favorites(&self, p: &[FavoriteGroup]) -> CoreResult<()> {
        self.save("favorites", &p).await
    }
    pub async fn load_favorites(&self) -> CoreResult<Vec<FavoriteGroup>> {
        Ok(self.load::<Vec<FavoriteGroup>>("favorites").await?.unwrap_or_default())
    }
}
