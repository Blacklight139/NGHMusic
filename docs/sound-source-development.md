# 音源开发文档

本面向开发者的文档说明如何为跨平台音乐播放器编写自定义音源。音源以一份 JSON 配置描述，由 Rust 核心 `music-core` 的音源引擎（`core/src/sources/json.rs` 中的 `JsonSource`）解释执行，自动完成 HTTP 请求、字段映射、分页与鉴权，将异构在线音乐 API 适配为统一的 `Song`/`Album`/`Artist`/`Lyric`/`SearchResult` 数据结构。

> 配置对应的 JSON Schema 文件位于 `schemas/sound-source.schema.json`，Rust 端结构定义见 `core/src/sources/schema.rs`。两者字段保持一致，Schema 在编译期内嵌进二进制，可离线校验。

---

## 1. 概述

**音源（Sound Source）** 是一份 JSON 配置，描述如何通过 API 获取主流音乐软件（网易云、QQ音乐等）的元数据、搜索结果、播放 URL 与歌词。

核心能力：

- 通过 `manifest` 声明音源身份（id / name / version / author）。
- 通过 `endpoints` 声明搜索、元数据、播放 URL、歌词、排行榜接口的 URL、HTTP 方法、参数名与请求头。
- 通过 `auth` 声明鉴权方式（无鉴权 / query 参数 / header）。
- 通过 `fieldMapping` 把音源返回的原始 JSON 字段映射为标准 `Song`/`Album`/`Artist`/`Lyric` 字段，支持简单字段名与点路径。
- 通过 `pagination` 声明分页起始页与默认每页数量。
- 通过 `timeoutMs` 声明请求超时。

音源加载后会注册到 `SourceManager`（`core/src/sources/mod.rs`），按 `priority` 排序参与聚合搜索。本地音源（`LocalSource`）作为内置源自动接入，无需编写 JSON。

### 标准数据结构

音源最终产出的数据结构定义在 `core/src/models.rs`：

| 结构 | 关键字段 |
| --- | --- |
| `Song` | `id`、`source_id`、`title`、`artists: Vec<String>`、`album: Option<String>`、`cover_url`、`duration_ms: Option<u64>`（毫秒）、`lyric_url`、`play_url`、`local_path`、`origin` |
| `SongOrigin` | 内部标签 `type` 区分：`Online { source_id, play_url }` / `Local { path }` / `Nas { protocol, url }` |
| `Album` | `id`、`source_id`、`name`、`artists`、`cover_url`、`song_ids: Vec<String>` |
| `Artist` | `id`、`source_id`、`name`、`avatar_url`、`song_ids: Vec<String>` |
| `Lyric` | `lines: Vec<LyricLine>`、`translation: Option<Vec<LyricLine>>` |
| `LyricLine` | `time_ms: Option<u64>`（毫秒，`None` 表示无时间戳的纯文本行）、`text: String` |
| `SearchResult` | `keyword`、`songs`、`albums`、`artists`、`total: u64`、`page: u32`、`page_size: u32` |
| `Leaderboard` | `id`、`source_id`、`name`、`cover_url`、`songs: Vec<Song>` |

---

## 2. 快速开始

最小可用的音源 JSON 示例（仅含必填字段）：

```json
{
  "manifest": {
    "id": "demo-source",
    "name": "Demo Source",
    "version": "1.0.0",
    "author": "you"
  },
  "endpoints": {
    "search": { "url": "https://music.example.com/api/search" },
    "metadata": { "url": "https://music.example.com/api/metadata" },
    "playUrl": { "url": "https://music.example.com/api/play" }
  },
  "fieldMapping": {
    "song": { "id": "id", "title": "name" },
    "album": { "id": "albumId", "name": "albumName" },
    "artist": { "id": "artistId", "name": "artistName" },
    "lyric": { "lines": [{ "timeMs": "time", "text": "content" }] }
  }
}
```

加载与校验：

