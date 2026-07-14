// 职责：NAS / 协议音源浏览页（HarmonyOS ArkUI）：
// - 飞牛 NAS：健康检查、登录、目录浏览、点击音频文件经 feiniu_stream 拉流播放。
// - 协议源：添加 / 列出 / 删除、浏览目录、点击音频文件经 protocol_stream 拉流播放。
// 重试策略遵循 docs：502/504 指数退避（1s/2s/4s，最多 3 次），401 提示重新登录，
// 404 修正路径，501 占位提示（SMB/DLNA/NFS）。
// 图标统一使用 sys.symbol.* 系统符号（无 emoji），豆包风格 ngh_* token。

import { MusicCoreBridge, NasFile } from '../bridge/MusicCoreBridge';
import { playerManager } from '../player/PlayerManager';

// 协议源本地视图结构（解析 protocol_list 返回 JSON）
interface ProtocolSourceDto {
  id: string;
  protocol: string;
  root: string;
  enabled: boolean;
  placeholder: boolean;
}

@Component
export struct NasPage {
  @State mode: number = 0; // 0 = 飞牛, 1 = 协议
  // 飞牛
  @State baseUrl: string = '';
  @State username: string = '';
  @State password: string = '';
  @State healthText: string = '未检测';
  @State healthOk: number = -1; // -1 未知, 0 不可达, 1 可达
  @State feiniuPath: string = '/';
  @State feiniuFiles: NasFile[] = [];
  @State feiniuLoading: boolean = false;
  // 协议
  @State protocolSources: ProtocolSourceDto[] = [];
  @State selectedSourceId: string = '';
  @State protocolPath: string = '/';
  @State protocolEntries: string[] = [];
  @State protocolLoading: boolean = false;
  @State showAddSheet: boolean = false;
  @State newSourceJson: string = '';
  // 通用提示
  @State toastMsg: string = '';

  build() {
    Column() {
      Text('NAS / 协议音源').fontSize(20).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_primary'))
      Text('飞牛 NAS 与网络协议源浏览播放').fontSize(12)
        .fontColor($r('app.color.ngh_text_secondary')).margin({ top: 2 })

      // 模式切换
      Row({ space: 8 }) {
        this.modeChip('飞牛 NAS', 0)
        this.modeChip('协议源', 1)
      }
      .margin({ top: 12 })

      Scroll() {
        Column() {
          if (this.mode === 0) {
            this.feiniuPanel()
          } else {
            this.protocolPanel()
          }
        }
      }
      .layoutWeight(1).margin({ top: 12 })
    }
    .width('100%').height('100%').padding(16)
    .backgroundColor($r('app.color.ngh_background'))
    .bindPopup(this.toastMsg.length > 0, {
      message: this.toastMsg,
      placementOnTop: false,
      onStateChange: (e: boolean) => { if (!e) this.toastMsg = ''; }
    })
  }

  @Builder
  modeChip(label: string, idx: number) {
    Text(label)
      .fontSize(12)
      .layoutWeight(1).textAlign(TextAlign.Center)
      .padding({ top: 6, bottom: 6 })
      .fontColor(idx === this.mode ? $r('app.color.ngh_surface') : $r('app.color.ngh_text_primary'))
      .backgroundColor(idx === this.mode ? $r('app.color.ngh_primary') : $r('app.color.ngh_surface_alt'))
      .borderRadius(8)
      .onClick(() => {
        this.mode = idx;
        if (idx === 1) this.loadProtocolSources();
      })
  }

  // ===================== 飞牛 NAS =====================

