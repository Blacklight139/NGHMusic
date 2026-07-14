// 职责：HarmonyOS 通过 NAPI（napi-rs）调用 music-core（Rust）的 ArkTS 桥接层。
// 封装：appVersion/importSource/search/listLocalSongs/play/getMetadata/getPlayUrl/getLyric/
//       getLeaderboards/feiniu*/protocol*/local*/cache*/sourceValidate。
//
// FFI 方案：NAPI（Node-API）/ napi-rs。
// 启用步骤（需在 core 增加 napi-rs 依赖并构建 NAPI 产物）：
// 1. 在 core/Cargo.toml 增加：
//    [lib] crate-type = ["cdylib"]  （HarmonyOS 仅需 cdylib）
//    [dependencies] napi = { version = "2", features = ["napi8"] }
//                   napi-derive = "2"
//    [build-dependencies] napi-build = "2"
// 2. 在 core/ 新建 build.rs：fn main() { napi_build::setup(); }
// 3. 在 core/src 新建 napi 模块（如 napi.rs），用 #[napi] 标注导出函数：
//    #[napi]
//    pub fn app_version() -> String { env!("CARGO_PKG_VERSION").into() }
//    #[napi]
//    pub fn import_source(json: String) -> Result<String> { ... }
//    #[napi]
//    pub fn search(keyword: String, page: u32, page_size: u32) -> SearchResult { ... }
//    #[napi]
//    pub fn list_local_songs() -> Vec<Song> { ... }
//    #[napi]
//    pub fn play(song_id: String) -> Result<String> { ... }
//    并用 #[napi(object)] 派生 Song/SearchResult 等结构。
// 4. 交叉编译为 HarmonyOS NAPI 动态库（.so）：
//    cargo build --release --target aarch64-linux-ohos
//    （需安装 HarmonyOS NDK，配置 ~/.cargo/config.toml 指向 OHOS 工具链）
//    产物 libmusic_core.so 重命名为 libmusiccore.so，放入 entry/src/main/libs/<abi>/，
//    并在 entry/build-profile.json5 的 abiFilters 中配置对应 ABI。
// 5. 在 ArkTS 侧 import：
//    import { appVersion, importSource, search, listLocalSongs, play } from 'libmusiccore.so';
//    （HarmonyOS 通过 .so 模块名导入 NAPI 导出符号）
//
// 说明：本桥接假设 NAPI 模块名为 libmusiccore.so，导出函数 appVersion 等。
//       若 .so 未链接，调用降级为占位实现（返回空/默认值），保证脚手架可编译。
//
// 数据类型：统一从 ../models/Models.ets 复用，与 core/src/models.rs 严格对齐（snake_case 字段名）。
// 修复 HAR-003：SongOrigin 的 Nas 变体使用 `protocol` 字段（NOT `protocol_name`），见 Models.ets。

export {
  Song, SongOrigin, Album, Artist, Lyric, LyricLine,
  SearchResult, Leaderboard, Playlist, FavoriteGroup,
  PlayMode, PlayState, NasFile,
  playModeToSnakeString, playModeFromSnakeString,
} from '../models/Models';

import type { Song, SearchResult } from '../models/Models';

// 音源信息（UI 辅助结构：sourceType 为 camelCase 显示字段，与 core SourceManager SourceInfo 对齐：
// id/name/version/enabled/sourceType/priority/description）。该结构仅用于音源管理 UI 展示，
// 不直接来自 Rust 模型序列化，故保留 camelCase 命名以与 ArkTS 习惯一致。
export interface SourceInfo {
  id: string;
  name: string;
  version: string;
  enabled: boolean;
  sourceType: string; // 'json' | 'community' | 'local'
  priority: number;
  description: string;
}

// 标记 NAPI 模块是否可用（占位：默认 false，链接 .so 后改为 true）
const NAPI_ENABLED = false;

// 脚手架占位：NAPI 未链接时使用的内存音源列表，便于音源管理 UI 在未链接 .so 时也可演示/验证。
// 链接 music-core 后由核心 SourceManager 真正管理，本数组不再使用。
const scaffoldSources: SourceInfo[] = [];
let scaffoldPrioritySeq = 0;

// NAPI 导出符号引用（实际启用后取消注释）
// import * as musicCore from 'libmusiccore.so';

export class MusicCoreBridge {
  /** 返回核心库版本号 */
  static appVersion(): string {
    if (NAPI_ENABLED) {
      // return musicCore.appVersion();
    }
    return '0.1.0-scaffold';
  }

