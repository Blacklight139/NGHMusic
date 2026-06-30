// 职责：歌词页 ArkUI 组件，逐行展示并高亮当前行，简约风格占位。
// 对齐桌面端 pages/lyrics.js：与 PlayerManager.position 同步滚动。

import { playerManager } from '../player/PlayerManager';

interface LyricLine { timeMs: number; text: string }

@Component
export struct LyricsPage {
  @State currentIndex: number = 0;
  private lines: Array<LyricLine> = [
    { timeMs: 0, text: '示例歌词第一行' },
    { timeMs: 5000, text: '示例歌词第二行' },
    { timeMs: 10000, text: '示例歌词第三行' },
    { timeMs: 15000, text: '示例歌词第四行' }
  ];
  private scroller: Scroller = new Scroller();

  build() {
    Column() {
      Text('歌词').fontSize(22).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.text_primary'))
      List({ scroller: this.scroller }) {
        ForEach(this.lines, (line: LyricLine, i: number) => {
          ListItem() {
            Text(line.text)
              .fontSize(i === this.currentIndex ? 18 : 16)
              .fontWeight(i === this.currentIndex ? FontWeight.Bold : FontWeight.Normal)
              .fontColor(i === this.currentIndex ? $r('app.color.text_primary') : $r('app.color.text_muted'))
              .padding({ top: 12, bottom: 12 })
              .onClick(() => playerManager.seek(line.timeMs))
          }
        }, (line: LyricLine) => line.text)
      }
      .layoutWeight(1)
      .margin({ top: 16 })
      .divider({ strokeWidth: 0 })
      .onScrollIndex((start: number, end: number) => {
        // 占位：实际由 PlayerManager position 回调驱动 currentIndex 与 scrollTo
      })
    }
    .width('100%').height('100%').padding(16)
    .backgroundColor($r('app.color.bg_alt'))
  }
}
