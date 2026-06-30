# 音源接口文档

本组接口封装 `music-core` 音源引擎能力，提供音源导入、校验、启用/禁用、聚合搜索、歌曲元数据/播放 URL/歌词获取、排行榜查询等 HTTP API。底层实现见 `core/src/sources/`（`schema.rs` 校验、`json.rs` 引擎、`community.rs` 适配、`local.rs` 本地源、`mod.rs::SourceManager`、`search.rs::Aggregator`）与 `core/src/cache.rs` 缓存层。

数据结构与 `core/src/models.rs` 一致：`Song`、`Album`、`Artist`、`Lyric`、`SearchResult`、`Leaderboard`、`SongOrigin`（内部标签 `type` 区分 `Online`/`Local`/`Nas`）。

---

## 接口分组：音源管理

### 导入音源

导入一份音源 JSON（标准格式或社区格式），自动适配为标准 `SoundSourceConfig` 并返回适配后配置与迁移报告。

- **方法**：`POST`
- **路径**：`/sources/import`
- **Content-Type**：`application/json`
- **请求 Body**（raw json）：音源原始 JSON，可为标准格式、社区格式 A（sources 数组）或社区格式 B（扁平 endpoint）。

```json
{
  "manifest": {
    "id": "demo-source",
    "name": "Demo Source",
    "version": "1.0.0",
    "author": "tester"
  },
  "endpoints": {
    "search": { "url": "https://music.example.com/api/search" },
    "metadata": { "url": "https://music.example.com/api/metadata" },
    "playUrl": { "url": "https://music.example.com/api/play" }
  },
  "fieldMapping": {
    "song": { "id": "id", "title": "name", "artists": "artists", "durationMs": "duration" },
    "album": { "id": "albumId", "name": "albumName" },
    "artist": { "id": "artistId", "name": "artistName" },
    "lyric": { "lines": [{ "timeMs": "time", "text": "content" }] },
    "searchResult": { "total": "total", "songs": "songs" }
  },
  "pagination": { "pageStart": 0, "pageSizeDefault": 20 },
  "timeoutMs": 10000
}
```

- **响应示例**（200 OK）：

```json
{
  "sourceFormat": "standard",
  "warnings": [],
  "config": {
    "manifest": {
      "id": "demo-source",
      "name": "Demo Source",
      "version": "1.0.0",
      "author": "tester"
    },
    "endpoints": {
      "search": { "url": "https://music.example.com/api/search" },
      "metadata": { "url": "https://music.example.com/api/metadata" },
      "playUrl": { "url": "https://music.example.com/api/play" }
    },
    "fieldMapping": {
      "song": { "id": "id", "title": "name", "artists": "artists", "durationMs": "duration" },
      "album": { "id": "albumId", "name": "albumName" },
      "artist": { "id": "artistId", "name": "artistName" },
      "lyric": { "lines": [{ "timeMs": "time", "text": "content" }] },
      "searchResult": { "total": "total", "songs": "songs" }
    },
    "pagination": { "pageStart": 0, "pageSizeDefault": 20 },
    "timeoutMs": 10000
  }
}
```

社区格式 A 导入示例响应（`sourceFormat=community-a`，含 warnings）：

```json
{
  "sourceFormat": "community-a",
  "warnings": [
    "社区格式 A 未提供 metadata/playUrl 接口，已使用占位 URL"
  ],
  "config": {
    "manifest": { "id": "music", "name": "社区音源", "version": "0.0.0", "author": "community" },
    "endpoints": {
      "search": { "url": "https://community.example.com/search" },
      "metadata": { "url": "https://example.com/unsupported" },
      "playUrl": { "url": "https://example.com/unsupported" }
    },
    "auth": { "type": "none" },
    "fieldMapping": {
      "song": { "id": "id", "title": "name" },
      "album": { "id": "albumId", "name": "albumName" },
      "artist": { "id": "artistId", "name": "artistName" },
      "lyric": { "lines": [{ "timeMs": "time", "text": "content" }] }
    },
    "pagination": { "pageStart": 0, "pageSizeDefault": 20 },
    "timeoutMs": 10000
  }
}
```

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 400 | `Schema` | 无法识别的社区音源格式 / 社区格式缺少 name 或 url |
| 422 | `Schema` | 适配后配置未通过严格 Schema 校验（含具体路径） |