  @Builder
  feiniuPanel() {
    Column({ space: 12 }) {
      // 健康检查
      Row({ space: 8 }) {
        SymbolGlyph(this.healthSymbol())
          .fontSize(16)
          .fontColor([this.healthOk === 1 ? $r('app.color.ngh_primary') : $r('app.color.ngh_text_tertiary')])
        Text(this.healthText).fontSize(13).fontColor($r('app.color.ngh_text_secondary'))
        Blank()
        Button('健康检查', { type: ButtonType.Capsule, stateEffect: true })
          .fontSize(12)
          .backgroundColor($r('app.color.ngh_primary'))
          .fontColor($r('app.color.ngh_surface'))
          .onClick(() => this.checkHealth())
      }
      .width('100%').padding(12)
      .backgroundColor($r('app.color.ngh_surface')).borderRadius(12)

      // 登录表单
      Column({ space: 8 }) {
        Text('飞牛登录').fontSize(14).fontWeight(FontWeight.Medium)
          .fontColor($r('app.color.ngh_text_primary'))
        TextInput({ text: this.baseUrl, placeholder: '服务地址，如 https://nas.example.com' })
          .onChange((v: string) => this.baseUrl = v)
          .backgroundColor($r('app.color.ngh_surface')).borderRadius(8)
        TextInput({ text: this.username, placeholder: '用户名' })
          .onChange((v: string) => this.username = v)
          .backgroundColor($r('app.color.ngh_surface')).borderRadius(8)
        TextInput({ text: this.password, placeholder: '密码' })
          .type(InputType.Password)
          .onChange((v: string) => this.password = v)
          .backgroundColor($r('app.color.ngh_surface')).borderRadius(8)
        Button('登录', { type: ButtonType.Capsule, stateEffect: true })
          .width('100%')
          .backgroundColor($r('app.color.ngh_primary'))
          .fontColor($r('app.color.ngh_surface'))
          .onClick(() => this.login())
      }
      .width('100%').padding(12)
      .backgroundColor($r('app.color.ngh_surface')).borderRadius(12)

      this.feiniuBrowser()
    }
  }

  @Builder
  feiniuBrowser() {
    Column({ space: 8 }) {
      Row({ space: 8 }) {
        Button('返回', { type: ButtonType.Capsule, stateEffect: true })
          .fontSize(12)
          .enabled(this.feiniuPath !== '/' && this.feiniuPath.length > 0)
          .onClick(() => {
            this.feiniuPath = this.parentPath(this.feiniuPath);
            this.refreshFeiniuFiles();
          })
        Text(this.feiniuPath).fontSize(12).fontColor($r('app.color.ngh_text_secondary'))
          .layoutWeight(1).maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
        Button('刷新', { type: ButtonType.Capsule, stateEffect: true })
          .fontSize(12)
          .onClick(() => this.refreshFeiniuFiles())
      }
      if (this.feiniuLoading) {
        Loading().width(24).height(24).margin({ top: 8, bottom: 8 })
      }
      if (this.feiniuFiles.length === 0 && !this.feiniuLoading) {
        Text('未列出文件，登录后刷新').fontSize(12)
          .fontColor($r('app.color.ngh_text_tertiary'))
          .padding({ top: 16, bottom: 16 })
      } else {
        ForEach(this.feiniuFiles, (f: NasFile) => {
          this.fileRow(f.name, f.is_dir, f.size, () => this.handleFeiniuTap(f))
        }, (f: NasFile) => f.name)
      }
    }
    .width('100%').padding(12)
    .backgroundColor($r('app.color.ngh_surface')).borderRadius(12)
  }

  private healthSymbol(): Resource {
    if (this.healthOk === 1) return $r('sys.symbol.checkmark_seal_fill');
    if (this.healthOk === 0) return $r('sys.symbol.xmark_octagon_fill');
    return $r('sys.symbol.questionmark_circle');
  }

  private checkHealth(): void {
    this.healthText = '检测中…';
    this.healthOk = -1;
    this.retrySync(() => MusicCoreBridge.feiniuHealth(), (raw: string) => {
      try {
        const obj = JSON.parse(raw) as Record<string, Object>;
        const ok = Boolean(obj['healthy']);
        this.healthOk = ok ? 1 : 0;
        this.healthText = ok ? '飞牛服务可达' : '飞牛服务不可达';
      } catch (e) {
        this.healthOk = 0;
        this.healthText = '健康检查失败';
        this.showToast(this.formatError(e));
      }
    }, (e: Error) => {
      this.healthOk = 0;
      this.healthText = '健康检查失败';
      this.showToast(this.formatError(e));
    });
  }

