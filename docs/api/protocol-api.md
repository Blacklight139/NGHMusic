# 协议接口文档

本组接口提供统一协议源管理，封装 SMB、WebDAV、FTP、DLNA、NFS 等远程文件协议，通过 `ProtocolClient` trait（`core/src/protocols/mod.rs`）屏蔽具体协议差异，上层统一访问 `list` / `read` / `stream_url` 三类能力。

各协议实现状态：

| 协议 | 实现状态 | 客户端结构 |
| --- | --- | --- |
| WebDAV | 已实现（基于 HTTP PROPFIND/GET） | `WebDavClient { base_url, client, auth }` |
| FTP | 已实现（基于 suppaftp，纯 Rust 无系统依赖） | `FtpClient { host, port, username, password }` |
| SMB | 占位实现（需启用 `pavao` feature，依赖系统 libsmbclient） | `SmbClient { host, share, username, password }` |
| DLNA | 占位实现（需集成 dlna-rs/UPnP 库） | `DlnaClient { control_url }` |
| NFS | 占位实现（需系统挂载或 ONC RPC 客户端库） | `NfsClient { server, export }` |

> SMB / DLNA / NFS 的 `list` / `read` / `stream_url` 当前调用会返回 `CoreError::Protocol` 占位错误，需启用对应 feature 或系统库后方可使用。WebDAV / FTP 为完整可用实现。

---

## 接口分组：协议源管理

### 添加协议源

添加一个远程协议源。`protocol` 字段决定其余配置字段语义（见下方各协议配置差异表）。

- **方法**：`POST`
- **路径**：`/protocols/sources`
- **Content-Type**：`application/json`
- **请求 Body**：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `protocol` | `"smb"` \| `"webdav"` \| `"ftp"` \| `"dlna"` \| `"nfs"` | 是 | 协议类型 |
| `host` | string | 视协议 | 服务器地址（SMB / FTP / NFS） |
| `port` | u16 | 视协议 | 端口（FTP 默认 21；WebDAV 在 base_url 中体现） |
| `auth` | object | 否 | 鉴权信息（见各协议差异） |
| `root` | string | 否 | 根路径 / 导出路径 |

WebDAV 示例：

```json
{
  "protocol": "webdav",
  "auth": {
    "base_url": "https://dav.example.com/dav",
    "username": "user",
    "password": "pass"
  },
  "root": "/music"
}
```

FTP 示例：

```json
{
  "protocol": "ftp",
  "host": "ftp.example.com",
  "port": 2121,
  "auth": { "username": "user", "password": "pass" },
  "root": "/music"
}
```

SMB 示例（占位）：

```json
{
  "protocol": "smb",
  "host": "smb.example.com",
  "auth": { "username": "user", "password": "pass", "share": "music" },
  "root": "/"
}
```

NFS 示例（占位）：

```json
{
  "protocol": "nfs",
  "host": "nas.local",
  "root": "/export/music"
}
```

DLNA 示例（占位）：

```json
{
  "protocol": "dlna",
  "auth": { "control_url": "http://dlna.local:8200/MediaServer/Control" },
  "root": "/"
}
```

- **响应示例**（201 Created）：

```json
{
  "id": "proto-1",
  "protocol": "webdav",
  "root": "/music",
  "enabled": true
}
```

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 400 | `Protocol` | `protocol` 取值非法 / 必填字段缺失（如 FTP 缺少 `host`） |
| 422 | `Protocol` | 协议配置语义错误（如 WebDAV `base_url` 非 URL） |
| 501 | `Protocol` | SMB / DLNA / NFS 当前为占位实现，无法实际连接（创建后调用 `list`/`read`/`stream` 会返回占位错误） |

---

### 列出协议源

列出已加载的协议源。

- **方法**：`GET`
- **路径**：`/protocols/sources`

- **响应示例**（200 OK）：

```json
{
  "sources": [
    {
      "id": "proto-1",
      "protocol": "webdav",
      "root": "/music",
      "enabled": true
    },
    {
      "id": "proto-2",
      "protocol": "ftp",
      "root": "/music",
      "enabled": true
    },
    {
      "id": "proto-3",
      "protocol": "smb",
      "root": "/",
      "enabled": false,
      "placeholder": true
    }
  ]
}
```

> `placeholder: true` 表示该协议为占位实现（SMB / DLNA / NFS），实际文件操作不可用。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 500 | `Internal` | 内部错误 |

---

### 删除协议源

按 id 移除协议源，断开对应连接。

- **方法**：`DELETE`
- **路径**：`/protocols/sources/{id}`
- **路径参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 协议源 id |

- **响应示例**（200 OK）：

```json
{
  "id": "proto-1",
  "deleted": true
}
```

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 404 | `NotFound` | 协议源 id 不存在 |