---

### 校验音源

校验音源 JSON 是否符合标准 Schema，不加载、不持久化。先做 `validate_strict`，再做 `validate_config` 语义校验。

- **方法**：`POST`
- **路径**：`/sources/validate`
- **Content-Type**：`application/json`
- **请求 Body**（raw json）：音源 JSON（同导入）。

- **响应示例**（200 OK，校验通过）：

```json
{
  "valid": true,
  "errors": []
}
```

- **响应示例**（422 Unprocessable Entity，校验失败）：

```json
{
  "valid": false,
  "errors": [
    "路径 manifest: 缺少必填属性 id",
    "auth.type 为 \"header\" 时必须提供非空 tokenParam"
  ]
}
```

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 422 | `Schema` | 配置不符合 Schema 或语义校验失败，`errors` 列出全部问题 |

---

### 列出已加载音源

列出当前 `SourceManager` 中所有音源及其状态。

- **方法**：`GET`
- **路径**：`/sources`

- **响应示例**（200 OK）：

```json
{
  "sources": [
    {
      "id": "local",
      "name": "本地音乐",
      "enabled": true,
      "priority": 100
    },
    {
      "id": "demo-source",
      "name": "Demo Source",
      "enabled": true,
      "priority": 10
    },
    {
      "id": "authed-source",
      "name": "需鉴权音源",
      "enabled": false,
      "priority": 5
    }
  ]
}
```

> 返回字段对应 `SourceManager::list()`：`(id, name, enabled, priority)`。`ordered_sources()` 按 priority 降序仅含启用项，参与聚合搜索。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 500 | `Internal` | SourceManager 锁获取失败等内部错误 |

---

### 启用音源

启用指定 id 的音源，使其重新参与聚合搜索。不存在时为空操作（仍返回 200）。

- **方法**：`POST`
- **路径**：`/sources/{id}/enable`
- **路径参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 音源 id（如 `demo-source`） |

- **响应示例**（200 OK）：

```json
{
  "id": "authed-source",
  "enabled": true
}
```

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 404 | `NotFound` | 音源 id 不存在（视实现可选返回 200 空操作） |

---

### 禁用音源

禁用指定 id 的音源，使其不参与聚合搜索但保留配置。不存在时为空操作。

- **方法**：`POST`
- **路径**：`/sources/{id}/disable`
- **路径参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 音源 id |

- **响应示例**（200 OK）：

```json
{
  "id": "demo-source",
  "enabled": false
}
```

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 404 | `NotFound` | 音源 id 不存在（视实现可选返回 200 空操作） |

---

### 删除音源

按 id 移除音源（`remove_source`），不存在则无操作。本地音源（`local`）通常不允许删除。

- **方法**：`DELETE`
- **路径**：`/sources/{id}`
- **路径参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 音源 id |

- **响应示例**（200 OK）：

```json
{
  "id": "demo-source",
  "deleted": true
}
```

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 404 | `NotFound` | 音源 id 不存在 |
| 403 | `Source` | 试图删除受保护音源（如内置 `local`） |

---

## 接口分组：搜索与歌曲

### 聚合搜索

跨所有启用音源聚合搜索。底层 `Aggregator`（`core/src/search.rs`）顺序查询各音源并合并结果，单个音源失败记录 `warn` 并跳过，不影响整体结果。

- **方法**：`GET`
- **路径**：`/sources/search`
- **Query 参数**：

| 参数 | 类型 | 必填 | 默认 | 说明 |
| --- | --- | --- | --- | --- |
| `keyword` | string | 是 | — | 搜索关键词 |
| `page` | u32 | 否 | `1` | 页码（从 `pagination.pageStart` 起算） |
| `page_size` | u32 | 否 | `20` | 每页数量（默认 `pagination.pageSizeDefault`） |

- **请求示例**：

```
GET /sources/search?keyword=晴天&page=1&page_size=20
```

- **响应示例**（200 OK）：