  private login(): void {
    const b = this.baseUrl.trim();
    const u = this.username.trim();
    if (b.length === 0 || u.length === 0) {
      this.showToast('请填写服务地址与用户名');
      return;
    }
    this.feiniuLoading = true;
    try {
      const raw = MusicCoreBridge.feiniuLogin(b, u, this.password);
      if (raw === null) {
        this.showToast('登录失败：核心未链接');
      } else {
        this.feiniuPath = '/';
        this.refreshFeiniuFiles();
        this.showToast('登录成功');
      }
    } catch (e) {
      const msg = (e as Error)?.message ?? String(e);
      if (msg.includes('401')) {
        this.showToast('用户名或密码错误（401），请检查凭据后重试。');
      } else {
        this.showToast('登录失败：' + this.formatError(e));
      }
    } finally {
      this.feiniuLoading = false;
    }
  }

  private refreshFeiniuFiles(): void {
    this.feiniuLoading = true;
    this.feiniuFiles = [];
    this.retrySync(() => MusicCoreBridge.feiniuListFiles(this.feiniuPath), (raw: string) => {
      try {
        const obj = JSON.parse(raw) as Record<string, Object>;
        const files = (obj['files'] ?? []) as NasFile[];
        this.feiniuFiles = files.slice().sort((a: NasFile, b: NasFile) => {
          if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
          return a.name.localeCompare(b.name);
        });
      } catch (e) {
        this.showToast('列目录失败：' + this.formatError(e));
      }
    }, (e: Error) => {
      this.showToast('列目录失败：' + this.formatError(e));
    });
    this.feiniuLoading = false;
  }

  private handleFeiniuTap(f: NasFile): void {
    if (f.is_dir) {
      this.feiniuPath = this.joinPath(this.feiniuPath, f.name, true);
      this.refreshFeiniuFiles();
      return;
    }
    if (!this.isAudio(f.name)) return;
    this.feiniuLoading = true;
    const full = this.joinPath(this.feiniuPath, f.name, false);
    this.retrySync(() => MusicCoreBridge.feiniuStream(full), (raw: string) => {
      try {
        const obj = JSON.parse(raw) as Record<string, Object>;
        const url = String(obj['url'] ?? '');
        if (url.length === 0) { this.showToast('未获取到播放地址'); return; }
        playerManager.playSong({
          id: 'feiniu-' + f.name,
          title: f.name,
          artist: '—',
          playUrl: url,
        });
        this.showToast('开始播放：' + f.name);
      } catch (e) {
        this.showToast('获取播放地址失败：' + this.formatError(e));
      }
    }, (e: Error) => {
      this.showToast('获取播放地址失败：' + this.formatError(e));
    });
    this.feiniuLoading = false;
  }

  // ===================== 协议源 =====================

  @Builder
  protocolPanel() {
    Column({ space: 12 }) {
      // 协议源管理
      Column({ space: 8 }) {
        Row({ space: 8 }) {
          Text('协议源').fontSize(14).fontWeight(FontWeight.Medium)
            .fontColor($r('app.color.ngh_text_primary')).layoutWeight(1)
          Button('刷新', { type: ButtonType.Capsule, stateEffect: true })
            .fontSize(12)
            .onClick(() => this.loadProtocolSources())
          Button('添加', { type: ButtonType.Capsule, stateEffect: true })
            .fontSize(12)
            .backgroundColor($r('app.color.ngh_primary'))
            .fontColor($r('app.color.ngh_surface'))
            .onClick(() => this.showAddSheet = true)
        }
        if (this.protocolSources.length === 0) {
          Text('尚无协议源，点击「添加」创建（WebDAV / FTP 可用，SMB / DLNA / NFS 为占位）')
            .fontSize(11).fontColor($r('app.color.ngh_text_tertiary'))
        }
        ForEach(this.protocolSources, (s: ProtocolSourceDto) => {
          this.sourceRow(s)
        }, (s: ProtocolSourceDto) => s.id)
      }
      .width('100%').padding(12)
      .backgroundColor($r('app.color.ngh_surface')).borderRadius(12)

      // 浏览区
      if (this.selectedSourceId.length > 0) {
        this.protocolBrowser()
      }
    }
  }

