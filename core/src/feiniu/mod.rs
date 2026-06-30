//! 飞牛 API 客户端：鉴权、列目录、取流。

use crate::{CoreError, CoreResult};
use serde::{Deserialize, Serialize};

pub struct FeiniuClient {
    http: reqwest::Client,
    base_url: String,
    token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FeiniuConfig {
    /// 飞牛 NAS 地址，如 https://nas.example.com:5666
    pub base_url: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeiniuLoginResp {
    pub token: Option<String>,
    #[serde(default)]
    pub msg: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeiniuFile {
    pub name: String,
    #[serde(default)]
    pub is_dir: bool,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FeiniuListResp {
    #[serde(default)]
    pub files: Vec<FeiniuFile>,
    #[serde(default)]
    pub msg: Option<String>,
}

impl FeiniuClient {
    pub fn new(cfg: FeiniuConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            http,
            base_url: cfg.base_url.trim_end_matches('/').to_string(),
            token: None,
        }
    }

    /// 登录获取 token
    pub async fn login(&mut self, cfg: &FeiniuConfig) -> CoreResult<()> {
        let url = format!("{}/api/v1/auth/login", self.base_url);
        let body = serde_json::json!({
            "username": cfg.username,
            "password": cfg.password,
        });
        let resp = self.http.post(&url).json(&body).send().await
            .map_err(|e| CoreError::Feiniu(e.to_string()))?;
        let status = resp.status();
        let parsed: FeiniuLoginResp = resp.json().await
            .map_err(|e| CoreError::Feiniu(format!("解析登录响应失败: {e}")))?;
        if !status.is_success() {
            return Err(CoreError::Feiniu(format!(
                "登录失败 {} : {}",
                status,
                parsed.msg.unwrap_or_default()
            )));
        }
        let token = parsed.token.ok_or_else(|| CoreError::Feiniu("登录响应缺少 token".into()))?;
        self.token = Some(token);
        Ok(())
    }

    fn auth(&self) -> CoreResult<reqwest::RequestBuilder> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| CoreError::Auth("未登录飞牛".into()))?;
        Ok(self.http.get("").bearer_auth(token))
    }

    /// 列目录
    pub async fn list_dir(&self, path: &str) -> CoreResult<Vec<FeiniuFile>> {
        let url = format!("{}/api/v1/fs/list", self.base_url);
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| CoreError::Auth("未登录飞牛".into()))?;
        let resp = self
            .http
            .get(&url)
            .bearer_auth(token)
            .query(&[("path", path)])
            .send()
            .await
            .map_err(|e| CoreError::Feiniu(e.to_string()))?;
        let status = resp.status();
        let parsed: FeiniuListResp = resp.json().await
            .map_err(|e| CoreError::Feiniu(format!("解析列表响应失败: {e}")))?;
        if !status.is_success() {
            return Err(CoreError::Feiniu(format!(
                "列目录失败 {} : {}",
                status,
                parsed.msg.unwrap_or_default()
            )));
        }
        Ok(parsed.files)
    }

    /// 取流地址
    pub async fn stream_url(&self, path: &str) -> CoreResult<String> {
        let token = self
            .token
            .as_ref()
            .ok_or_else(|| CoreError::Auth("未登录飞牛".into()))?;
        Ok(format!(
            "{}/api/v1/fs/stream?path={}&token={}",
            self.base_url,
            urlencoding(path),
            token
        ))
    }
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char);
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}