```json
{
  "keyword": "晴天",
  "songs": [
    {
      "id": "1001",
      "source_id": "demo-source",
      "title": "晴天",
      "artists": ["周杰伦"],
      "album": "叶惠美",
      "cover_url": "https://music.example.com/covers/1001.jpg",
      "duration_ms": 240000,
      "lyric_url": null,
      "play_url": "https://music.example.com/stream/1001.mp3",
      "local_path": null,
      "origin": {
        "type": "Online",
        "source_id": "demo-source",
        "play_url": "https://music.example.com/stream/1001.mp3"
      }
    },
    {
      "id": "<uuid>",
      "source_id": "local",
      "title": "晴天.flac",
      "artists": ["Unknown Artist"],
      "album": null,
      "cover_url": null,
      "duration_ms": null,
      "lyric_url": null,
      "play_url": null,
      "local_path": "/music/晴天.flac",
      "origin": {
        "type": "Local",
        "path": "/music/晴天.flac"
      }
    }
  ],
  "albums": [
    {
      "id": "a1",
      "source_id": "demo-source",
      "name": "叶惠美",
      "artists": ["周杰伦"],
      "cover_url": "https://music.example.com/covers/album-a1.jpg",
      "song_ids": ["1001", "1002"]
    }
  ],
  "artists": [
    {
      "id": "ar1",
      "source_id": "demo-source",
      "name": "周杰伦",
      "avatar_url": "https://music.example.com/artists/ar1.jpg",
      "song_ids": ["1001"]
    }
  ],
  "total": 128,
  "page": 1,
  "page_size": 20
}
```

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 400 | `Schema` | keyword 为空或参数非法 |
| 500 | `Internal` | 全部音源均失败且无任何结果（视实现可选返回空结果而非 500） |

> 单个音源失败不会导致整体 500，会被 `log::warn` 记录并跳过。

---

### 获取歌曲元数据

按歌曲 id 获取单首歌曲的完整元数据（`get_metadata`）。用于补全搜索结果中缺失的封面、时长、专辑等字段。

- **方法**：`GET`
- **路径**：`/sources/{id}/songs/{songId}/metadata`
- **路径参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 音源 id |
| `songId` | string | 是 | 歌曲 id（音源内唯一） |

- **响应示例**（200 OK）：

```json
{
  "id": "1001",
  "source_id": "demo-source",
  "title": "晴天",
  "artists": ["周杰伦"],
  "album": "叶惠美",
  "cover_url": "https://music.example.com/covers/1001.jpg",
  "duration_ms": 240000,
  "lyric_url": "https://music.example.com/lyric/1001.lrc",
  "play_url": "https://music.example.com/stream/1001.mp3",
  "local_path": null,
  "origin": {
    "type": "Online",
    "source_id": "demo-source",
    "play_url": "https://music.example.com/stream/1001.mp3"
  }
}
```

本地歌曲元数据示例（`origin.type = "Local"`）：

```json
{
  "id": "<uuid>",
  "source_id": "local",
  "title": "My Song",
  "artists": ["Unknown Artist"],
  "album": null,
  "cover_url": null,
  "duration_ms": null,
  "lyric_url": null,
  "play_url": null,
  "local_path": "/music/My Song.mp3",
  "origin": {
    "type": "Local",
    "path": "/music/My Song.mp3"
  }
}
```

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 404 | `NotFound` | 歌曲 id 不存在（本地音源返回 `本地歌曲不存在: <song_id>`） |
| 422 | `Source` | 无法从元数据响应映射歌曲（`fieldMapping.song.id` / `title` 字段名不匹配） |
| 502 | `Http` | 上游音源接口返回非 2xx 或网络错误 |
| 504 | `Http` | 请求超时（超过 `timeoutMs`） |

---

### 获取播放 URL

按歌曲 id 获取可播放 URL（`get_play_url`）。优先用 `song.playUrl` 字段映射提取，缺失则退化为整个响应字符串。命中缓存时返回本地缓存文件路径。

- **方法**：`GET`
- **路径**：`/sources/{id}/songs/{songId}/play-url`
- **路径参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 音源 id |
| `songId` | string | 是 | 歌曲 id |

- **响应示例**（200 OK，在线 URL）：

```json
{
  "url": "https://music.example.com/stream/1001.mp3",
  "cached": false,
  "play_url": "https://music.example.com/stream/1001.mp3"
}
```

- **响应示例**（200 OK，缓存命中，返回本地文件路径）：

```json
{
  "url": "/var/cache/music-player/1001.cache",
  "cached": true,
  "play_url": "/var/cache/music-player/1001.cache"
}
```

