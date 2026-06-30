//! 飞牛 NAS API 客户端。
//!
//! 飞牛是 NAS 系统，提供 HTTP API。本模块封装登录、文件列表、
//! 流式播放 URL 获取与健康检查，作为 NAS 音源的具体实现之一。
//! 所有 HTTP/serde 错误统一映射为 [`CoreError::Feiniu`](crate::error::CoreError::Feiniu)，
//! 非 2xx 响应的错误信息包含状态码与响应体片段。

use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::{CoreError, Result};

/// 飞牛 NAS 文件条目。
///
/// 兼容 `is_dir` 与 `isDir` 两种字段命名（通过 serde alias）。
/// `size` 与 `modified` 缺失时使用默认值，保证对真实飞牛 API 字段的健壮解析。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NasFile {
    /// 文件/目录名称
    pub name: String,
    /// 是否为目录（兼容 `isDir`）
    #[serde(alias = "isDir", default)]
    pub is_dir: bool,
    /// 字节大小
    #[serde(default)]
    pub size: u64,
    /// 修改时间（原始字符串，格式由服务端决定）
    #[serde(default)]
    pub modified: Option<String>,
}

/// 飞牛 NAS HTTP API 客户端。
pub struct FeiniuClient {
    /// API 基础地址，如 `https://nas.example.com`
    pub base_url: String,
    /// 复用的 reqwest 客户端
    pub client: reqwest::Client,
    /// 登录后的访问令牌
    pub token: Option<String>,
}