  @Builder
  sourceRow(s: ProtocolSourceDto) {
    Row({ space: 8 }) {
      Column({ space: 2 }) {
        Text(`${s.protocol} · ${s.id}`).fontSize(13).fontWeight(FontWeight.Medium)
          .fontColor($r('app.color.ngh_text_primary'))
        if (s.placeholder) {
          Text('占位实现，浏览不可用').fontSize(11).fontColor($r('app.color.ngh_warning'))
        } else {
          Text(s.root.length === 0 ? '/' : s.root).fontSize(11)
            .fontColor($r('app.color.ngh_text_tertiary'))
        }
      }.layoutWeight(1).alignItems(HorizontalAlign.Start)
      Button('浏览', { type: ButtonType.Capsule, stateEffect: true })
        .fontSize(12)
        .onClick(() => {
          this.selectedSourceId = s.id;
          if (!s.placeholder) {
            this.protocolPath = s.root.length === 0 ? '/' : s.root;
            this.refreshProtocolEntries();
          }
        })
      Button('删除', { type: ButtonType.Capsule, stateEffect: true })
        .fontSize(12)
        .fontColor($r('app.color.ngh_danger'))
        .onClick(() => this.deleteSource(s.id))
    }
    .width('100%').padding({ top: 8, bottom: 8 })
  }

  @Builder
  protocolBrowser() {
    Column({ space: 8 }) {
      Text(`当前协议源：${this.selectedSource()?.protocol ?? ''} · ${this.selectedSourceId}`)
        .fontSize(13).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_primary'))
      if (this.selectedSource()?.placeholder) {
        Text(`${this.selectedSource()?.protocol ?? ''} 为占位实现，需启用对应 feature，建议使用 WebDAV / FTP`)
          .fontSize(11).fontColor($r('app.color.ngh_warning'))
      } else {
        Row({ space: 8 }) {
          Button('返回', { type: ButtonType.Capsule, stateEffect: true })
            .fontSize(12)
            .enabled(this.protocolPath !== '/' && this.protocolPath.length > 0)
            .onClick(() => {
              this.protocolPath = this.parentPath(this.protocolPath);
              this.refreshProtocolEntries();
            })
          Text(this.protocolPath).fontSize(12).fontColor($r('app.color.ngh_text_secondary'))
            .layoutWeight(1).maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
          Button('刷新', { type: ButtonType.Capsule, stateEffect: true })
            .fontSize(12)
            .onClick(() => this.refreshProtocolEntries())
        }
        if (this.protocolLoading) {
          Loading().width(24).height(24).margin({ top: 8, bottom: 8 })
        }
        if (this.protocolEntries.length === 0 && !this.protocolLoading) {
          Text('未列出条目，点击刷新').fontSize(12)
            .fontColor($r('app.color.ngh_text_tertiary'))
            .padding({ top: 16, bottom: 16 })
        } else {
          ForEach(this.protocolEntries, (rawName: string) => {
            const isDir = rawName.endsWith('/');
            const name = isDir ? rawName.substring(0, rawName.length - 1) : rawName;
            this.fileRow(name, isDir, 0, () => this.handleProtocolTap(name, isDir))
          }, (rawName: string) => rawName)
        }
      }
    }
    .width('100%').padding(12)
    .backgroundColor($r('app.color.ngh_surface')).borderRadius(12)
  }

  private selectedSource(): ProtocolSourceDto | undefined {
    return this.protocolSources.find((s: ProtocolSourceDto) => s.id === this.selectedSourceId);
  }

  private loadProtocolSources(): void {
    try {
      const raw = MusicCoreBridge.protocolList();
      if (raw === null) { this.protocolSources = []; return; }
      const obj = JSON.parse(raw) as Record<string, Object>;
      const arr = (obj['sources'] ?? []) as ProtocolSourceDto[];
      this.protocolSources = arr;
    } catch (e) {
      this.protocolSources = [];
      this.showToast('加载协议源失败：' + this.formatError(e));
    }
  }