> 缓存层（`CacheManager`）以 `song_id` 为 key，命中时返回本地 `.cache` 文件路径避免重复下载；未命中时调用 fetcher 下载并写入 `cache_dir/<sanitized_key>.cache`，超容量按 LRU 淘汰。`max_entries` 固定 1000，`max_bytes` 由配置决定。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 404 | `NotFound` | 歌曲 id 不存在 |
| 422 | `Source` | 无法从响应中提取播放 URL（未配置 `playUrl` 映射且响应非字符串） |
| 502 | `Http` | 上游接口错误 |
| 504 | `Http` | 请求超时 |

---

### 获取歌词

按歌曲 id 获取歌词（`get_lyric`）。引擎接受响应为数组（行列表）或对象中含 `lines` 字段。

- **方法**：`GET`
- **路径**：`/sources/{id}/songs/{songId}/lyric`
- **路径参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 音源 id |
| `songId` | string | 是 | 歌曲 id |

- **响应示例**（200 OK）：

```json
{
  "lines": [
    { "time_ms": 12340, "text": "第一行歌词" },
    { "time_ms": 15670, "text": "第二行歌词" },
    { "time_ms": 20000, "text": "第三行歌词" },
    { "time_ms": null, "text": "无时间戳的纯文本行" }
  ],
  "translation": null
}
```

带翻译示例：

```json
{
  "lines": [
    { "time_ms": 12340, "text": "第一行" },
    { "time_ms": 15670, "text": "第二行" }
  ],
  "translation": [
    { "time_ms": 12340, "text": "First line" },
    { "time_ms": 15670, "text": "Second line" }
  ]
}
```

> `time_ms` 为毫秒；`null` 表示无时间戳的纯文本歌词行（排在所有时间戳行之后）。`LyricLine` 定义见 `core/src/models.rs`。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 404 | `NotFound` | 音源未配置歌词接口（`音源 <id> 未配置歌词接口`）或歌曲不存在 |
| 422 | `Source` | 歌词响应格式错误：应为数组或含 lines 字段 |
| 502 | `Http` | 上游接口错误 |

---

### 获取排行榜

获取音源提供的排行榜列表（`get_leaderboards`）。未配置 `leaderboards` endpoint 时返回空数组。

- **方法**：`GET`
- **路径**：`/sources/{id}/leaderboards`
- **路径参数**：

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 音源 id |

- **响应示例**（200 OK）：

```json
[
  {
    "id": "hot-100",
    "source_id": "demo-source",
    "name": "热歌榜",
    "cover_url": "https://music.example.com/leaderboards/hot-100.jpg",
    "songs": [
      {
        "id": "2001",
        "source_id": "demo-source",
        "title": "热门歌曲 A",
        "artists": ["歌手A"],
        "album": "专辑A",
        "cover_url": "https://music.example.com/covers/2001.jpg",
        "duration_ms": 210000,
        "lyric_url": null,
        "play_url": null,
        "local_path": null,
        "origin": {
          "type": "Online",
          "source_id": "demo-source",
          "play_url": ""
        }
      }
    ]
  },
  {
    "id": "new-songs",
    "source_id": "demo-source",
    "name": "新歌榜",
    "cover_url": null,
    "songs": []
  }
]
```

> 排行榜接口期望返回数组，每个元素含 `id`、`name`、`coverUrl`（或 `cover_url`，可选）、`songs`（可选，按 `fieldMapping.song` 映射）。`id` 为空的条目会被跳过。

- **错误码**：

| HTTP | 错误类型 | 说明 |
| --- | --- | --- |
| 502 | `Http` | 排行榜接口请求失败 |
| 504 | `Http` | 请求超时 |

> 本地音源（`local`）调用此接口固定返回空数组（`本地音源无排行榜`），不会报错。

---

## 通用错误响应格式

所有接口错误返回统一结构：

```json
{
  "error": {
    "kind": "Schema",
    "message": "音源配置校验失败: 路径 manifest: 缺少必填属性 id"
  }
}
```

`kind` 取值对应 `CoreError` 变体（`core/src/error.rs`）：`Io`、`Json`、`Http`、`Source`、`Schema`、`NotFound`、`Cache`、`Protocol`、`Feiniu`、`Ffi`。
