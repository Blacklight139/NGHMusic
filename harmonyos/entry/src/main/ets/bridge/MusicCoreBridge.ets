// 职责：HarmonyOS 通过 NAPI（napi-rs）调用 music-core（Rust）的 ArkTS 桥接层。
// 封装：appVersion/importSource/search/listLocalSongs/play。
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

// 数据类型（与 core/src/models.rs 对齐）
export interface Song {
  id: string;
  source_id: string;
  title: string;
  artists: string[];
  album?: string;
  cover_url?: string;
  duration_ms?: number;
  lyric_url?: string;
  play_url?: string;
  local_path?: string;
  origin: SongOrigin;
}

export type SongOrigin =
  | { type: 'Online'; source_id: string; play_url: string }
  | { type: 'Local'; path: string }
  | { type: 'Nas'; protocol_name: string; url: string };

export interface SearchResult {
  keyword: string;
  songs: Song[];
  albums: Album[];
  artists: Artist[];
  total: number;
  page: number;
  page_size: number;
}

export interface Album { id: string; source_id: string; name: string; artists: string[]; cover_url?: string; song_ids: string[]; }
export interface Artist { id: string; source_id: string; name: string; avatar_url?: string; song_ids: string[]; }

// 音源信息（与 core SourceManager SourceInfo 对齐：id/name/version/enabled/sourceType/priority/description）
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
}