```rust
use music_core::sources::schema::{SoundSourceConfig, validate_config};
use music_core::sources::json::JsonSource;

// 1. 严格 Schema 校验（基于内嵌 JSON Schema）
let value: serde_json::Value = serde_json::from_str(&json_str).unwrap();
SoundSourceConfig::validate_strict(&value).expect("schema 校验失败");

// 2. 反序列化为强类型配置
let config = SoundSourceConfig::from_json(&json_str).unwrap();
// 3. 额外语义校验（如 auth.type != none 时 tokenParam 必填）
validate_config(&config).unwrap();

// 4. 构造音源并加入 SourceManager
let source = JsonSource::new(config).unwrap();
```

字段说明：

- `manifest.id` 必须匹配 `^[a-z0-9-]{1,64}$`（小写字母、数字、连字符）。
- `manifest.version` 必须为语义化版本号（`^\d+\.\d+\.\d+$`）。
- `endpoints.search` / `metadata` / `playUrl` 为必填，`lyric` / `leaderboards` 为可选。
- `fieldMapping.song.id` / `title`、`album.id` / `name`、`artist.id` / `name`、`lyric.lines` 为必填。

---

## 3. 标准 JSON Schema 说明

顶层对象必填字段为 `manifest`、`endpoints`、`fieldMapping`，`auth`、`pagination`、`timeoutMs` 为可选。所有对象均禁止额外字段（`additionalProperties: false`）。

### 3.1 `manifest`（音源清单）

| 字段 | 类型 | 必填 | 默认值 | 约束 / 示例 |
| --- | --- | --- | --- | --- |
| `id` | string | 是 | — | `^[a-z0-9-]{1,64}$`，如 `"netease"` |
| `name` | string | 是 | — | 长度 1–64，如 `"网易云音乐"` |
| `version` | string | 是 | — | semver `^\d+\.\d+\.\d+$`，如 `"1.2.0"` |
| `author` | string | 是 | — | 如 `"community"` |
| `description` | string | 否 | — | 音源描述 |
| `homepage` | string | 否 | — | URI 格式，如 `"https://example.com"` |

### 3.2 `endpoints`（接口定义）

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `search` | Endpoint | 是 | 搜索接口 |
| `metadata` | Endpoint | 是 | 元数据接口（按歌曲 id 获取单首详情） |
| `playUrl` | Endpoint | 是 | 播放 URL 接口（按歌曲 id 获取可播放 URL） |
| `lyric` | Endpoint | 否 | 歌词接口，未配置时 `get_lyric` 返回 `NotFound` |
| `leaderboards` | LeaderboardsEndpoint | 否 | 排行榜接口，未配置时返回空数组 |

#### Endpoint（search / metadata / playUrl / lyric 共用）

| 字段 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `url` | string | 是 | — | 接口 URL，URI 格式 |
| `method` | `"GET"` \| `"POST"` | 否 | `"GET"` | HTTP 方法（大写） |
| `queryParam` | string | 否 | `"keyword"` | 搜索关键词参数名 |
| `pageParam` | string | 否 | `"page"` | 页码参数名 |
| `pageSizeParam` | string | 否 | `"page_size"` | 每页数量参数名 |
| `headers` | object<string,string> | 否 | — | 自定义请求头 |
| `responseType` | `"json"` \| `"text"` | 否 | `"json"` | 响应类型；`text` 时整体作为字符串 |
| `idParam` | string | 否 | `"id"` | 歌曲 id 参数名（metadata / playUrl / lyric 使用） |

> GET 请求时，参数进入 query string；POST 请求时，参数进入 JSON body（值为字符串）。

#### LeaderboardsEndpoint

| 字段 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `url` | string | 是 | — | 排行榜列表接口 URL |
| `headers` | object<string,string> | 否 | — | 自定义请求头 |

> 排行榜接口仅支持 GET，期望返回数组，每个元素含 `id`、`name`、`coverUrl`/`cover_url`（可选）、`songs`（可选）。

### 3.3 `auth`（鉴权，可选）

