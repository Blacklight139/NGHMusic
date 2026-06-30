//! 基于 HTTP 的音源请求客户端（reqwest）。

use crate::sources::schema::{SoundSourceConfig, SourceEndpoint, HttpMethod, AuthConfig};
use crate::{CoreError, CoreResult};
use serde_json::Value;

/// 执行一次音源端点请求，返回解析后的 JSON
pub async fn request_endpoint(
    client: &reqwest::Client,
    cfg: &SoundSourceConfig,
    ep: &SourceEndpoint,
    // 占位符替换：`{{auth.token}}` 等
    substitutions: &[(&str, &str)],
) -> CoreResult<Value> {
    let url = substitute(&ep.url, substitutions);
    let mut req = match ep.method {
        HttpMethod::Get => client.get(&url),
        HttpMethod::Post => client.post(&url),
    };

    // headers
    for (k, v) in &ep.headers {
        req = req.header(k, substitute(v, substitutions));
    }
    if let Some(auth) = &cfg.auth {
        req = apply_auth(req, auth, substitutions)?;
    }

    // params / body
    match ep.method {
        HttpMethod::Get => {
            for (k, v) in &ep.params {
                req = req.query(&[(k.as_str(), substitute(v, substitutions))]);
            }
        }
        HttpMethod::Post => {
            if let Some(body) = &ep.body {
                req = req
                    .header("Content-Type", "application/json")
                    .body(substitute(body, substitutions));
            } else if !ep.params.is_empty() {
                let mut map = serde_json::Map::new();
                for (k, v) in &ep.params {
                    map.insert(k.clone(), Value::String(substitute(v, substitutions)));
                }
                req = req.json(&Value::Object(map));
            }
        }
    }

    let resp = req.send().await.map_err(|e| CoreError::Network(e.to_string()))?;
    let status = resp.status();
    let body_text = resp.text().await.map_err(|e| CoreError::Network(e.to_string()))?;
    if !status.is_success() {
        return Err(CoreError::SourceApi(format!(
            "{} 状态码 {} : {}",
            url, status, truncate(&body_text, 200)
        )));
    }
    let value: Value = serde_json::from_str(&body_text)
        .map_err(|e| CoreError::Parse(format!("响应非 JSON: {e}")))?;
    // 沿 data_path 取值
    let value = if let Some(path) = &ep.data_path {
        extract_path(&value, path)?
    } else {
        value
    };
    Ok(value)
}

/// 应用鉴权（按值传递 RequestBuilder 并返回）
fn apply_auth(
    mut req: reqwest::RequestBuilder,
    auth: &AuthConfig,
    subs: &[(&str, &str)],
) -> CoreResult<reqwest::RequestBuilder> {
    let token = auth
        .token
        .as_deref()
        .map(|t| substitute(t, subs))
        .unwrap_or_default();
    if token.is_empty() {
        return Err(CoreError::Auth("鉴权 token 为空".into()));
    }
    match auth.auth_type.as_str() {
        "header" => {
            req = req.header(&auth.name, token);
        }
        "bearer" => {
            req = req.header("Authorization", format!("Bearer {token}"));
        }
        "query" => {
            req = req.query(&[(auth.name.as_str(), token.as_str())]);
        }
        _ => {
            return Err(CoreError::Auth(format!(
                "未知鉴权类型: {}",
                auth.auth_type
            )));
        }
    }
    Ok(req)
}

/// 简化 JSONPath：仅支持点号路径，数组索引 `[i]`
pub fn extract_path(value: &Value, path: &str) -> CoreResult<Value> {
    let mut cur = value;
    for seg in path.split('.') {
        let seg = seg.trim();
        if seg.is_empty() {
            continue;
        }
        cur = cur
            .get(seg)
            .ok_or_else(|| CoreError::Parse(format!("路径 {path} 在 {seg} 处缺失")))?;
    }
    Ok(cur.clone())
}

/// 替换占位符 `{{name}}`
pub fn substitute(template: &str, subs: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (k, v) in subs {
        let from = format!("{{{{{k}}}}}");
        out = out.replace(&from, v);
    }
    out
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() > n {
        format!("{}...", &s[..n])
    } else {
        s.to_string()
    }
}
