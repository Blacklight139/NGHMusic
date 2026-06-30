//! FTP 协议客户端。
//!
//! 基于 suppaftp（纯 Rust FTP 客户端，无系统依赖）实现 FTP 访问。
//! suppaftp 为同步 API，本模块通过 `tokio::task::spawn_blocking` 包装阻塞调用，
//! 避免阻塞 tokio runtime。所有 FTP 错误统一映射为 [`CoreError::Protocol`](crate::error::CoreError::Protocol)。

use async_trait::async_trait;
use url::Url;

use std::net::ToSocketAddrs;
use std::time::Duration;

use crate::error::{CoreError, Result};
use crate::protocols::ProtocolClient;

/// FTP 连接超时（秒）。避免不可达地址长时间占用 tokio 阻塞线程（OS TCP SYN 重试默认约 75s+）。
const FTP_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// 将 `host:port` 解析为单个 `SocketAddr`，供 `FtpStream::connect_timeout` 使用。
fn resolve_socket_addr(host: &str, port: u16) -> Result<std::net::SocketAddr> {
    let addr_str = format!("{host}:{port}");
    addr_str
        .to_socket_addrs()
        .map_err(|e| CoreError::Protocol(format!("FTP 地址解析失败: {}", e)))?
        .next()
        .ok_or_else(|| CoreError::Protocol(format!("FTP 地址无法解析: {}", addr_str)))
}

/// FTP 协议客户端。
pub struct FtpClient {
    /// 服务器主机
    pub host: String,
    /// 端口
    pub port: u16,
    /// 登录用户名
    pub username: String,
    /// 登录密码
    pub password: String,
}

impl FtpClient {
    /// 构造 FTP 客户端。
    pub fn new(host: String, port: u16, username: String, password: String) -> Self {
        Self {
            host,
            port,
            username,
            password,
        }
    }

    /// 构造 `ftp://user:pass@host:port/path` URL，用户名/密码做 URL 编码。
    fn build_url(&self, path: &str) -> Result<String> {
        let mut url = Url::parse(&format!("ftp://{}:{}", self.host, self.port))
            .map_err(|e| CoreError::Protocol(format!("URL 构造失败: {}", e)))?;
        url.set_username(self.username.as_str())
            .map_err(|_| CoreError::Protocol("URL 构造失败: 用户名设置失败".into()))?;
        url.set_password(Some(self.password.as_str()))
            .map_err(|_| CoreError::Protocol("URL 构造失败: 密码设置失败".into()))?;
        let p = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{}", path)
        };
        url.set_path(&p);
        Ok(url.to_string())
    }
}