---

## 接口分组：协议文件操作

### 浏览目录

列出协议源下指定路径的条目名称。

- **方法**：`GET`
- **路径**：`/protocols/sources/{id}/list`
- **路径参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 协议源 id |

- **Query 参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `path` | string | 是 | 相对路径，如 `/周杰伦` 或 `/` |

- **请求示例**：

```
GET /protocols/sources/proto-1/list?path=/周杰伦
```

- **响应示例**（200 OK，WebDAV / FTP 返回条目名称列表）：

```json
{
  "path": "/周杰伦",
  "entries": [
    "晴天.mp3",
    "七里香.flac",
    "叶惠美"
  ]
}
```

> 底层调用 `ProtocolClient::list(path)`。WebDAV 通过 PROPFIND（`Depth: 1`）解析 multistatus XML 提取 `href`；FTP 通过 `LIST` 命令解析 Unix 风格行（取第 9 列之后作为文件名，跳过 `total` 汇总行）。FTP 基于 `suppaftp` 同步 API，通过 `tokio::task::spawn_blocking` 包装避免阻塞 runtime。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 404 | `NotFound` | 协议源 id 不存在 |
| 401 | `Protocol` | 鉴权失败（WebDAV Basic auth 错误 / FTP 登录失败） |
| 404 | `Protocol` | 路径不存在（WebDAV PROPFIND 失败 / FTP LIST 失败） |
| 501 | `Protocol` | SMB / DLNA / NFS 占位实现（`SMB 协议需启用 pavao feature...，当前为占位实现` 等） |
| 502 | `Protocol` | WebDAV PROPFIND 请求失败 / FTP 连接失败 / FTP 任务执行失败 |
| 504 | `Protocol` | 读取 PROPFIND 响应失败 / 超时 |

---

### 读取文件字节

读取协议源下指定路径文件的内容为字节。

- **方法**：`GET`
- **路径**：`/protocols/sources/{id}/read`
- **路径参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 协议源 id |

- **Query 参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `path` | string | 是 | 文件路径 |

- **请求示例**：

```
GET /protocols/sources/proto-2/read?path=/music/晴天.mp3
```

- **响应示例**（200 OK，`application/octet-stream`）：

返回文件原始字节流。

> 底层调用 `ProtocolClient::read(path)`。WebDAV 通过 GET 读取文件返回 `bytes`；FTP 通过 `RETR` 命令读取为 `Cursor` 后取 `into_inner` 字节。注意大文件读取会占用内存，建议优先使用 `stream` 接口拉流。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 404 | `NotFound` | 协议源 id 不存在 |
| 404 | `Protocol` | 文件路径不存在 |
| 401 | `Protocol` | 鉴权失败 |
| 501 | `Protocol` | SMB / DLNA / NFS 占位实现 |
| 502 | `Protocol` | WebDAV GET 请求失败 / FTP RETR 失败 / FTP 任务执行失败 |
| 504 | `Protocol` | 读取 WebDAV 文件失败 / 超时 |

---

### 获取流式播放 URL

生成协议源下指定文件的可流式播放 URL，供播放器直接拉流。

- **方法**：`GET`
- **路径**：`/protocols/sources/{id}/stream`
- **路径参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 协议源 id |

- **Query 参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `path` | string | 是 | 文件路径 |

- **请求示例**：

```
GET /protocols/sources/proto-1/stream?path=/music/晴天.mp3
```

- **响应示例**（200 OK，WebDAV 直链）：

```json
{
  "url": "https://dav.example.com/dav/music/晴天.mp3"
}
```

- **响应示例**（200 OK，FTP URL，含编码后的凭据与路径）：

```json
{
  "url": "ftp://user:p%40ss@example.com:2121/music/%E6%99%B4%E5%A4%A9.mp3"
}
```

> 底层调用 `ProtocolClient::stream_url(path)`。
> - WebDAV：直接返回 `base_url` 与 `path` 拼接的 URL（直链，可由支持 HTTP 的播放器拉流；若配置了 Basic auth，播放器需自带凭据）。
> - FTP：构造 `ftp://user:pass@host:port/path` URL，用户名/密码/路径均做 URL 编码（如 `@` → `%40`、空格 → `%20`、中文按百分号编码）。
> - SMB / DLNA / NFS：占位实现，返回 `CoreError::Protocol` 占位错误（SMB 无标准 URL 方案，建议先 `read` 落盘或转 HTTP 中转；DLNA / NFS 同理）。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 404 | `NotFound` | 协议源 id 不存在 |
| 400 | `Protocol` | URL 构造失败（FTP，`URL 构造失败: ...`） |
| 501 | `Protocol` | SMB / DLNA / NFS 占位实现 |

---

## 各协议配置字段差异表

