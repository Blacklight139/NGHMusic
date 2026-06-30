//! 音源引擎：加载、校验、启用/禁用、优先级、元数据获取、播放数据定位、歌词获取。
//!
//! 标准 JSON Schema 见 `schemas/sound-source.schema.json`。

pub mod schema;
pub mod migration;
pub mod engine;
pub mod http_source;

pub use engine::{SourceEngine, SourceHandle};
pub use schema::{SoundSourceConfig, SourceFieldMapping, SourceEndpoint, AuthConfig, HttpMethod};
