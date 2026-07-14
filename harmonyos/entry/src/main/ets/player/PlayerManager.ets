// 职责：封装 @ohos.multimedia.media 的 AVPlayer，提供 load/play/pause/seek/setVolume/setLoop，
//       状态通过 AppStorage 暴露（currentSong/position/duration/isPlaying/volume/mode）。
// 集成方式：在 EntryAbility/页面中通过 AppStorage 与 PlayerManager 单例交互。
//
// 说明：AVPlayer 是 HarmonyOS 推荐的音频播放组件，状态机驱动（idle→initialized→prepared→
//       playing→paused→stopped→released）。本封装监听 stateChange/positionUpdate/complete 回调。
//
// Bug 修复（见 docs/bug-report.md）：
//   - HAR-001（状态不一致）：playAt 在 playUrl 为空时不再「更新标题如同正在播放」，
//     而是记录错误并跳过 load，避免 UI 误显示「正在播放」。
//   - HAR-002（未处理异常）：ensurePlayer/load/play 等异步方法均包裹 try-catch，
//     media.createAVPlayer() 抛错时不再冒泡为未处理 rejection。
//   - HAR-004（资源）：新增显式 release() 方法，调用 avPlayer.release() 进入 released 状态。
//   - HAR-005（状态机，播放不可用）：load 仅赋 url 进入 initialized 后直接返回，从未调用 prepare()，
//     AVPlayer 不会进入 prepared，后续 play() 无效。改为：赋 url 后调用 prepare() 并等待 prepared 状态；
//     若 AVPlayer 已处于 prepared/playing/paused/completed/stopped（持有旧源），先 reset() 回 idle 再赋新 url，
//     避免在非 idle 状态覆写 url 导致状态机异常。
//   - HAR-006（竞态，资源泄漏）：ensurePlayer 两次并发调用会各自 await media.createAVPlayer()，
//     创建两个 AVPlayer 实例，先创建者被覆写后泄漏且其回调永不注销。改为以 in-flight Promise 缓存，
//     并发调用复用同一次创建结果。

import media from '@ohos.multimedia.media';
import { PlayMode } from '../models/Models';

// 重新导出 PlayMode 以兼容现有页面 `import { PlayMode } from '../player/PlayerManager'` 用法
export { PlayMode } from '../models/Models';

interface PlayerStatus {
  currentTitle: string;
  currentArtist: string;
  currentSongId: string;     // 当前歌曲 id（HAR-001 修复后用于判断是否有可播放曲目）
  position: number;          // 毫秒
  duration: number;          // 毫秒
  isPlaying: boolean;
  volume: number;            // 0~1
  mode: PlayMode;
  hasPlayable: boolean;      // 当前是否持有可播放曲目（HAR-001：避免「假播放」UI）
}

const STORAGE_KEY = 'player_status';

// 队列条目：仅承载播放所需的最小字段；UI 完整 Song 信息由调用方维护
interface QueueItem {
  id: string;
  title: string;
  artist: string;
  playUrl?: string;          // 为空 / undefined 表示无可播放 URL
}

export class PlayerManager {
  private avPlayer: media.AVPlayer | null = null;
  // HAR-006：in-flight Promise 缓存，并发 ensurePlayer 调用复用同一次 createAVPlayer 结果。
  private ensurePlayerPromise: Promise<media.AVPlayer | null> | null = null;
  // HAR-005：等待 AVPlayer 状态机推进到目标状态的临时回调（reset→idle / prepare→prepared）。
  private stateWaiter: ((state: string) => void) | null = null;
  private queue: QueueItem[] = [];
  private index: number = 0;

  private status: PlayerStatus = {
    currentTitle: '未在播放',
    currentArtist: '—',
    currentSongId: '',
    position: 0,
    duration: 0,
    isPlaying: false,
    volume: 0.8,
    mode: PlayMode.SEQUENTIAL,
    hasPlayable: false,
  };

  constructor() {
    AppStorage.setOrCreate<PlayerStatus>(STORAGE_KEY, this.status);
  }

  /**
   * 确保 AVPlayer 已创建。HAR-002：包裹 try-catch，避免 createAVPlayer 抛错冒泡。
   * HAR-006 修复：并发调用复用同一 in-flight Promise，避免创建多个 AVPlayer 实例造成泄漏。
   */
  private ensurePlayer(): Promise<media.AVPlayer | null> {
    if (this.avPlayer) {
      return Promise.resolve(this.avPlayer);
    }
    if (this.ensurePlayerPromise) {
      return this.ensurePlayerPromise;
    }
    this.ensurePlayerPromise = this.createPlayer();
    return this.ensurePlayerPromise;
  }

