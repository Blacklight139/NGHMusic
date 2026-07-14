// 职责：歌词页 ArkUI 组件，逐行展示并高亮当前行（豆包风格 ngh_* token）。
// 对齐桌面端 pages/lyrics.js：与 PlayerManager.position 同步滚动。
// 数据来源：MusicCoreBridge.getLyric(sourceId, songId) 返回 Lyric JSON（snake_case 字段）。
// 占位：未拉取歌词时显示示例行；接入核心后由 currentSongId 驱动加载。

import { playerManager } from '../player/PlayerManager';
import { LyricLine } from '../models/Models';

interface PlayerStatus {
  currentTitle: string;
  currentArtist: string;
  currentSongId: string;
  position: number;
  duration: number;
  isPlaying: boolean;
  volume: number;
  mode: number;
  hasPlayable: boolean;
}

@Component
export struct LyricsPage {
  @State currentIndex: number = 0;
  @State pressedId: string = '';
  // HAR-007 修复：@Watch 监听 playerStatus 变化（position 由 PlayerManager timeUpdate 回调驱动），
  // 在 onStatusChange 中依据进度计算高亮行并滚动；原 onScrollIndex 占位回调从未更新 currentIndex。
  @StorageLink('player_status') @Watch('onStatusChange') playerStatus: PlayerStatus = {
    currentTitle: '未在播放', currentArtist: '—', currentSongId: '',
    position: 0, duration: 0, isPlaying: false, volume: 0.8,
    mode: 0, hasPlayable: false,
  };
  // 改为 @State：接入核心后由 MusicCoreBridge.getLyric 异步填充时可响应式刷新列表。
  @State lines: Array<LyricLine> = [
    { time_ms: 0, text: '示例歌词第一行' },
    { time_ms: 5000, text: '示例歌词第二行' },
    { time_ms: 10000, text: '示例歌词第三行' },
    { time_ms: 15000, text: '示例歌词第四行' }
  ];
  private scroller: Scroller = new Scroller();

  /**
   * HAR-007 修复：playerStatus 变化时按 position 计算当前高亮行索引并滚动。
   * 选取时间戳 <= 当前进度的最后一行；索引未变则跳过，减少不必要的重组与滚动。
   */
  private onStatusChange(): void {
    const pos = this.playerStatus.position;
    let idx = 0;
    for (let i = 0; i < this.lines.length; i++) {
      const t = this.lines[i].time_ms;
      if (t === undefined) {
        continue;
      }
      if (t <= pos) {
        idx = i;
      } else {
        break;
      }
    }
    if (idx !== this.currentIndex) {
      this.currentIndex = idx;
      this.scroller.scrollToIndex(idx);
    }
  }

  build() {
    Column() {
      Text('歌词').fontSize(22).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_primary'))
      if (this.playerStatus.hasPlayable) {
        Text(`${this.playerStatus.currentTitle} — ${this.playerStatus.currentArtist}`)
          .fontSize(12).fontColor($r('app.color.ngh_text_secondary')).margin({ top: 2 })
      } else {
        Text('尚未开始播放').fontSize(12)
          .fontColor($r('app.color.ngh_text_secondary')).margin({ top: 2 })
      }

      List({ scroller: this.scroller }) {
        ForEach(this.lines, (line: LyricLine, i: number) => {
          ListItem() {
            Column() {
              Text(line.text)
                .fontSize(i === this.currentIndex ? 18 : 16)
                .fontWeight(i === this.currentIndex ? FontWeight.Bold : FontWeight.Normal)
                .fontColor(i === this.currentIndex ? $r('app.color.ngh_primary') : $r('app.color.ngh_text_tertiary'))
                .maxLines(1)
                .textOverflow({ overflow: TextOverflow.Ellipsis })
            }
            .alignItems(HorizontalAlign.Start)
            .width('100%')
            .padding({ top: 12, bottom: 12 })
            .onClick(() => {
              if (line.time_ms !== undefined) {
                playerManager.seek(line.time_ms);
              }
            })
            .onTouch((event: TouchEvent) => {
              if (event.type === TouchType.Down) {
                this.pressedId = String(i);
              } else if (event.type === TouchType.Up || event.type === TouchType.Cancel) {
                this.pressedId = '';
              }
            })
            .scale(this.pressedId === String(i) ? { x: 0.98, y: 0.98 } : { x: 1, y: 1 })
            .animation({ duration: 150, curve: Curve.EaseOut })
          }
        }, (line: LyricLine, i: number) => `${i}_${line.text}`)
      }
      .layoutWeight(1)
      .margin({ top: 16, bottom: 12 })
      .padding(16)
      .backgroundColor($r('app.color.ngh_surface'))
      .borderRadius(12)
      .border({ width: 1, color: $r('app.color.ngh_border') })
      .shadow({ radius: 8, color: '#14000000', offsetX: 0, offsetY: 2 })
      .clip(true)
      .divider({ strokeWidth: 0 })
      .onScrollIndex((start: number, end: number) => {
        // HAR-007：高亮行与滚动由 @Watch('onStatusChange') 依据 playerStatus.position 驱动，
        // 此回调仅反映用户手动滚动范围，不用于更新 currentIndex。
      })
    }
    .width('100%').height('100%').padding(16)
    .backgroundColor($r('app.color.ngh_background'))
  }
}
