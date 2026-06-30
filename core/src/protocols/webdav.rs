//! WebDAV 客户端（PROPFIND）

use async_trait::async_trait;
use crate::models::{WebDavConfig, ProtocolEntry};
use crate::{CoreError, CoreResult};

pub struct WebDavClient {
    cfg: WebDavConfig,
    http: reqwest::Client,
}

impl WebDavClient {
    pub fn new(cfg: WebDavConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self { cfg, http }
    }
}

#[async_trait]
impl super::ProtocolClient for WebDavClient {
    fn kind(&self) -> &'static str { "webdav" }

    async fn list(&self, path: &str) -> CoreResult<Vec<ProtocolEntry>> {
        let base = self.cfg.url.trim_end_matches('/');
        let url = format!("{base}/{}", path.trim_start_matches('/'));
        let method = reqwest::Method::from_bytes(b"PROPFIND").unwrap();
        let mut req = self.http.request(method, &url)
            .header("Depth", "1")
            .header("Content-Type", "application/xml; charset=utf-8")
            .body(r#"<?xml version="1.0" encoding="utf-8"?><propfind xmlns="DAV:"><prop><displayname/><getcontentlength/><resourcetype/></prop></propfind>"#);
        if let (Some(u), Some(p)) = (&self.cfg.username, &self.cfg.password) {
            req = req.basic_auth(u, Some(p));
        }
        let resp = req.send().await.map_err(|e| CoreError::Protocol {
            protocol: "webdav".into(),
            message: e.to_string(),
        })?;
        let status = resp.status();
        if !status.is_success() && status.as_u16() != 207 {
            return Err(CoreError::Protocol {
                protocol: "webdav".into(),
                message: format!("PROPFIND 状态码 {status}"),
            });
        }
        let body = resp.text().await.map_err(|e| CoreError::Protocol {
            protocol: "webdav".into(),
            message: e.to_string(),
        })?;
        Ok(parse_propfind(&body, &url))
    }
}

/// 极简 PROPFIND 解析（按 href/displayname/getcontentlength/resourcetype 抽取）
fn parse_propfind(xml: &str, base_url: &str) -> Vec<ProtocolEntry> {
    let mut out = Vec::new();
    let href_re = regex::Regex::new(r"(?i)<[^>]*href[^>]*>([^<]+)<").unwrap();
    let name_re = regex::Regex::new(r"(?i)<[^>]*displayname[^>]*>([^<]+)<").unwrap();
    let len_re = regex::Regex::new(r"(?i)<[^>]*getcontentlength[^>]*>([^<]+)<").unwrap();
    let coll_re = regex::Regex::new(r"(?i)<[^>]*collection[^>]*/>").unwrap();

    // 兼容 D: 与 d: 命名空间前缀
    let resp_iter: Vec<&str> = {
        let v: Vec<&str> = xml.split("<D:response>").skip(1).collect();
        if v.is_empty() {
            xml.split("<d:response>").skip(1).collect()
        } else {
            v
        }
    };

    for r in resp_iter {
        let href = href_re.captures(r).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());
        let name = name_re.captures(r).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());
        let size = len_re.captures(r).and_then(|c| c.get(1)).and_then(|m| m.as_str().parse::<u64>().ok());
        let is_dir = coll_re.is_match(r);
        if let Some(href) = href {
            let name = name.unwrap_or_else(|| {
                href.trim_end_matches('/').rsplit('/').next().unwrap_or("").to_string()
            });
            let url = if href.starts_with("http") {
                href
            } else {
                let path = if href.starts_with('/') { href.clone() } else { format!("/{href}") };
                format!("{}{}", base_url.trim_end_matches('/'), path)
            };
            out.push(ProtocolEntry { name, is_dir, size, url });
        }
    }
    out
}