  /** 导入音源 JSON（旧入口，返回提示文本，保留以兼容） */
  static importSource(json: string): string {
    if (NAPI_ENABLED) {
      // return musicCore.importSource(json);
    }
    return `（占位）音源 JSON 已接收，长度 ${json.length} 字符；链接 music-core 后生效`;
  }

  /** 列出全部音源（按 priority 升序） */
  static listSourcesOrdered(): SourceInfo[] {
    if (NAPI_ENABLED) {
      // return musicCore.listSourcesOrdered();
    }
    return scaffoldSources.slice().sort((a: SourceInfo, b: SourceInfo) => a.priority - b.priority);
  }

  /** 更新指定音源优先级 */
  static updateSourcePriority(id: string, newPriority: number): void {
    if (NAPI_ENABLED) {
      // musicCore.updateSourcePriority(id, newPriority);
      return;
    }
    const s = scaffoldSources.find((x: SourceInfo) => x.id === id);
    if (s) { s.priority = newPriority; }
  }

  /** 按 orderedIds 顺序重排全部音源（下标即新 priority） */
  static reorderSources(orderedIds: string[]): void {
    if (NAPI_ENABLED) {
      // musicCore.reorderSources(orderedIds);
      return;
    }
    orderedIds.forEach((id: string, idx: number) => {
      const s = scaffoldSources.find((x: SourceInfo) => x.id === id);
      if (s) { s.priority = idx; }
    });
  }

  /** 删除指定音源 */
  static deleteSource(id: string): void {
    if (NAPI_ENABLED) {
      // musicCore.deleteSource(id);
      return;
    }
    const idx = scaffoldSources.findIndex((x: SourceInfo) => x.id === id);
    if (idx >= 0) { scaffoldSources.splice(idx, 1); }
  }

  /** 启用/禁用指定音源 */
  static setSourceEnabled(id: string, enabled: boolean): void {
    if (NAPI_ENABLED) {
      // musicCore.setSourceEnabled(id, enabled);
      return;
    }
    const s = scaffoldSources.find((x: SourceInfo) => x.id === id);
    if (s) { s.enabled = enabled; }
  }

  /** 由 JSON 字符串导入音源，返回新建的 SourceInfo；失败返回 null */
  static importSourceFromJson(json: string): SourceInfo | null {
    if (NAPI_ENABLED) {
      // return musicCore.importSourceFromJson(json);
    }
    try {
      const obj = JSON.parse(json) as Record<string, Object>;
      const rawType = obj['source_type'] ?? obj['sourceType'];
      const info: SourceInfo = {
        id: String(obj['id'] ?? `src_${Date.now()}`),
        name: String(obj['name'] ?? '未命名音源'),
        version: String(obj['version'] ?? '1.0.0'),
        enabled: Boolean(obj['enabled'] ?? true),
        sourceType: rawType !== undefined ? String(rawType) : 'json',
        priority: scaffoldPrioritySeq++,
        description: String(obj['description'] ?? ''),
      };
      scaffoldSources.push(info);
      return info;
    } catch (e) {
      return null;
    }
  }

  /** 聚合搜索 */
  static search(keyword: string, page: number, pageSize: number): SearchResult | null {
    if (NAPI_ENABLED) {
      // return musicCore.search(keyword, page, pageSize);
    }
    return null;
  }

  /** 列出本地音乐 */
  static listLocalSongs(): Song[] | null {
    if (NAPI_ENABLED) {
      // return musicCore.listLocalSongs();
    }
    return null;
  }