  private refreshProtocolEntries(): void {
    if (this.selectedSourceId.length === 0) return;
    this.protocolLoading = true;
    this.protocolEntries = [];
    this.retrySync(() => MusicCoreBridge.protocolListFiles(this.selectedSourceId, this.protocolPath), (raw: string) => {
      try {
        const obj = JSON.parse(raw) as Record<string, Object>;
        const entries = (obj['entries'] ?? []) as string[];
        this.protocolEntries = entries.slice().sort((a: string, b: string) => {
          const aDir = a.endsWith('/');
          const bDir = b.endsWith('/');
          if (aDir !== bDir) return aDir ? -1 : 1;
          return a.localeCompare(b);
        });
      } catch (e) {
        this.showToast('浏览失败：' + this.formatError(e));
      }
    }, (e: Error) => {
      this.showToast('浏览失败：' + this.formatError(e));
    });
    this.protocolLoading = false;
  }

  private handleProtocolTap(name: string, isDir: boolean): void {
    if (isDir) {
      this.protocolPath = this.joinPath(this.protocolPath, name, true);
      this.refreshProtocolEntries();
      return;
    }
    if (!this.isAudio(name)) return;
    this.protocolLoading = true;
    const full = this.joinPath(this.protocolPath, name, false);
    this.retrySync(() => MusicCoreBridge.protocolStream(this.selectedSourceId, full), (raw: string) => {
      try {
        const obj = JSON.parse(raw) as Record<string, Object>;
        const url = String(obj['url'] ?? '');
        if (url.length === 0) { this.showToast('未获取到播放地址'); return; }
        playerManager.playSong({
          id: 'proto-' + name,
          title: name,
          artist: '—',
          playUrl: url,
        });
        this.showToast('开始播放：' + name);
      } catch (e) {
        this.showToast('获取播放地址失败：' + this.formatError(e));
      }
    }, (e: Error) => {
      this.showToast('获取播放地址失败：' + this.formatError(e));
    });
    this.protocolLoading = false;
  }

  private deleteSource(id: string): void {
    try {
      MusicCoreBridge.protocolDelete(id);
      if (this.selectedSourceId === id) this.selectedSourceId = '';
      this.loadProtocolSources();
      this.showToast('已删除协议源 ' + id);
    } catch (e) {
      this.showToast('删除失败：' + this.formatError(e));
    }
  }

  private addSource(): void {
    const json = this.newSourceJson.trim();
    if (json.length === 0) { this.showToast('请填写协议源配置 JSON'); return; }
    try {
      MusicCoreBridge.protocolAdd(json);
      this.showAddSheet = false;
      this.newSourceJson = '';
      this.loadProtocolSources();
      this.showToast('协议源已添加');
    } catch (e) {
      this.showToast('添加协议源失败：' + this.formatError(e));
    }
  }

  // ===================== 通用 Builder =====================

  @Builder
  fileRow(name: string, isDir: boolean, size: number, onTap: () => void) {
    Row({ space: 12 }) {
      SymbolGlyph(isDir ? $r('sys.symbol.folder') : $r('sys.symbol.music_note'))
        .fontSize(16)
        .fontColor([$r('app.color.ngh_primary')])
      Text(name).fontSize(13).fontColor($r('app.color.ngh_text_primary'))
        .layoutWeight(1).maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
      if (!isDir && size > 0) {
        Text(this.formatBytes(size)).fontSize(11)
          .fontColor($r('app.color.ngh_text_tertiary'))
      }
    }
    .width('100%').padding({ top: 10, bottom: 10, left: 4, right: 4 })
    .borderRadius(8)
    .onClick(() => onTap())
  }

  // ===================== 重试 / 错误处理 =====================