| 字段 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `type` | `"none"` \| `"query"` \| `"header"` | 否 | `"none"` | 鉴权方式 |
| `tokenParam` | string | 条件必填 | — | `type` 为 `query` 或 `header` 时必填，token 参数名 |
| `token` | string | 否 | — | token 值，可在运行时注入 |

鉴权注入规则（见 `json.rs::request_json`）：

- `header` 模式：以请求头 `tokenParam: token` 注入每个请求。
- `query` 模式：GET 时注入 query 参数；POST 时注入 JSON body 字段。
- `none` 模式：不注入。

> 语义校验（`validate_config`）：`type` 为 `query` / `header` 时，若 `tokenParam` 为空则报错。

### 3.4 `fieldMapping`（字段映射）

| 子对象 | 必填 | 说明 |
| --- | --- | --- |
| `song` | 是 | 歌曲字段映射 |
| `album` | 是 | 专辑字段映射 |
| `artist` | 是 | 艺术家字段映射 |
| `lyric` | 是 | 歌词字段映射 |
| `searchResult` | 否 | 搜索结果容器字段映射 |

#### song

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 歌曲 id 字段名或点路径 |
| `title` | string | 是 | 标题字段名或点路径 |
| `artists` | string | 否 | 艺术家字段名（值可为字符串或数组） |
| `album` | string | 否 | 专辑名字段名 |
| `coverUrl` | string | 否 | 封面 URL 字段名 |
| `durationMs` | string | 否 | 时长字段名（数值，单位毫秒） |
| `lyricUrl` | string | 否 | 歌词 URL 字段名 |
| `playUrl` | string | 否 | 播放 URL 字段名 |

#### album

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 专辑 id |
| `name` | string | 是 | 专辑名 |
| `artists` | string | 否 | 艺术家（数组或字符串） |
| `coverUrl` | string | 否 | 封面 URL |
| `songIds` | string | 否 | 歌曲 id 列表（数组） |

#### artist

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `id` | string | 是 | 艺术家 id |
| `name` | string | 是 | 艺术家名 |
| `avatarUrl` | string | 否 | 头像 URL |
| `songIds` | string | 否 | 歌曲 id 列表（数组） |

#### lyric

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `lines` | array<LyricLineMapping> | 是 | 歌词行映射，每项含 `timeMs` 与 `text` 字段名 |
| `timeField` | string | 否 | 统一时间字段名（保留位） |

`LyricLineMapping`：

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `timeMs` | string | 是 | 时间字段名（毫秒数值） |
| `text` | string | 是 | 歌词文本字段名 |

> 歌词响应可为 JSON 数组（直接是行列表），或对象中含 `lines` 字段。引擎会自动识别。

#### searchResult（可选）

| 字段 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| `total` | string | 否 | 总数字段名（数值） |
| `songs` | string | 否 | 歌曲列表字段名（数组） |
| `albums` | string | 否 | 专辑列表字段名（数组） |
| `artists` | string | 否 | 艺术家列表字段名（数组） |

### 3.5 `pagination`（分页，可选）

| 字段 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `pageStart` | number | 否 | `0` | 起始页码 |
| `pageSizeDefault` | number | 否 | `20` | 默认每页数量 |

### 3.6 `timeoutMs`（请求超时，可选）

| 字段 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| `timeoutMs` | number | 否 | `10000` | 请求超时（毫秒），用于构建 reqwest 客户端；`<= 0` 时回退为 10000 |

---

## 4. 字段映射详解

字段映射由 `core/src/sources/json.rs` 的 `extract_field` 实现，规则如下：

### 4.1 简单字段名

形如 `"title"`，等价于 `value["title"]`：

```json
{ "id": "s1", "name": "My Song" }
```

```json
"fieldMapping": { "song": { "id": "id", "title": "name" } }
```

### 4.2 点路径（dotted path）

形如 `"data.song.name"`，等价于 `value["data"]["song"]["name"]`，用于嵌套响应：

```json
{ "data": { "song": { "id": "s2", "name": "Nested" } } }
```

```json
"fieldMapping": { "song": { "id": "data.song.id", "title": "data.song.name" } }
```