  private async createPlayer(): Promise<media.AVPlayer | null> {
    try {
      const player = await media.createAVPlayer();
      player.on('stateChange', (state: string) => {
        if (state === 'playing') {
          this.update({ isPlaying: true });
        } else if (state === 'paused' || state === 'stopped') {
          this.update({ isPlaying: false });
        } else if (state === 'error') {
          console.error('[PlayerManager] AVPlayer 进入 error 状态');
          this.update({ isPlaying: false });
        }
        // HAR-005：通知等待者状态机已推进（reset→idle / prepare→prepared）。
        const waiter = this.stateWaiter;
        if (waiter) {
          waiter(state);
        }
      });
      player.on('durationUpdate', (duration: number) => {
        this.update({ duration });
      });
      player.on('timeUpdate', (positionMs: number) => {
        this.update({ position: positionMs });
      });
      player.on('complete', () => {
        this.toNext();
      });
      this.avPlayer = player;
      return player;
    } catch (e) {
      console.error(`[PlayerManager] createAVPlayer 失败: ${(e as Error).message ?? e}`);
      return null;
    } finally {
      // 创建完成（成功或失败）后清空缓存，允许后续重试。
      this.ensurePlayerPromise = null;
    }
  }

  /**
   * 等待 AVPlayer 状态机推进到 target 状态。HAR-005 辅助：load 中等待 reset→idle 与 prepare→prepared。
   * 超时返回 false（不阻塞调用方），由调用方决定是否继续。
   */
  private waitForState(target: string, timeoutMs: number = 5000): Promise<boolean> {
    return new Promise<boolean>((resolve) => {
      let done = false;
      const timer = setTimeout(() => {
        if (!done) {
          done = true;
          this.stateWaiter = null;
          console.warn(`[PlayerManager] waitForState('${target}') 超时`);
          resolve(false);
        }
      }, timeoutMs);
      this.stateWaiter = (state: string) => {
        if (state === target && !done) {
          done = true;
          clearTimeout(timer);
          this.stateWaiter = null;
          resolve(true);
        }
      };
    });
  }

  /**
   * 加载并播放指定 URL。HAR-002：包裹 try-catch。
   * url 为空字符串时直接返回 false，不修改任何播放状态（HAR-001 的底层守卫）。
   * HAR-005 修复：赋 url 后必须 prepare() 并等待 prepared，否则 play() 无效；
   *   若 AVPlayer 已持有旧源（非 idle/initialized/error），先 reset() 回 idle 再赋新 url。
   */
  async load(url: string, title: string = '未知曲目', artist: string = '—', songId: string = ''): Promise<boolean> {
    if (url === undefined || url === null || url.length === 0) {
      // HAR-001 守卫：不在此处改标题；调用方 playAt 决定 UI 处理
      return false;
    }
    const player = await this.ensurePlayer();
    if (!player) {
      return false;
    }
    try {
      this.update({
        currentTitle: title,
        currentArtist: artist,
        currentSongId: songId,
        position: 0,
        hasPlayable: true,
      });
      // HAR-005：若 AVPlayer 已处于持有旧源的状态（prepared/playing/paused/completed/stopped），
      // 直接覆写 url 会破坏状态机；先 reset() 回 idle，等待 idle 后再赋新 url。
      const cur = player.state;
      if (cur && cur !== 'idle' && cur !== 'initialized' && cur !== 'error') {
        player.reset();
        await this.waitForState('idle');
      }
      player.url = url;          // 在线/本地 URL；赋值后 AVPlayer 进入 initialized
      player.volume = this.status.volume;
      player.loop = (this.status.mode === PlayMode.SINGLE_LOOP);
      player.prepare();          // HAR-005：进入 prepared 状态，play() 才能生效
      await this.waitForState('prepared');
      return true;
    } catch (e) {
      console.error(`[PlayerManager] load 失败: ${(e as Error).message ?? e}`);
      this.update({ hasPlayable: false, isPlaying: false });
      return false;
    }
  }

  /**
   * 触发播放。HAR-002：包裹 try-catch。
   */
  async play(): Promise<void> {
    const player = await this.ensurePlayer();
    if (!player) {
      return;
    }
    try {
      player.play();
    } catch (e) {
      console.error(`[PlayerManager] play 失败: ${(e as Error).message ?? e}`);
    }
  }

  async pause(): Promise<void> {
    if (!this.avPlayer) {
      return;
    }
    try {
      this.avPlayer.pause();
    } catch (e) {
      console.error(`[PlayerManager] pause 失败: ${(e as Error).message ?? e}`);
    }
  }

  async resume(): Promise<void> {
    await this.play();
  }

  async seek(positionMs: number): Promise<void> {
    if (!this.avPlayer) {
      return;
    }
    try {
      this.avPlayer.seek(positionMs);
      this.update({ position: positionMs });
    } catch (e) {
      console.error(`[PlayerManager] seek 失败: ${(e as Error).message ?? e}`);
    }
  }