impl FeiniuClient {
    /// 构造客户端，初始未登录。
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
            token: None,
        }
    }

    /// 拼接完整接口地址（去除 base_url 末尾多余的 `/`）。
    fn endpoint(&self, path: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), path)
    }

    /// 登录并保存 token。
    ///
    /// POST `/api/v1/auth/login`，body 为 `{username,password}`。
    /// 健壮处理响应：兼容 `token` / `access_token`，以及 `data` 嵌套。
    pub async fn login(&mut self, username: &str, password: &str) -> Result<()> {
        #[derive(Serialize)]
        struct LoginReq<'a> {
            username: &'a str,
            password: &'a str,
        }

        let url = self.endpoint("/api/v1/auth/login");
        let resp = self
            .client
            .post(&url)
            .json(&LoginReq { username, password })
            .send()
            .await
            .map_err(|e| CoreError::Feiniu(format!("登录请求失败: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CoreError::Feiniu(format!(
                "登录失败: 状态 {} {}",
                status,
                body.chars().take(200).collect::<String>()
            )));
        }

        let value: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| CoreError::Feiniu(format!("解析登录响应失败: {}", e)))?;

        // 健壮处理：兼容 token / access_token，及 data 嵌套
        let token = value
            .get("token")
            .or_else(|| value.get("access_token"))
            .or_else(|| value.get("data").and_then(|d| d.get("token")))
            .or_else(|| value.get("data").and_then(|d| d.get("access_token")))
            .and_then(|v| v.as_str())
            .ok_or_else(|| CoreError::Feiniu("登录响应缺少 token 字段".into()))?
            .to_string();
        self.token = Some(token);
        Ok(())
    }

    /// 列出指定路径下的文件。
    ///
    /// GET `/api/v1/files?path=...`，携带 `Authorization: Bearer <token>`。
    /// 响应可为 `{files:[...]}` 或裸数组，逐项解析为 [`NasFile`]，失败项跳过。
    pub async fn list_files(&self, path: &str) -> Result<Vec<NasFile>> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| CoreError::Feiniu("未登录".into()))?;

        let url = self.endpoint("/api/v1/files");
        let resp = self
            .client
            .get(&url)
            .bearer_auth(token.as_str())
            .query(&[("path", path)])
            .send()
            .await
            .map_err(|e| CoreError::Feiniu(format!("list_files 请求失败: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CoreError::Feiniu(format!(
                "list_files 失败: 状态 {} {}",
                status,
                body.chars().take(200).collect::<String>()
            )));
        }

        let value: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| CoreError::Feiniu(format!("解析文件列表响应失败: {}", e)))?;

        // 健壮处理：{files:[...]} 或裸数组
        let arr = value
            .get("files")
            .and_then(|v| v.as_array())
            .or_else(|| value.as_array())
            .ok_or_else(|| CoreError::Feiniu("文件列表响应非数组".into()))?;

        // 逐项解析，失败项跳过以保证健壮性
        let files: Vec<NasFile> = arr
            .iter()
            .filter_map(|item| serde_json::from_value::<NasFile>(item.clone()).ok())
            .collect();
        Ok(files)
    }

    /// 生成流式播放 URL。
    ///
    /// 返回 `{base_url}/api/v1/files/stream?path=<encoded>`，
    /// 若已登录则附带 `token` 作为 query 参数。`path` 自动做 URL 编码。
    pub async fn get_stream_url(&self, path: &str) -> Result<String> {
        let mut url = Url::parse(&self.endpoint("/api/v1/files/stream"))
            .map_err(|e| CoreError::Feiniu(format!("URL 构造失败: {}", e)))?;
        url.query_pairs_mut().append_pair("path", path);
        if let Some(token) = &self.token {
            url.query_pairs_mut().append_pair("token", token);
        }
        Ok(url.to_string())
    }

    /// 健康检查。
    ///
    /// GET `/api/v1/health`，2xx 返回 `Ok`，否则返回 [`CoreError::Feiniu`]。
    /// 用于异常检测与重试判断。
    pub async fn ping(&self) -> Result<()> {
        let url = self.endpoint("/api/v1/health");
        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| CoreError::Feiniu(format!("健康检查请求失败: {}", e)))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            Err(CoreError::Feiniu(format!(
                "健康检查失败: 状态 {} {}",
                status,
                body.chars().take(200).collect::<String>()
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn get_stream_url_encodes_path() {
        let client = FeiniuClient::new("https://nas.example.com".into());
        let url = client.get_stream_url("/music/my file.mp3").await.unwrap();
        assert!(url.contains("/api/v1/files/stream"));
        assert!(url.contains("path="));
        // 原始未编码的路径不应直接出现
        assert!(!url.contains("path=/music/my file.mp3"));
        // form-urlencoded：空格 -> +，斜杠 -> %2F
        assert!(url.contains("my+file.mp3"));
        assert!(url.contains("%2Fmusic"));
    }

    #[tokio::test]
    async fn get_stream_url_handles_trailing_slash() {
        let client = FeiniuClient::new("https://nas.example.com/".into());
        let url = client.get_stream_url("song.mp3").await.unwrap();
        assert!(url.starts_with("https://nas.example.com/"));
        assert!(url.contains("/api/v1/files/stream"));
    }

    #[tokio::test]
    async fn get_stream_url_appends_token_when_logged_in() {
        let mut client = FeiniuClient::new("https://nas.example.com".into());
        client.token = Some("abc123".into());
        let url = client.get_stream_url("/a.mp3").await.unwrap();
        assert!(url.contains("token=abc123"));
    }

    #[test]
    fn nasfile_deserializes_snake_and_camel() {
        let snake = r#"{"name":"a.mp3","is_dir":false,"size":100,"modified":"2024-01-01"}"#;
        let f: NasFile = serde_json::from_str(snake).unwrap();
        assert_eq!(f.name, "a.mp3");
        assert!(!f.is_dir);
        assert_eq!(f.size, 100);
        assert_eq!(f.modified.as_deref(), Some("2024-01-01"));

        let camel = r#"{"name":"b","isDir":true,"size":0}"#;
        let f: NasFile = serde_json::from_str(camel).unwrap();
        assert!(f.is_dir);
        assert_eq!(f.size, 0);
        assert!(f.modified.is_none());
    }
}