任一段不存在或 mapping 为空都返回 `None`，缺失字段返回默认值（空数组 / `None`），**不会 panic**。

### 4.3 artists：字符串或数组

`artists` 字段的值可为：

- 字符串（单艺术家）→ 映射为 `vec![s]`；
- 字符串数组（多艺术家）→ 映射为 `Vec<String>`。

由 `collect_artists` 统一处理：

```json
"artists": "Solo"               // → ["Solo"]
"artists": ["A", "B", "C"]      // → ["A", "B", "C"]
```

### 4.4 durationMs：数值毫秒

`durationMs` 为数值（毫秒），支持整数与浮点（截断）。由 `value_to_u64` 处理：

```json
"durationMs": 180000      // 整数
"durationMs": 180000.0     // 浮点，截断为 180000
```

### 4.5 多时间戳歌词行

歌词响应支持两种形态：

1. 响应本身是数组（每项为一行）：

```json
[
  { "time": 12340, "content": "第一行" },
  { "time": 15670, "content": "第二行" }
]
```

2. 响应对象含 `lines` 字段：

```json
{
  "lines": [
    { "time": 12340, "content": "第一行" },
    { "time": 15670, "content": "第二行" }
  ]
}
```

映射配置（取首个 `LyricLineMapping` 描述每行的字段名）：

```json
"lyric": { "lines": [{ "timeMs": "time", "text": "content" }] }
```

引擎逐行解析：`time_ms` 缺失时为 `None`（纯文本歌词行），`text` 缺失时为空字符串。

### 4.6 playUrl 提取回退

`get_play_url` 优先用 `song.playUrl` 字段映射提取；若映射缺失或字段不存在，则退化为将整个响应作为字符串返回（适用于接口直接返回纯文本 URL 的场景）。

---

## 5. 元数据 API 对接

以对接主流音乐软件为例。在线音乐 API 通常提供：搜索接口（按关键词返回歌曲列表）、歌曲详情接口（按 id 返回元数据）、播放 URL 接口（按 id 返回可播放 URL）、歌词接口（按 id 返回 LRC 或结构化歌词）。

### 5.1 对接要点

1. **请求参数**：通过 `queryParam` / `pageParam` / `pageSizeParam` / `idParam` 配置参数名，使其匹配目标 API。GET 进入 query，POST 进入 JSON body。
2. **响应解析**：通过 `fieldMapping` + `searchResult` 描述如何从响应中取出歌曲列表、总数与各字段；嵌套结构用点路径。
3. **字段映射**：把音源的字段名映射到标准字段，例如音源用 `songName` 表标题，则 `song.title = "songName"`。

### 5.2 完整对接示例：搜索接口

假设音源搜索接口为 `GET https://music.example.com/api/search`，参数 `kw`（关键词）、`p`（页码，从 1 开始）、`n`（每页数量），响应结构：

```json
{
  "code": 0,
  "data": {
    "total": 128,
    "list": [
      {
        "songId": "1001",
        "songName": "晴天",
        "singers": ["周杰伦"],
        "albumName": "叶惠美",
        "cover": "https://music.example.com/covers/1001.jpg",
        "duration": 240000,
        "playUrl": "https://music.example.com/stream/1001.mp3"
      }
    ]
  }
}
```

对应音源配置：

```json
{
  "manifest": {
    "id": "demo-music",
    "name": "示例音乐",
    "version": "1.0.0",
    "author": "demo"
  },
  "endpoints": {
    "search": {
      "url": "https://music.example.com/api/search",
      "method": "GET",
      "queryParam": "kw",
      "pageParam": "p",
      "pageSizeParam": "n",
      "responseType": "json"
    },
    "metadata": { "url": "https://music.example.com/api/song/detail", "idParam": "songId" },
    "playUrl": { "url": "https://music.example.com/api/song/url", "idParam": "songId" },
    "lyric": { "url": "https://music.example.com/api/song/lyric", "idParam": "songId" }
  },
  "auth": { "type": "none" },
  "fieldMapping": {
    "song": {
      "id": "songId",
      "title": "songName",
      "artists": "singers",
      "album": "albumName",
      "coverUrl": "cover",
      "durationMs": "duration",
      "playUrl": "playUrl"
    },
    "album": { "id": "albumId", "name": "albumName" },
    "artist": { "id": "singerId", "name": "singerName" },
    "lyric": { "lines": [{ "timeMs": "time", "text": "content" }] },
    "searchResult": {
      "total": "data.total",
      "songs": "data.list"
    }
  },
  "pagination": { "pageStart": 1, "pageSizeDefault": 20 },
  "timeoutMs": 8000
}
```