#[async_trait]
impl ProtocolClient for FtpClient {
    async fn list(&self, path: &str) -> Result<Vec<String>> {
        let host = self.host.clone();
        let port = self.port;
        let username = self.username.clone();
        let password = self.password.clone();
        let path = path.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<String>> {
            use suppaftp::FtpStream;
            let socket_addr = resolve_socket_addr(&host, port)?;
            let mut ftp = FtpStream::connect_timeout(socket_addr, FTP_CONNECT_TIMEOUT)
                .map_err(|e| CoreError::Protocol(format!("FTP 连接失败: {}", e)))?;

            // 用闭包承载业务逻辑，确保任意错误路径都在最后调用 quit() 释放连接
            let result = (|| -> Result<Vec<String>> {
                ftp.login(username.as_str(), password.as_str())
                    .map_err(|e| CoreError::Protocol(format!("FTP 登录失败: {}", e)))?;
                let opt_path = if path.is_empty() {
                    None
                } else {
                    Some(path.as_str())
                };
                let lines = ftp
                    .list(opt_path)
                    .map_err(|e| CoreError::Protocol(format!("FTP LIST 失败: {}", e)))?;
                let names = lines.iter().filter_map(|l| parse_list_line(l)).collect();
                Ok(names)
            })();
            let _ = ftp.quit();
            result
        })
        .await
        .map_err(|e| CoreError::Protocol(format!("FTP 任务执行失败: {}", e)))?
    }

    async fn read(&self, path: &str) -> Result<Vec<u8>> {
        let host = self.host.clone();
        let port = self.port;
        let username = self.username.clone();
        let password = self.password.clone();
        let path = path.to_string();
        tokio::task::spawn_blocking(move || -> Result<Vec<u8>> {
            use suppaftp::FtpStream;
            let socket_addr = resolve_socket_addr(&host, port)?;
            let mut ftp = FtpStream::connect_timeout(socket_addr, FTP_CONNECT_TIMEOUT)
                .map_err(|e| CoreError::Protocol(format!("FTP 连接失败: {}", e)))?;

            // 用闭包承载业务逻辑，确保任意错误路径都在最后调用 quit() 释放连接
            let result = (|| -> Result<Vec<u8>> {
                ftp.login(username.as_str(), password.as_str())
                    .map_err(|e| CoreError::Protocol(format!("FTP 登录失败: {}", e)))?;
                let cursor = ftp
                    .retr_as_buffer(path.as_str())
                    .map_err(|e| CoreError::Protocol(format!("FTP RETR 失败: {}", e)))?;
                Ok(cursor.into_inner())
            })();
            let _ = ftp.quit();
            result
        })
        .await
        .map_err(|e| CoreError::Protocol(format!("FTP 任务执行失败: {}", e)))?
    }

    async fn stream_url(&self, path: &str) -> Result<String> {
        self.build_url(path)
    }
}

/// 解析单行 Unix 风格 LIST 输出，返回文件名。
///
/// 格式形如 `drwxr-xr-x 2 user group 4096 Jan 1 12:00 name`，
/// 第一个字符为 `d` 表示目录。文件名取第 9 列之后（可含空格）。
/// 跳过 `total` 汇总行与空行。
fn parse_list_line(line: &str) -> Option<String> {
    let line = line.trim();
    if line.is_empty() || line.starts_with("total") {
        return None;
    }
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }
    Some(parts[8..].join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn stream_url_basic_format() {
        let client = FtpClient {
            host: "example.com".into(),
            port: 2121,
            username: "user".into(),
            password: "pass".into(),
        };
        let url = client.stream_url("/music/file.mp3").await.unwrap();
        assert!(url.starts_with("ftp://"));
        // 使用非默认端口 2121，url crate 会规范化掉默认端口 21
        assert!(url.contains("user:pass@example.com:2121"));
        assert!(url.ends_with("/music/file.mp3"));
    }

    #[tokio::test]
    async fn stream_url_encodes_password() {
        let client = FtpClient {
            host: "example.com".into(),
            port: 21,
            username: "user".into(),
            password: "p@ss w0rd".into(),
        };
        let url = client.stream_url("/file.mp3").await.unwrap();
        // @ 与空格应被 URL 编码
        assert!(url.contains("p%40ss%20w0rd"));
        assert!(!url.contains("p@ss w0rd"));
    }

    #[tokio::test]
    async fn stream_url_encodes_path_with_space() {
        let client = FtpClient {
            host: "example.com".into(),
            port: 2121,
            username: "u".into(),
            password: "p".into(),
        };
        let url = client.stream_url("/music/my file.mp3").await.unwrap();
        assert!(url.contains("my%20file.mp3"));
    }

    #[test]
    fn parse_list_line_unix() {
        let dir = "drwxr-xr-x 2 user group 4096 Jan 1 12:00 Music";
        assert_eq!(parse_list_line(dir).as_deref(), Some("Music"));
        let file = "-rw-r--r-- 1 user group 1024 Jan 1 12:00 track.mp3";
        assert_eq!(parse_list_line(file).as_deref(), Some("track.mp3"));
        assert_eq!(parse_list_line("total 8"), None);
        assert_eq!(parse_list_line(""), None);
        assert_eq!(parse_list_line("short line"), None);
    }
}