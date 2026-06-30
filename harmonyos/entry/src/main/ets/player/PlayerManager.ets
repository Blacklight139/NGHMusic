// 职责：封装 @ohos.multimedia.avplayer 的 AVPlayer，提供 load/play/pause/seek/setVolume/setLoop，
//       状态通过 AppStorage 暴露（currentSong/position/duration/isPlaying/volume/mode）。
// 集成方式：在 EntryAbility/页面中通过 AppStorage 与 PlayerManager 单例交互。
//
// 说明：AVPlayer 是 HarmonyOS 推荐的音频播放组件，状态机驱动（idle→initialized→prepared→
//       playing→paused→stopped→released）。本封装监听 stateChange/positionUpdate/complete 回调。

import media from '@ohos.multimedia.media';

// 播放模式枚举（对应 Rust PlayMode）
export enum PlayMode { SEQUENTIAL = 0, SINGLE_LOOP = 1, RANDOM = 2 }

interface PlayerStatus {
  currentTitle: string;
  currentArtist: string;
  position: number;   // 毫秒
  duration: number;   // 毫秒
  isPlaying: boolean;
  volume: number;     // 0~1
  mode: PlayMode;
}

const STORAGE_KEY = 'player_status';

export class PlayerManager {
  private avPlayer: media.AVPlayer | null = null;
  private queue: Array<{ title: string; artist: string; playUrl?: string }> = [];
  private index: number = 0;

  private status: PlayerStatus = {
    currentTitle: '未在播放',
    currentArtist: '—',
    position: 0,
    duration: 0,
    isPlaying: false,
    volume: 0.8,
    mode: PlayMode.SEQUENTIAL,
  };

  constructor() {
    AppStorage.setOrCreate<PlayerStatus>(STORAGE_KEY, this.status);
  }

  private async ensurePlayer(): Promise<media.AVPlayer> {
    if (this.avPlayer) {
      return this.avPlayer;
    }
    const player = await media.createAVPlayer();
    player.on('stateChange', (state: string) => {
      if (state === 'playing') {
        this.update({ isPlaying: true });
      } else if (state === 'paused' || state === 'stopped') {
        this.update({ isPlaying: false });
      } else if (state === 'error') {
        console.error('[PlayerManager] AVPlayer 进入 error 状态');
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
  }

  // 加载并播放指定 URL
  async load(url: string, title: string = '未知曲目', artist: string = '—'): Promise<void> {
    const player = await this.ensurePlayer();
    this.update({ currentTitle: title, currentArtist: artist, position: 0 });
    player.url = url;       // 在线/本地 URL；赋值后 AVPlayer 进入 initialized
    player.volume = this.status.volume;
  }

  async play(): Promise<void> {
    const player = await this.ensurePlayer();
    player.play();
  }

  async pause(): Promise<void> {
    if (this.avPlayer) {
      this.avPlayer.pause();
    }
  }

  async resume(): Promise<void> {
    await this.play();
  }

  async seek(positionMs: number): Promise<void> {
    if (this.avPlayer) {
      this.avPlayer.seek(positionMs);
      this.update({ position: positionMs });
    }
  }

  async setVolume(v: number): Promise<void> {
    const vol = Math.max(0, Math.min(1, v));
    this.update({ volume: vol });
    if (this.avPlayer) {
      this.avPlayer.volume = vol;
    }
  }

  setMode(mode: PlayMode): void {
    this.update({ mode });
    if (this.avPlayer) {
      this.avPlayer.loop = (mode === PlayMode.SINGLE_LOOP);
    }
  }

  toggleMode(): void {
    const next = (this.status.mode + 1) % 3;
    this.setMode(next as PlayMode);
  }

  // 队列与上一首/下一首（占位：仅切换标题，实际需解析 play_url）
  setQueue(songs: Array<{ title: string; artist: string; playUrl?: string }>): void {
    this.queue = songs;
    this.index = 0;
  }

  async toNext(): Promise<void> {
    if (this.queue.length === 0) {
      return;
    }
    switch (this.status.mode) {
      case PlayMode.SINGLE_LOOP:
        if (this.avPlayer) {
          this.avPlayer.seek(0);
          await this.play();
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

  private async playAt(i: number): Promise<void> {
    const song = this.queue[i];
    if (song && song.playUrl) {
      await this.load(song.playUrl, song.title, song.artist);
    } else {
      this.update({ currentTitle: song.title, currentArtist: song.artist });
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