请求示例（搜索 `晴天`，第 1 页，每页 20 条）：

```
GET https://music.example.com/api/search?kw=晴天&p=1&n=20
```

解析结果（伪 Song）：

```json
{
  "id": "1001",
  "source_id": "demo-music",
  "title": "晴天",
  "artists": ["周杰伦"],
  "album": "叶惠美",
  "cover_url": "https://music.example.com/covers/1001.jpg",
  "duration_ms": 240000,
  "play_url": "https://music.example.com/stream/1001.mp3",
  "local_path": null,
  "origin": { "type": "Online", "source_id": "demo-music", "play_url": "https://music.example.com/stream/1001.mp3" }
}
```

### 5.3 主流软件对接注意事项

- **网易云 / QQ音乐**：通常需 Cookie 或鉴权头，使用 `auth.type = "header"` + `tokenParam`（如 `Cookie` 或 `Authorization`），`token` 可在运行时注入。响应多为嵌套结构，务必用点路径（如 `result.songs`）。
- **分页起始**：部分 API 页码从 1 开始（如示例），通过 `pagination.pageStart = 1` 声明；从 0 开始则用默认 `0`。
- **响应类型**：若接口直接返回纯文本播放 URL，将该 endpoint 的 `responseType` 设为 `"text"`，或在 `get_play_url` 阶段依赖整体字符串回退。
- **编码**：路径与 query 参数由 reqwest 自动做 URL 编码，无需手动处理。

---

## 6. 搜索定位与缓存播放对接

### 6.1 完整流程

```
用户输入关键词
   │
   ▼
GET /sources/search?keyword=...&page=1&page_size=20
   │  聚合搜索（Aggregator，core/src/search.rs）
   │  顺序查询各启用音源，单个失败记录 warn 并跳过
   ▼
SearchResult { songs: [Song{id, title, artists, ...}, ...] }
   │
   ▼ 用户点击某首歌
GET /sources/{id}/songs/{songId}/metadata
   │  get_metadata(songId) 补全字段（专辑、封面、时长等）
   ▼
GET /sources/{id}/songs/{songId}/play-url
   │  get_play_url(songId) 返回可播放 URL
   ▼
缓存层（CacheManager，core/src/cache.rs）
   │  key = song_id；命中返回本地 .cache 文件路径
   │  未命中下载流数据写入 cache_dir/<sanitized_key>.cache
   │  超容量按 LRU（最久未用）淘汰
   ▼
播放器播放（命中缓存 → 本地播放；未命中 → 在线流式 + 写缓存）
```

### 6.2 缓存命中优先本地播放

`CacheManager` 基于 LRU 文件缓存（`core/src/cache.rs`）：

- **缓存 key**：`song_id`（音源内唯一）。key 中非字母数字字符会被 `sanitize_key` 替换为 `_`，作为文件名与索引键。
- **缓存文件**：`cache_dir/<sanitized_key>.cache`。
- **命中**：`get_or_fetch` 命中时直接返回本地文件路径并更新 `last_access`，不调用 fetcher。
- **未命中**：调用 fetcher 下载字节数据，同步写入文件，更新索引与字节计数，触发 LRU 淘汰。
- **容量上限**：`max_entries` 固定为 1000，`max_bytes` 由调用方传入；超容量时按 `last_access` 升序淘汰。
- **持久化**：进程重启后扫描 `cache_dir` 下所有 `.cache` 文件重建内存索引，缓存仍可用。

