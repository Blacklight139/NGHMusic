//! WebDAV 协议客户端。
//!
//! 基于 HTTP 实现 WebDAV 访问：PROPFIND 列目录、GET 读取文件、
//! 生成直链 URL。支持可选的 Basic auth。multistatus XML 通过简单
//! 字符串扫描解析，避免引入重 XML 库。
//! HTTP 错误统一映射为 [`CoreError::Protocol`](crate::error::CoreError::Protocol)。

use async_trait::async_trait;
use reqwest::Method;

use crate::error::{CoreError, Result};
use crate::protocols::ProtocolClient;

/// PROPFIND allprop 请求体。
const PROPFIND_BODY: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<D:propfind xmlns:D="DAV:">
  <D:allprop/>
</D:propfind>"#;

/// WebDAV 协议客户端。
pub struct WebDavClient {
    /// 服务器根 URL，如 `https://server/dav`
    pub base_url: String,
    /// 复用的 reqwest 客户端
    pub client: reqwest::Client,
    /// 可选 Basic 鉴权（用户名、密码）
    pub auth: Option<(String, String)>,
}

impl WebDavClient {
    /// 构造客户端。若 `username` 与 `password` 均为 `Some`，则启用 Basic auth。
    pub fn new(base_url: String, username: Option<String>, password: Option<String>) -> Self {
        let auth = match (username, password) {
            (Some(u), Some(p)) => Some((u, p)),
            _ => None,
        };
        Self {
            base_url,
            client: reqwest::Client::new(),
            auth,
        }
    }

    /// 拼接完整 URL。
    fn build_url(&self, path: &str) -> String {
        let base = self.base_url.trim_end_matches('/');
        let p = path.trim_start_matches('/');
        if p.is_empty() {
            base.to_string()
        } else {
            format!("{}/{}", base, p)
        }
    }

    /// 为请求注入 Basic auth（若配置）。
    fn with_auth<'a>(&'a self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.auth {
            Some((u, p)) => req.basic_auth(u.as_str(), Some(p.as_str())),
            None => req,
        }
    }
}

#[async_trait]
impl ProtocolClient for WebDavClient {
    async fn list(&self, path: &str) -> Result<Vec<String>> {
        let url = self.build_url(path);
        let method = Method::from_bytes(b"PROPFIND")
            .map_err(|e| CoreError::Protocol(format!("无效 HTTP 方法: {}", e)))?;
        let req = self
            .client
            .request(method, &url)
            .header("Depth", "1")
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(PROPFIND_BODY.to_string());
        let resp = self
            .with_auth(req)
            .send()
            .await
            .map_err(|e| CoreError::Protocol(format!("WebDAV PROPFIND 请求失败: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CoreError::Protocol(format!(
                "WebDAV PROPFIND 失败: 状态 {} {}",
                status,
                body.chars().take(200).collect::<String>()
            )));
        }
        let xml = resp
            .text()
            .await
            .map_err(|e| CoreError::Protocol(format!("读取 PROPFIND 响应失败: {}", e)))?;
        Ok(parse_propfind(&xml))
    }

    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        let url = self.build_url(path);
        let resp = self
            .with_auth(self.client.get(&url))
            .send()
            .await
            .map_err(|e| CoreError::Protocol(format!("WebDAV GET 请求失败: {}", e)))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(CoreError::Protocol(format!(
                "WebDAV GET 失败: 状态 {} {}",
                status,
                body.chars().take(200).collect::<String>()
            )));
        }
        let bytes = resp
            .bytes()
            .await
            .map_err(|e| CoreError::Protocol(format!("读取 WebDAV 文件失败: {}", e)))?;
        Ok(bytes.to_vec())
    }

    async fn stream_url(&self, path: &str) -> Result<String> {
        Ok(self.build_url(path))
    }
}

/// 简单解析 PROPFIND multistatus XML，提取所有 `href` 元素内容。
///
/// 兼容 `<d:href>`、`<D:href>`、`<href>` 等命名空间写法，
/// 采用字节扫描实现，避免引入完整 XML 解析依赖。
fn parse_propfind(xml: &str) -> Vec<String> {
    let mut hrefs = Vec::new();
    let bytes = xml.as_bytes();
    let n = bytes.len();
    let mut i = 0;
    while i < n {
        if bytes[i] == b'<' {
            // 提取起始标签名（到 > 或空白为止）
            let tag_start = i + 1;
            let mut j = tag_start;
            while j < n
                && bytes[j] != b'>'
                && bytes[j] != b' '
                && bytes[j] != b'\t'
                && bytes[j] != b'\n'
                && bytes[j] != b'\r'
            {
                j += 1;
            }
            let tag_name = &xml[tag_start..j];
            let tag_lower = tag_name.to_lowercase();
            let is_href = tag_lower == "href" || tag_lower.ends_with(":href");
            // 跳过可能存在的属性，定位到 '>'
            while j < n && bytes[j] != b'>' {
                j += 1;
            }
            if is_href {
                if j >= n {
                    break;
                }
                // 取标签内文本，直到下一个 '<'
                let text_start = j + 1;
                let mut k = text_start;
                while k < n && bytes[k] != b'<' {
                    k += 1;
                }
                let text = xml[text_start..k].trim();
                if !text.is_empty() {
                    hrefs.push(text.to_string());
                }
                i = k;
                continue;
            }
            i = j + 1;
        } else {
            i += 1;
        }
    }
    hrefs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_propfind_extracts_hrefs() {
        let xml = r#"<?xml version="1.0"?>
<d:multistatus xmlns:d="DAV:">
  <d:response>
    <d:href>/music/</d:href>
  </d:response>
  <d:response>
    <d:href>/music/song1.mp3</d:href>
  </d:response>
  <d:response>
    <d:href>/music/song2.mp3</d:href>
  </d:response>
</d:multistatus>"#;
        let hrefs = parse_propfind(xml);
        assert_eq!(hrefs.len(), 3);
        assert_eq!(hrefs[0], "/music/");
        assert_eq!(hrefs[1], "/music/song1.mp3");
        assert_eq!(hrefs[2], "/music/song2.mp3");
    }

    #[test]
    fn parse_propfind_uppercase_namespace() {
        let xml = r#"<D:multistatus xmlns:D="DAV:">
  <D:response><D:href>/a/b.txt</D:href></D:response>
  <D:response><D:href>/a/c.txt</D:href></D:response>
</D:multistatus>"#;
        let hrefs = parse_propfind(xml);
        assert_eq!(hrefs.len(), 2);
        assert_eq!(hrefs[0], "/a/b.txt");
        assert_eq!(hrefs[1], "/a/c.txt");
    }

    #[test]
    fn parse_propfind_no_namespace() {
        let xml = r#"<multistatus><response><href>/x.txt</href></response></multistatus>"#;
        let hrefs = parse_propfind(xml);
        assert_eq!(hrefs, vec!["/x.txt".to_string()]);
    }

    #[test]
    fn parse_propfind_empty() {
        assert!(parse_propfind("").is_empty());
        assert!(parse_propfind("<a></a>").is_empty());
    }
}
