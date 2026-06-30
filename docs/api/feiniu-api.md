# 飞牛接口文档

本组接口封装飞牛 NAS HTTP API，提供登录、目录列表、流式播放 URL 生成与健康检查能力。底层实现见 `core/src/feiniu.rs::FeiniuClient`，所有 HTTP/serde 错误统一映射为 `CoreError::Feiniu`，非 2xx 响应的错误信息含状态码与响应体片段（前 200 字符）。

飞牛 NAS 客户端原语：

- `base_url`：飞牛服务地址，如 `https://nas.example.com`（末尾多余的 `/` 会自动去除）。
- 登录后保存 `token`，后续请求携带 `Authorization: Bearer <token>`。
- 文件条目结构 `NasFile`：`name`、`is_dir`（兼容 `isDir`）、`size`（默认 0）、`modified`（可选字符串）。

---

## 接口分组：飞牛 NAS

### 登录

向飞牛服务登录并获取访问 token，后续接口调用需依赖该 token。

- **方法**：`POST`
- **路径**：`/feiniu/login`
- **Content-Type**：`application/json`
- **请求 Body**：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `base_url` | string | 是 | 飞牛服务地址，如 `https://nas.example.com` |
| `username` | string | 是 | 登录用户名 |
| `password` | string | 是 | 登录密码 |

```json
{
  "base_url": "https://nas.example.com",
  "username": "admin",
  "password": "p@ssw0rd"
}
```

- **响应示例**（200 OK）：

```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "base_url": "https://nas.example.com"
}
```

> 底层调用飞牛 `POST {base_url}/api/v1/auth/login`，body 为 `{username, password}`。响应兼容 `token` / `access_token`，以及 `data` 嵌套（`data.token` / `data.access_token`）。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 401 | `Feiniu` | 用户名或密码错误（飞牛返回 401） |
| 422 | `Feiniu` | 登录响应缺少 token 字段（`登录响应缺少 token 字段`） |
| 502 | `Feiniu` | 飞牛服务不可达 / 登录请求失败（`登录请求失败: ...`） |
| 502 | `Feiniu` | 登录失败：状态码 + 响应体片段（`登录失败: 状态 403 ...`） |
| 504 | `Feiniu` | 解析登录响应失败（响应非 JSON） |

**重试说明**：网络抖动导致的 502 建议指数退避重试（如 1s / 2s / 4s，最多 3 次）；401 不可重试，需提示用户检查凭据。

---

### 列目录

列出飞牛 NAS 指定路径下的文件与子目录。

- **方法**：`GET`
- **路径**：`/feiniu/files`
- **Query 参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `path` | string | 是 | 要列出的目录路径，如 `/music` 或 `/music/周杰伦` |

- **请求示例**：

```
GET /feiniu/files?path=/music
```

- **响应示例**（200 OK）：

```json
{
  "path": "/music",
  "files": [
    {
      "name": "周杰伦",
      "is_dir": true,
      "size": 0,
      "modified": "2024-01-15T08:30:00Z"
    },
    {
      "name": "晴天.mp3",
      "is_dir": false,
      "size": 5242880,
      "modified": "2024-01-15T08:35:12Z"
    },
    {
      "name": "七里香.flac",
      "is_dir": false,
      "size": 31457280,
      "modified": "2024-01-16T10:00:00Z"
    }
  ]
}
```

> 底层调用飞牛 `GET {base_url}/api/v1/files?path=...`，携带 `Authorization: Bearer <token>`。响应可为 `{files: [...]}` 或裸数组，逐项解析为 `NasFile`，失败项跳过以保证健壮性。`is_dir` 字段兼容 `isDir` 命名；`size` / `modified` 缺失时使用默认值。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 401 | `Feiniu` | 未登录或 token 失效（`未登录`） |
| 404 | `Feiniu` | 路径不存在（飞牛返回 404，错误信息 `list_files 失败: 状态 404 ...`） |
| 422 | `Feiniu` | 文件列表响应非数组（`文件列表响应非数组`） |
| 502 | `Feiniu` | 飞牛服务不可达 / list_files 请求失败 |
| 502 | `Feiniu` | list_files 失败：状态码 + 响应体片段 |
| 504 | `Feiniu` | 解析文件列表响应失败（响应非 JSON） |

**重试说明**：401 需重新登录后重试；404 路径不存在不可重试，需修正 `path`；502/504 建议指数退避重试。

---

### 获取播放流 URL

生成飞牛 NAS 文件的可流式播放 URL，供播放器直接拉流。

- **方法**：`GET`
- **路径**：`/feiniu/stream`
- **Query 参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `path` | string | 是 | 文件路径，如 `/music/周杰伦/晴天.mp3` |

- **请求示例**：

```
GET /feiniu/stream?path=/music/周杰伦/晴天.mp3
```

- **响应示例**（200 OK）：

```json
{
  "url": "https://nas.example.com/api/v1/files/stream?path=%2Fmusic%2F%E5%91%A8%E6%9D%B0%E4%BC%A6%2F%E6%99%B4%E5%A4%A9.mp3&token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

> 底层拼接 `{base_url}/api/v1/files/stream?path=<encoded>&token=<token>`。`path` 自动做 URL 编码（含空格、中文、斜杠等特殊字符）；已登录时附带 `token` 作为 query 参数。该接口不实际请求飞牛，仅生成 URL，故不会返回飞牛侧错误，但后续播放器拉流时可能遇到鉴权或路径错误。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 400 | `Feiniu` | URL 构造失败（`base_url` 非法，`URL 构造失败: ...`） |

> 拉流阶段（播放器访问该 URL）可能返回：401 token 失效、404 文件不存在、403 无权限。建议播放器对 401 自动重新登录刷新 token 后重试一次。

---

### 健康检查

探测飞牛服务是否可达，用于异常检测与重试判断。

- **方法**：`GET`
- **路径**：`/feiniu/health`

- **请求示例**：

```
GET /feiniu/health
```

- **响应示例**（200 OK）：

```json
{
  "healthy": true,
  "base_url": "https://nas.example.com"
}
```

> 底层调用飞牛 `GET {base_url}/api/v1/health`，2xx 返回 `Ok`，否则返回 `CoreError::Feiniu`。无需携带 token。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 502 | `Feiniu` | 飞牛服务不可达 / 健康检查请求失败（`健康检查请求失败: ...`） |
| 502 | `Feiniu` | 健康检查失败：状态码 + 响应体片段（`健康检查失败: 状态 503 ...`） |

**重试说明**：建议作为前置探活，失败时按指数退避重试 3 次；持续失败应标记飞牛音源不可用并提示用户检查网络或飞牛服务状态。

---

## 数据结构

### NasFile

| 字段 | 类型 | 说明 |
| --- | --- | --- |
| `name` | string | 文件/目录名称 |
| `is_dir` | bool | 是否为目录（响应字段名兼容 `isDir`） |
| `size` | u64 | 字节大小，缺失默认 0 |
| `modified` | string \| null | 修改时间（原始字符串，格式由服务端决定） |

```json
{
  "name": "晴天.mp3",
  "is_dir": false,
  "size": 5242880,
  "modified": "2024-01-15T08:35:12Z"
}
```

---

## 通用错误响应格式

```json
{
  "error": {
    "kind": "Feiniu",
    "message": "list_files 失败: 状态 404 Not Found {\"error\":\"path not found\"}"
  }
}
```

`kind` 固定为 `Feiniu`（飞牛相关错误均映射为 `CoreError::Feiniu`），`message` 含状态码与响应体片段（前 200 字符）便于排查。