播放策略：优先检查缓存是否命中（`has` / `get_path`），命中则播放本地缓存文件，避免重复下载；未命中则在线流式播放并在后台写缓存。

### 6.3 get_metadata 用于补全

搜索结果中的 `Song` 可能仅含部分字段（如 `play_url` 缺失）。点击播放前调用 `get_metadata` 补全封面、时长、专辑等字段，再调用 `get_play_url` 取得可播放 URL。

---

## 7. 社区音源迁移方案

社区维护的音源 JSON 格式多样，`core/src/sources/community.rs` 提供适配层，导入时自动转换为标准 `SoundSourceConfig`。

### 7.1 支持的社区格式

| 格式 | 识别特征 | 字段映射 |
| --- | --- | --- |
| 标准格式 | 同时含 `manifest` + `endpoints` + `fieldMapping` | 原样返回 |
| 社区格式 A | `{ "sources": [ { "name", "url", "type" } ] }`（sources 数组） | 取首个 source：`name` → `manifest.name`，`url` → `endpoints.search.url`，`type` → `manifest.id`（slug 化） |
| 社区格式 B | 扁平 endpoint：含 `search_url` / `song_url` / `metadata_url` 等 | `name` → `manifest.name`，`search_url` → `endpoints.search.url`，`song_url` → `endpoints.playUrl.url`，`metadata_url` → `endpoints.metadata.url` |

字段名兼容 snake_case / camelCase / 点号分隔（如 `search.url`）：`pick_str` 按候选键名列表依次匹配。

### 7.2 adapt_with_report 返回转换报告

```rust
use music_core::sources::community::adapt_with_report;

let report = adapt_with_report(&raw_json)?;
// AdaptResult { config, source_format, warnings }
// - config: 转换后的标准 SoundSourceConfig JSON（可通过 validate_strict）
// - source_format: "standard" / "community-a" / "community-b"
// - warnings: 警告列表
```

`source_format` 取值：

- `"standard"`：已是标准格式，无警告。
- `"community-a"`：sources 数组格式转换。
- `"community-b"`：扁平 endpoint 格式转换。

### 7.3 警告（warnings）类型

- 社区格式 A 未提供 `metadata` / `playUrl` 接口，已使用占位 URL `https://example.com/unsupported`。
- 社区格式 A 含多个 source，仅取首个。
- 社区格式 B 缺少 `search_url` / `song_url` / `metadata_url`，已使用占位 URL。
- 存在未映射字段：如 `extra_field`、`another_extra`。

### 7.4 失败回退

- 非对象 JSON、无法识别的格式 → 返回 `Err(CoreError::Schema("无法识别的社区音源格式"))`。
- 社区格式 A 缺少 `name` 或 `url` → 返回 `Err`。
- 社区格式 B 缺少 `name` → 返回 `Err`。
- `slugify` 处理：转小写、非字母数字替换为 `-`、去重 `-`、截断到 64 字符；纯符号/纯中文名称结果为空时回退为 `"source"`，保证符合 schema pattern。

调用方应捕获错误并向用户提示「导入失败：原因」，同时可展示 `warnings` 帮助用户理解转换情况。

---

## 8. 本地音源

内置本地音源（`core/src/sources/local.rs::LocalSource`），无需编写 JSON：

- **音源标识**：`id = "local"`，`name = "本地音乐"`。
- **接入聚合搜索**：作为 `Source` trait 实现，自动加入 `SourceManager`，搜索结果标注「本地」来源（`origin.type = "Local"`）。
- **支持的扩展名**：`mp3`、`flac`、`m4a`、`ape`、`ogg`、`wav`、`aac`（大小写不敏感）。
- **目录扫描**：递归扫描用户指定的根目录（`add_directory`），按扩展名过滤，lofty 解析元数据后 upsert 入 SQLite。
- **元数据解析**：使用 lofty（支持 ID3v1/v2、FLAC、MP4、APE 标签）。缺失字段回退：title → 文件名 stem，artists → `["Unknown Artist"]`。多艺术家按 `;` 或 `/` 分割。
- **持久化索引**：SQLite 表 `local_tracks`（id/path/title/artist/album/cover/duration_ms/mtime），路径唯一约束，upsert 时保留原 id 仅更新其余字段。
- **增量监听**：notify watcher 监听目录变化（Create/Modify/Remove），文件存在则 upsert、不存在则删除，无需全量重扫。
- **重新扫描**：`rescan` 重新解析所有现存文件并 upsert，删除磁盘上已不存在的记录。
- **扫描进度**：`scan_progress()` 返回 `ScanProgress { current_count, scanning }`。
- **歌词/排行榜**：本地音源 `get_lyric` 返回 `NotFound`，`get_leaderboards` 返回空数组。

