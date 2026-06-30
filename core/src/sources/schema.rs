//! 标准 JSON Schema 对应的 Rust 类型与校验逻辑。

use serde::{Deserialize, Serialize};
use crate::{CoreError, CoreResult};

/// 音源配置（标准 Schema 的反序列化目标）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundSourceConfig {
    /// Schema 版本
    #[serde(rename = "schemaVersion")]
    pub schema_version: String,
    /// 音源元信息
    pub manifest: SourceManifest,
    /// 鉴权
    #[serde(default)]
    pub auth: Option<AuthConfig>,
    /// 字段映射（音源原始字段名 -> 标准字段）
    #[serde(default)]
    pub field_mapping: SourceFieldMapping,
    /// 端点
    pub endpoints: SourceEndpoints,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceManifest {
    /// 唯一 ID
    pub id: String,
    /// 展示名称
    pub name: String,
    /// 作者
    pub author: Option<String>,
    /// 版本
    pub version: Option<String>,
    /// 描述
    pub description: Option<String>,
    /// 主站
    pub homepage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    /// 鉴权类型：header / query / body / bearer
    #[serde(rename = "type")]
    pub auth_type: String,
    /// token 字段名
    pub name: String,
    /// 占位符 `{{auth.token}}`，运行时由用户填入
    pub token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceFieldMapping {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_artist")]
    pub artist: String,
    #[serde(default = "default_album")]
    pub album: String,
    #[serde(default = "default_cover")]
    pub cover: String,
    #[serde(default = "default_duration")]
    pub duration: String,
    #[serde(default = "default_lyric")]
    pub lyric: String,
    #[serde(default = "default_song_id")]
    pub song_id: String,
    #[serde(default = "default_play_url")]
    pub play_url: String,
}

fn default_title() -> String { "title".into() }
fn default_artist() -> String { "artist".into() }
fn default_album() -> String { "album".into() }
fn default_cover() -> String { "cover".into() }
fn default_duration() -> String { "duration".into() }
fn default_lyric() -> String { "lyric".into() }
fn default_song_id() -> String { "id".into() }
fn default_play_url() -> String { "url".into() }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceEndpoints {
    pub search: Option<SourceEndpoint>,
    pub metadata: Option<SourceEndpoint>,
    pub play: Option<SourceEndpoint>,
    pub lyric: Option<SourceEndpoint>,
    pub ranking: Option<SourceEndpoint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    Get,
    Post,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEndpoint {
    pub method: HttpMethod,
    pub url: String,
    /// 查询参数（支持 `{{auth.token}}` 占位符）
    #[serde(default)]
    pub params: Vec<(String, String)>,
    /// 请求头
    #[serde(default)]
    pub headers: Vec<(String, String)>,
    /// POST body 模板（JSON 字符串，支持占位符）
    #[serde(default)]
    pub body: Option<String>,
    /// 响应数据 JSONPath 风格路径（简化：点号路径）
    #[serde(default)]
    pub data_path: Option<String>,
}

impl SoundSourceConfig {
    /// 解析并校验一份标准音源 JSON
    pub fn from_json(json: &str) -> CoreResult<Self> {
        let cfg: SoundSourceConfig = serde_json::from_str(json).map_err(|e| {
            CoreError::SchemaValidation(format!("解析失败: {e}"))
        })?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// 基本校验
    pub fn validate(&self) -> CoreResult<()> {
        if self.manifest.id.trim().is_empty() {
            return Err(CoreError::SchemaValidation("manifest.id 不能为空".into()));
        }
        if self.manifest.name.trim().is_empty() {
            return Err(CoreError::SchemaValidation("manifest.name 不能为空".into()));
        }
        if self.endpoints.search.is_none() && self.endpoints.metadata.is_none() {
            return Err(CoreError::SchemaValidation(
                "endpoints 至少需要 search 或 metadata 之一".into(),
            ));
        }
        if let Some(ep) = &self.endpoints.search {
            ep.validate("search")?;
        }
        if let Some(ep) = &self.endpoints.metadata {
            ep.validate("metadata")?;
        }
        if let Some(ep) = &self.endpoints.play {
            ep.validate("play")?;
        }
        Ok(())
    }
}

impl SourceEndpoint {
    pub fn validate(&self, name: &str) -> CoreResult<()> {
        if self.url.trim().is_empty() {
            return Err(CoreError::SchemaValidation(format!(
                "endpoints.{name}.url 不能为空"
            )));
        }
        let _ = url::Url::parse(&self.url).map_err(|e| {
            CoreError::SchemaValidation(format!("endpoints.{name}.url 非法: {e}"))
        })?;
        Ok(())
    }
}