### SMB（占位）

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `host` | string | 是 | 服务器地址 |
| `auth.share` | string | 是 | 共享名 |
| `auth.username` | string | 否 | 登录用户名 |
| `auth.password` | string | 否 | 登录密码 |
| `root` | string | 否 | 起始路径 |

> **启用方式**：在 `Cargo.toml` 启用 `pavao` feature（依赖系统级 libsmbclient 共享库）。`stream_url` 无标准 URL 方案，建议先 `read` 落盘或转 HTTP 中转。

### WebDAV（已实现）

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `auth.base_url` | string | 是 | 服务器根 URL，如 `https://server/dav` |
| `auth.username` | string | 否 | Basic auth 用户名（与 password 同时提供才启用） |
| `auth.password` | string | 否 | Basic auth 密码 |
| `root` | string | 否 | 起始路径 |

> 基于 HTTP 实现：PROPFIND（`Depth: 1` + allprop 请求体）列目录，GET 读取文件，`stream_url` 返回 `base_url/path` 拼接的直链。multistatus XML 通过字节扫描解析 `href`（兼容 `<d:href>`、`<D:href>`、`<href>` 命名空间）。

### FTP（已实现）

| 字段 | 类型 | 必填 | 默认 | 说明 |
| --- | --- | --- | --- | --- |
| `host` | string | 是 | — | 服务器主机 |
| `port` | u16 | 是 | — | 端口（如 21、2121） |
| `auth.username` | string | 是 | — | 登录用户名 |
| `auth.password` | string | 是 | — | 登录密码 |
| `root` | string | 否 | — | 起始路径 |

> 基于 `suppaftp`（纯 Rust FTP 客户端，无系统依赖）。同步 API 通过 `tokio::task::spawn_blocking` 包装，避免阻塞 runtime。`stream_url` 构造 `ftp://user:pass@host:port/path`，凭据与路径自动 URL 编码。`list` 解析 Unix 风格 LIST 输出（`drwxr-xr-x ... name`，取第 9 列之后）。

### DLNA（占位）

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `auth.control_url` | string | 是 | 设备控制 URL（如 `http://dlna.local:8200/MediaServer/Control`） |
| `root` | string | 否 | 起始路径 |

> **启用方式**：集成 `dlna-rs` 或类似 UPnP/DLNA 库，完成 SSDP 设备发现 → 获取 ContentDirectory 服务 → Browse 动作 → 解析 DIDL-Lite。协议栈复杂（异步 SSDP 多播 + SOAP HTTP），生态库选型与稳定性需谨慎评估。

### NFS（占位）

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `host` | string | 是 | 服务器地址 |
| `root` | string | 是 | NFS 导出路径（export path），如 `/export/music` |

> **启用方式**：
> 1. 通过系统挂载 NFS 导出后按本地文件访问（最稳妥，但需 root/特权）；
> 2. 或引入 NFSv3 ONC RPC 客户端库（Rust 生态不成熟，需谨慎评估）。
> Rust 生态缺少成熟的纯 Rust NFS 客户端库，当前为骨架实现。

---

## 通用错误响应格式

```json
{
  "error": {
    "kind": "Protocol",
    "message": "WebDAV PROPFIND 失败: 状态 404 Not Found <body>"
  }
}
```

`kind` 固定为 `Protocol`（协议相关错误均映射为 `CoreError::Protocol`），`message` 含状态码与响应体片段（前 200 字符）便于排查。占位实现的错误信息包含「占位」与「需启用」关键词。

### 占位实现错误信息

| 协议 | 错误信息 |
| --- | --- |
| SMB | `SMB 协议需启用 pavao feature（依赖 libsmbclient 系统库），当前为占位实现` |
| DLNA | `DLNA 协议需集成 dlna-rs/UPnP 库（SSDP 发现 + SOAP 控制），当前为占位实现` |
| NFS | `NFS 协议需系统挂载或 ONC RPC 客户端库（Rust 生态较弱），当前为占位实现` |

---

## 重试与降级说明

- **WebDAV / FTP 鉴权失败（401）**：提示用户检查凭据，不可自动重试。
- **路径不存在（404）**：不可重试，需修正 `path`。
- **网络错误 / 服务不可达（502 / 504）**：建议指数退避重试（1s / 2s / 4s，最多 3 次）。
- **占位协议（501）**：不可重试，需提示用户该协议当前未启用，引导启用对应 feature 或使用 WebDAV / FTP 替代。
- **SMB 拉流**：由于 SMB 无标准 URL 方案，建议对 SMB 源先调用 `read` 落盘到本地缓存，再以本地文件方式播放；或部署一个 HTTP 中转网关将 SMB 路径暴露为 HTTP 直链后作为 WebDAV 源接入。
