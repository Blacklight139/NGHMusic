//! 核心错误类型。统一错误码便于各端 FFI 转译为原生错误。

use thiserror::Error;

/// 错误码，用于 FFI 层透传到各端
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// 未知错误
    Unknown,
    /// 非法参数 / 边界值
    InvalidArgument,
    /// 音源 Schema 校验失败
    SchemaValidation,
    /// 音源不存在或未启用
    SourceNotFound,
    /// 音源 API 调用失败
    SourceApi,
    /// 网络请求失败
    Network,
    /// 鉴权失败
    Auth,
    /// 缓存读写失败
    Cache,
    /// 协议客户端错误（SMB/WebDAV/FTP/DLNA/NFS）
    Protocol,
    /// 飞牛 API 错误
    Feiniu,
    /// 存储（持久化）错误
    Storage,
    /// 解析错误（JSON/LRC 等）
    Parse,
    /// 未找到资源（歌曲/歌词等）
    NotFound,
    /// IO 错误
    Io,
}

#[derive(Debug, Error, serde::Serialize)]
#[serde(tag = "kind", content = "detail")]
pub enum CoreError {
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("schema validation failed: {0}")]
    SchemaValidation(String),

    #[error("source not found: {0}")]
    SourceNotFound(String),

    #[error("source api error: {0}")]
    SourceApi(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("auth error: {0}")]
    Auth(String),

    #[error("cache error: {0}")]
    Cache(String),

    #[error("protocol error ({protocol}): {message}")]
    Protocol { protocol: String, message: String },

    #[error("feiniu api error: {0}")]
    Feiniu(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("parse error: {0}")]
    Parse(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("unknown error: {0}")]
    Unknown(String),
}

impl CoreError {
    pub fn code(&self) -> ErrorCode {
        match self {
            CoreError::InvalidArgument(_) => ErrorCode::InvalidArgument,
            CoreError::SchemaValidation(_) => ErrorCode::SchemaValidation,
            CoreError::SourceNotFound(_) => ErrorCode::SourceNotFound,
            CoreError::SourceApi(_) => ErrorCode::SourceApi,
            CoreError::Network(_) => ErrorCode::Network,
            CoreError::Auth(_) => ErrorCode::Auth,
            CoreError::Cache(_) => ErrorCode::Cache,
            CoreError::Protocol { .. } => ErrorCode::Protocol,
            CoreError::Feiniu(_) => ErrorCode::Feiniu,
            CoreError::Storage(_) => ErrorCode::Storage,
            CoreError::Parse(_) => ErrorCode::Parse,
            CoreError::NotFound(_) => ErrorCode::NotFound,
            CoreError::Io(_) => ErrorCode::Io,
            CoreError::Unknown(_) => ErrorCode::Unknown,
        }
    }
}

impl From<std::io::Error> for CoreError {
    fn from(e: std::io::Error) -> Self {
        CoreError::Io(e.to_string())
    }
}

impl From<serde_json::Error> for CoreError {
    fn from(e: serde_json::Error) -> Self {
        CoreError::Parse(e.to_string())
    }
}

impl From<reqwest::Error> for CoreError {
    fn from(e: reqwest::Error) -> Self {
        if e.is_connect() || e.is_timeout() {
            CoreError::Network(e.to_string())
        } else if e.is_status() {
            CoreError::SourceApi(e.to_string())
        } else {
            CoreError::Network(e.to_string())
        }
    }
}

impl From<url::ParseError> for CoreError {
    fn from(e: url::ParseError) -> Self {
        CoreError::Parse(format!("url: {e}"))
    }
}

impl From<anyhow::Error> for CoreError {
    fn from(e: anyhow::Error) -> Self {
        CoreError::Unknown(e.to_string())
    }
}

pub type CoreResult<T> = Result<T, CoreError>;