  /**
   * 对 502/504 类网络错误做指数退避重试（1s/2s/4s，最多 3 次）。
   * 注意：HarmonyOS 桥接为同步调用，本方法在 502/504 时同步等待（scaffold 阶段）；
   * 真正异步化需 NAPI Promise 化。401/404/501 不重试，直接进入 onError。
   */
  private retrySync(
    fn: () => string | null,
    onSuccess: (raw: string) => void,
    onError: (e: Error) => void
  ): void {
    const delays = [1000, 2000, 4000];
    let last: Error | null = null;
    for (let attempt = 0; attempt <= 3; attempt++) {
      try {
        const raw = fn();
        if (raw === null) {
          // 核心未链接 / 返回空，按成功但无数据处理
          onSuccess('null');
          return;
        }
        // 检查响应是否为错误对象且可重试
        if (this.isRetryableErrorJson(raw)) {
          last = new Error(this.extractErrorMessage(raw));
          if (attempt === 3) break;
          // 同步等待（scaffold 阶段；真实环境应异步）
          const start = Date.now();
          while (Date.now() - start < delays[attempt]) { /* busy wait */ }
          continue;
        }
        onSuccess(raw);
        return;
      } catch (e) {
        last = e as Error;
        if (!this.isRetryable(last) || attempt === 3) break;
        const start = Date.now();
        while (Date.now() - start < delays[attempt]) { /* busy wait */ }
      }
    }
    if (last) onError(last);
    else onSuccess('null');
  }

  private isRetryable(e: Error): boolean {
    const msg = e.message ?? '';
    return msg.includes('502') || msg.includes('504') ||
           msg.includes('不可达') || msg.includes('请求失败');
  }

  private isRetryableErrorJson(raw: string): boolean {
    try {
      const obj = JSON.parse(raw) as Record<string, Object>;
      const err = obj['error'] as Record<string, Object> | undefined;
      if (!err) return false;
      const msg = String(err['message'] ?? '');
      return msg.includes('502') || msg.includes('504');
    } catch (e) {
      return false;
    }
  }

  private extractErrorMessage(raw: string): string {
    try {
      const obj = JSON.parse(raw) as Record<string, Object>;
      const err = obj['error'] as Record<string, Object> | undefined;
      if (err) return String(err['message'] ?? raw);
    } catch (e) { /* ignore */ }
    return raw;
  }

  private formatError(e: Object): string {
    const msg = (e as Error)?.message ?? String(e);
    if (msg.includes('401')) return msg + '\n提示：未登录或 token 失效，请重新登录。';
    if (msg.includes('404')) return msg + '\n提示：路径不存在，请修正路径。';
    if (msg.includes('501')) return msg + '\n提示：该协议为占位实现，请使用 WebDAV / FTP 或启用对应 feature。';
    return msg;
  }

  private showToast(msg: string): void {
    this.toastMsg = msg;
  }

  // ===================== 路径与格式工具 =====================

  private joinPath(parent: string, name: string, isDir: boolean): string {
    const p = parent.endsWith('/') ? parent.substring(0, parent.length - 1) : parent;
    const n = name.startsWith('/') ? name.substring(1) : name;
    let full = p.length === 0 ? '/' + n : p + '/' + n;
    if (isDir && !full.endsWith('/')) full += '/';
    return full;
  }

  private parentPath(path: string): string {
    if (path.length === 0 || path === '/') return '/';
    let p = path.endsWith('/') ? path.substring(0, path.length - 1) : path;
    if (p.length === 0) return '/';
    const idx = p.lastIndexOf('/');
    if (idx < 0) return '/';
    if (idx === 0) return '/';
    return p.substring(0, idx);
  }

  private isAudio(name: string): boolean {
    const exts = ['.mp3', '.flac', '.wav', '.m4a', '.aac', '.ogg', '.opus', '.wma'];
    const lower = name.toLowerCase();
    return exts.some((ext: string) => lower.endsWith(ext));
  }

  private formatBytes(bytes: number): string {
    const units = ['B', 'KB', 'MB', 'GB', 'TB'];
    let v = bytes;
    let i = 0;
    while (v >= 1024 && i < units.length - 1) { v /= 1024; i++; }
    return v.toFixed(2) + ' ' + units[i];
  }
}