  /** 解析并返回可播放 URL */
  static play(songId: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.play(songId);
    }
    return null;
  }

  // ---- 音源校验 ----

  /** 校验音源 JSON 是否符合标准 Schema，返回 {"valid":bool,"errors":[...]} JSON。 */
  static sourceValidate(json: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.sourceValidate(json);
    }
    return null;
  }

  // ---- 歌曲元数据/歌词/排行榜 ----
  // 实际启用后均替换为 musicCore.* 调用，返回核心序列化的 JSON 字符串。

  /** 获取指定音源下歌曲的完整元数据，返回 Song JSON。 */
  static getMetadata(sourceId: string, songId: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.getMetadata(sourceId, songId);
    }
    return null;
  }

  /** 获取指定音源下歌曲的可播放 URL，返回 {"url":...,"cached":false}。 */
  static getPlayUrl(sourceId: string, songId: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.getPlayUrl(sourceId, songId);
    }
    return null;
  }

  /** 获取指定音源下歌曲的歌词，返回 Lyric JSON。 */
  static getLyric(sourceId: string, songId: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.getLyric(sourceId, songId);
    }
    return null;
  }

  /** 获取指定音源的排行榜列表，返回 Leaderboard 数组 JSON。 */
  static getLeaderboards(sourceId: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.getLeaderboards(sourceId);
    }
    return null;
  }

  // ---- 飞牛 NAS ----
  // 实际启用后替换为 musicCore.feiniu* 调用，返回核心 JSON。

  /** 登录飞牛 NAS，返回 {"token":...,"base_url":...}。 */
  static feiniuLogin(baseUrl: string, username: string, password: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.feiniuLogin(baseUrl, username, password);
    }
    return null;
  }

  /** 列出飞牛 NAS 指定路径下的文件，返回 {"path":...,"files":[...]}。 */
  static feiniuListFiles(path: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.feiniuListFiles(path);
    }
    return null;
  }

  /** 生成飞牛 NAS 文件的可流式播放 URL，返回 {"url":...}。 */
  static feiniuStream(path: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.feiniuStream(path);
    }
    return null;
  }

  /** 飞牛服务健康检查，返回 {"healthy":bool,"base_url":...}。 */
  static feiniuHealth(): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.feiniuHealth();
    }
    return null;
  }

  // ---- 协议源（SMB/WebDAV/FTP/DLNA/NFS）----
  // 实际启用后替换为 musicCore.protocol* 调用，返回核心 JSON。
  // WebDAV/FTP 完整实现，SMB/DLNA/NFS 为占位实现（调用返回 Protocol 错误）。

  /** 添加一个远程协议源，返回 {"id":...,"protocol":...,"root":...,"enabled":true,"placeholder":bool}。 */
  static protocolAdd(configJson: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.protocolAdd(configJson);
    }
    return null;
  }

  /** 列出已加载的协议源，返回 {"sources":[...]}。 */
  static protocolList(): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.protocolList();
    }
    return null;
  }

  /** 删除指定 id 的协议源，返回 {"id":...,"deleted":bool}。 */
  static protocolDelete(id: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.protocolDelete(id);
    }
    return null;
  }

  /** 列出协议源下指定路径的条目名称，返回 {"path":...,"entries":[...]}。 */
  static protocolListFiles(id: string, path: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.protocolListFiles(id, path);
    }
    return null;
  }

  /** 读取协议源下指定文件为字节，返回 {"size":N,"data_base64":"..."}。 */
  static protocolRead(id: string, path: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.protocolRead(id, path);
    }
    return null;
  }

  /** 生成协议源下指定文件的可流式播放 URL，返回 {"url":...}。 */
  static protocolStream(id: string, path: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.protocolStream(id, path);
    }
    return null;
  }

  // ---- 本地音乐管理 ----
  // 实际启用后替换为 musicCore.local* 调用，返回核心 JSON。

  /** 初始化本地音乐源（打开/创建 SQLite 索引库），返回 {"ok":true}。 */
  static localInit(dbPath: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.localInit(dbPath);
    }
    return null;
  }

  /** 添加本地扫描目录并递归扫描入库，返回 {"ok":true}。 */
  static localAddDir(dir: string): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.localAddDir(dir);
    }
    return null;
  }

  /** 重新扫描所有已添加目录（增量更新），返回 {"ok":true}。 */
  static localRescan(): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.localRescan();
    }
    return null;
  }

  /** 返回本地扫描进度，返回 {"current_count":N,"scanning":bool}。 */
  static localProgress(): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.localProgress();
    }
    return null;
  }

  // ---- 缓存 ----
  // 实际启用后替换为 musicCore.cache* 调用，返回核心 JSON。

  /** 初始化播放缓存管理器，返回 {"ok":true}。 */
  static cacheInit(cacheDir: string, maxBytes: number): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.cacheInit(cacheDir, maxBytes);
    }
    return null;
  }

  /** 返回缓存统计，返回 {"entries":N,"total_bytes":N,"max_bytes":N}。 */
  static cacheStats(): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.cacheStats();
    }
    return null;
  }

  /** 清空所有缓存文件与索引，返回 {"ok":true}。 */
  static cacheClear(): string | null {
    if (NAPI_ENABLED) {
      // return musicCore.cacheClear();
    }
    return null;
  }
}