本地歌曲的 `Song`：

```json
{
  "id": "<uuid>",
  "source_id": "local",
  "title": "My Song",
  "artists": ["Unknown Artist"],
  "local_path": "/music/My Song.mp3",
  "origin": { "type": "Local", "path": "/music/My Song.mp3" }
}
```

---

## 9. 示例音源

### 9.1 标准格式（无鉴权）

```json
{
  "manifest": {
    "id": "demo-standard",
    "name": "示例标准音源",
    "version": "1.0.0",
    "author": "docs",
    "description": "无鉴权的标准音源示例",
    "homepage": "https://example.com"
  },
  "endpoints": {
    "search": {
      "url": "https://music.example.com/api/search",
      "method": "GET",
      "queryParam": "keyword",
      "pageParam": "page",
      "pageSizeParam": "page_size",
      "responseType": "json"
    },
    "metadata": { "url": "https://music.example.com/api/song/detail", "idParam": "id" },
    "playUrl": { "url": "https://music.example.com/api/song/url", "idParam": "id" },
    "lyric": { "url": "https://music.example.com/api/song/lyric", "idParam": "id" },
    "leaderboards": {
      "url": "https://music.example.com/api/leaderboards",
      "headers": { "X-Client": "music-player" }
    }
  },
  "auth": { "type": "none" },
  "fieldMapping": {
    "song": {
      "id": "id",
      "title": "name",
      "artists": "artists",
      "album": "album.name",
      "coverUrl": "album.cover",
      "durationMs": "duration",
      "lyricUrl": "lyricUrl",
      "playUrl": "playUrl"
    },
    "album": { "id": "album.id", "name": "album.name", "coverUrl": "album.cover", "songIds": "songIds" },
    "artist": { "id": "artist.id", "name": "artist.name", "avatarUrl": "artist.avatar" },
    "lyric": { "lines": [{ "timeMs": "time", "text": "content" }] },
    "searchResult": { "total": "total", "songs": "songs", "albums": "albums", "artists": "artists" }
  },
  "pagination": { "pageStart": 0, "pageSizeDefault": 20 },
  "timeoutMs": 10000
}
```

### 9.2 含鉴权（header 模式）

```json
{
  "manifest": {
    "id": "authed-source",
    "name": "需鉴权音源",
    "version": "1.0.0",
    "author": "docs"
  },
  "endpoints": {
    "search": {
      "url": "https://api.music.example.com/v1/search",
      "method": "GET",
      "queryParam": "q",
      "headers": { "Accept": "application/json" }
    },
    "metadata": { "url": "https://api.music.example.com/v1/songs", "idParam": "songId" },
    "playUrl": { "url": "https://api.music.example.com/v1/songs/url", "idParam": "songId" }
  },
  "auth": {
    "type": "header",
    "tokenParam": "Authorization",
    "token": "Bearer <your-token-here>"
  },
  "fieldMapping": {
    "song": { "id": "songId", "title": "title", "artists": "singers", "durationMs": "durationMs", "playUrl": "url" },
    "album": { "id": "albumId", "name": "albumName" },
    "artist": { "id": "artistId", "name": "artistName" },
    "lyric": { "lines": [{ "timeMs": "timeMs", "text": "text" }] },
    "searchResult": { "total": "data.total", "songs": "data.songs" }
  },
  "pagination": { "pageStart": 1, "pageSizeDefault": 30 },
  "timeoutMs": 8000
}
```

