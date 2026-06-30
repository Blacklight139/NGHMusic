//! 元数据 API 客户端。
//!
//! 通过 HTTP 抓取在线音源元数据 JSON，供上层补全歌曲信息。
//! - HTTP 非 2xx 返回 [`CoreError::Http`](crate::error::CoreError::Http)。
//! - JSON 反序列化失败返回 [`CoreError::Json`](crate::error::CoreError::Json)。

use crate::error::Result;

/// 元数据获取客户端。
pub struct MetadataClient {
    client: reqwest::Client,
}

impl MetadataClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// 抓取指定 URL 的 JSON 内容。
    ///
    /// - `url`：目标地址。
    /// - `headers`：可选的自定义请求头列表（`&[(&str, &str)]`）。
    pub async fn fetch_json(
        &self,
        url: &str,
        headers: Option<&[(&str, &str)]>,
    ) -> Result<serde_json::Value> {
        let mut req = self.client.get(url);
        if let Some(hs) = headers {
            for (k, v) in hs {
                req = req.header(*k, *v);
            }
        }
        let response = req.send().await?;
        let response = response.error_for_status()?;
        let text = response.text().await?;
        let value: serde_json::Value = serde_json::from_str(&text)?;
        Ok(value)
    }
}

impl Default for MetadataClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_returns_default() {
        let _client = MetadataClient::new();
        let _client = MetadataClient::default();
    }

    #[tokio::test]
    async fn fetch_json_invalid_url_returns_err() {
        let client = MetadataClient::new();
        // 非法 URL 应在发送阶段就报错（HTTP 错误）。
        let result = client.fetch_json("ht!tp://invalid url with space", None).await;
        assert!(result.is_err());
    }
}