  async setVolume(v: number): Promise<void> {
    const vol = Math.max(0, Math.min(1, v));
    this.update({ volume: vol });
    if (this.avPlayer) {
      try {
        this.avPlayer.volume = vol;
      } catch (e) {
        console.error(`[PlayerManager] setVolume 失败: ${(e as Error).message ?? e}`);
      }
    }
  }

  setMode(mode: PlayMode): void {
    this.update({ mode });
    if (this.avPlayer) {
      try {
        this.avPlayer.loop = (mode === PlayMode.SINGLE_LOOP);
      } catch (e) {
        console.error(`[PlayerManager] setMode loop 失败: ${(e as Error).message ?? e}`);
      }
    }
  }

  toggleMode(): void {
    const next = (this.status.mode + 1) % 3;
    this.setMode(next as PlayMode);
  }

  /**
   * 设置播放队列。songs 字段映射自 Song 模型（playUrl 为 Rust snake_case `play_url`，由调用方传入）。
   */
  setQueue(songs: QueueItem[]): void {
    this.queue = songs;
    this.index = 0;
  }

  getQueueLength(): number {
    return this.queue.length;
  }

  async toNext(): Promise<void> {
    if (this.queue.length === 0) {
      return;
    }
    switch (this.status.mode) {
      case PlayMode.SINGLE_LOOP:
        if (this.avPlayer) {
          try {
            this.avPlayer.seek(0);
            await this.play();
          } catch (e) {
            console.error(`[PlayerManager] toNext(SINGLE_LOOP) 失败: ${(e as Error).message ?? e}`);
          }
        }
        break;
      case PlayMode.RANDOM:
        this.index = Math.floor(Math.random() * this.queue.length);
        await this.playAt(this.index);
        break;
      case PlayMode.SEQUENTIAL:
      default:
        this.index = (this.index + 1) % this.queue.length;
        await this.playAt(this.index);
        break;
    }
  }

  async toPrev(): Promise<void> {
    if (this.queue.length === 0) {
      return;
    }
    this.index = (this.index - 1 + this.queue.length) % this.queue.length;
    await this.playAt(this.index);
  }

  /**
   * 播放队列中指定索引的歌曲。HAR-001 修复核心：
   * 当 song.playUrl 为空 / undefined 时，不更新标题（避免「假播放」UI），
   * 仅记录错误并直接返回，UI 维持原状态（currentTitle/isPlaying 不变）。
   */
  private async playAt(i: number): Promise<void> {
    const song = this.queue[i];
    if (!song) {
      return;
    }
    const url = song.playUrl ?? '';
    if (url.length === 0) {
      // HAR-001 修复：playUrl 为空时不进入「假播放」状态
      console.warn(`[PlayerManager] playAt: 歌曲「${song.title}」(id=${song.id}) 无可播放 URL，已跳过`);
      // 不调用 load，不更新 currentTitle/isPlaying，UI 不会误显示「正在播放」
      return;
    }
    const ok = await this.load(url, song.title, song.artist, song.id);
    if (ok) {
      await this.play();
    }
  }

  /**
   * 显式播放指定歌曲（外部触发，例如点击搜索结果）。
   * 若该歌曲已在队列中，则定位到对应索引；否则临时插入队列并播放。
   * HAR-001：playUrl 为空时不进入「假播放」状态。
   */
  async playSong(item: QueueItem): Promise<void> {
    const idx = this.queue.findIndex((s: QueueItem) => s.id === item.id);
    if (idx >= 0) {
      this.index = idx;
    } else {
      this.queue.push(item);
      this.index = this.queue.length - 1;
    }
    await this.playAt(this.index);
  }

  /**
   * HAR-004 修复：显式释放 AVPlayer，进入 released 状态。
   * 在 EntryAbility onDestroy / 页面 aboutToDisappear 时调用，避免依赖 GC。
   */
  async release(): Promise<void> {
    if (!this.avPlayer) {
      return;
    }
    try {
      await this.avPlayer.release();
    } catch (e) {
      console.error(`[PlayerManager] release 失败: ${(e as Error).message ?? e}`);
    } finally {
      this.avPlayer = null;
      // HAR-005/HAR-006：释放时清理等待者与 in-flight 创建缓存，避免悬空回调 / 旧 Promise 复用。
      this.stateWaiter = null;
      this.ensurePlayerPromise = null;
      this.update({
        isPlaying: false,
        hasPlayable: false,
        position: 0,
        duration: 0,
      });
    }
  }

  private update(patch: Partial<PlayerStatus>): void {
    this.status = { ...this.status, ...patch };
    AppStorage.set<PlayerStatus>(STORAGE_KEY, this.status);
  }

  getStatus(): PlayerStatus {
    return this.status;
  }
}

// 全局单例（供页面通过 AppStorage 或直接引用）
export const playerManager = new PlayerManager();