> 运行时也可不写死 `token`，由宿主在加载音源后注入（构造 `JsonSource` 前修改 `config.auth.token`）。

### 9.3 社区格式迁移说明

社区格式 A（sources 数组）：

```json
{
  "sources": [
    { "name": "社区音源", "url": "https://community.example.com/search", "type": "music" }
  ]
}
```

调用 `adapt_with_report` 转换后：

```json
{
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
```

报告：

- `source_format`: `"community-a"`
- `warnings`: `["社区格式 A 未提供 metadata/playUrl 接口，已使用占位 URL"]`

> 占位 URL 接口在调用 metadata / playUrl 时会返回网络错误，建议用户在迁移后补全对应接口或仅在搜索场景使用。

---

## 10. 错误码与调试

核心错误类型定义在 `core/src/error.rs::CoreError`，统一通过 `Result<T>` 传播。

### 10.1 常见错误

| 错误类型 | 触发场景 | 错误信息示例 | 排查建议 |
| --- | --- | --- | --- |
| `Schema` | 音源 JSON 不符合标准 Schema | `音源配置校验失败: 路径 manifest: 缺少必填属性 id` | 按 schema 逐字段检查；`manifest.id` 须匹配 `^[a-z0-9-]{1,64}$` |
| `Schema` | 社区音源无法识别 | `无法识别的社区音源格式` | 检查是否符合标准 / sources 数组 / 扁平 endpoint 三种格式 |
| `Schema` | `auth.type` 为 query/header 但 `tokenParam` 为空 | `auth.type 为 "header" 时必须提供非空 tokenParam` | 补全 `tokenParam` |
| `Json` | 响应不是合法 JSON（`responseType=json`） | `JSON 序列化/反序列化错误: ...` | 检查接口返回；若返回纯文本，改用 `responseType: "text"` |
| `Http` | 网络错误 / 非 2xx 响应 | `HTTP 请求错误: ...` / `error_for_status` | 检查 URL、鉴权、网络；`timeoutMs` 是否过小 |
| `Source` | 字段映射无法提取歌曲（id/title 缺失） | `无法从元数据响应映射歌曲: <song_id>` | 核对 `fieldMapping.song.id` / `title` 字段名是否与响应一致 |
| `Source` | 歌词响应格式错误 | `歌词响应格式错误：应为数组或含 lines 字段` | 调整接口返回为行数组，或包成 `{ "lines": [...] }` |
| `Source` | 无法提取播放 URL | `无法从响应中提取播放 URL` | 配置 `song.playUrl` 映射，或确保接口返回纯文本 URL |
| `NotFound` | 音源未配置歌词接口 | `音源 <id> 未配置歌词接口` | 在 `endpoints.lyric` 中配置歌词接口 |
| `NotFound` | 歌曲 id 不存在 | `本地歌曲不存在: <song_id>` / `未找到资源: ...` | 核对 song_id 是否来自该音源的搜索结果 |

### 10.2 调试技巧

- **离线校验**：在加载音源前先调用 `SoundSourceConfig::validate_strict(&value)`，错误信息含具体 JSON 路径。
- **字段映射**：用 `extract_field` 的点路径逐级验证。若 `total` 始终为 0，检查 `searchResult.total` 字段名是否与响应一致。
- **artists 类型**：若艺术家列表为空，确认目标字段是字符串还是数组；两者均被支持，但字段名必须指向真正的艺术家字段。
- **缓存**：查看 `cache_dir` 下 `.cache` 文件与 `CacheManager::stats()`（`entries` / `total_bytes` / `max_bytes`）判断命中与淘汰情况。
- **社区迁移**：检查 `adapt_with_report` 返回的 `warnings`，定位未映射字段与占位 URL。
- **日志**：聚合搜索中单个音源失败会通过 `log::warn!("音源 {} 搜索失败，已跳过: {}", ...)` 记录，不影响整体结果；排查时关注该日志。
