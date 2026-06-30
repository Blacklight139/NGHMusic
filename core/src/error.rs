//! 核心错误类型定义。
//!
//! 统一封装 IO、JSON 解析、HTTP、音源、Schema、缓存、协议、飞牛、FFI
//! 等错误，便于上层通过 `Result<T>` 传播与匹配。外部错误类型通过
//! `#[from]` 自动转换。

use thiserror::Error;

/// 核心库统一错误类型
#[derive(Debug, Error)]
pub enum CoreError {
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON 序列化/反序列化错误: {0}")]
    Json(#[from] serde_json::Error),

    #[error("HTTP 请求错误: {0}")]
    Http(#[from] reqwest::Error),

    #[error("音源错误: {0}")]
    Source(String),

    #[error("数据 Schema 错误: {0}")]
    Schema(String),

    #[error("未找到资源: {0}")]
    NotFound(String),

    #[error("缓存错误: {0}")]
    Cache(String),

    #[error("协议错误: {0}")]
    Protocol(String),

    #[error("飞牛错误: {0}")]
    Feiniu(String),

    #[error("FFI 错误: {0}")]
    Ffi(String),
}

/// 核心库统一 Result 别名
pub type Result<T> = std::result::Result<T, CoreError>;
