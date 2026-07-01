// 职责：歌词页 ArkUI 组件，逐行展示并高亮当前行（豆包风格 ngh_* token）。
// 对齐桌面端 pages/lyrics.js：与 PlayerManager.position 同步滚动。

import { playerManager } from '../player/PlayerManager';

interface LyricLine { timeMs: number; text: string; tr?: string }

@Component
export struct LyricsPage {
  @State currentIndex: number = 0;
  @State pressedId: string = '';
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
        .fontColor($r('app.color.ngh_text_primary'))
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
              if (line.tr !== undefined && line.tr.length > 0) {
                Text(line.tr)
                  .fontSize(12)
                  .fontColor($r('app.color.ngh_text_secondary'))
                  .margin({ top: 4 })
                  .maxLines(1)
                  .textOverflow({ overflow: TextOverflow.Ellipsis })
              }
            }
            .alignItems(HorizontalAlign.Start)
            .width('100%')
            .padding({ top: 12, bottom: 12 })
            .onClick(() => playerManager.seek(line.timeMs))
            .onTouch((event: TouchEvent) => {
              if (event.type === TouchType.Down) {
                this.pressedId = String(line.timeMs);
              } else if (event.type === TouchType.Up || event.type === TouchType.Cancel) {
                this.pressedId = '';
              }
            })
            .scale(this.pressedId === String(line.timeMs) ? { x: 0.98, y: 0.98 } : { x: 1, y: 1 })
            .animation({ duration: 150, curve: Curve.EaseOut })
          }
        }, (line: LyricLine) => line.text)
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
        // 占位：实际由 PlayerManager position 回调驱动 currentIndex 与 scrollTo
      })
    }
    .width('100%').height('100%').padding(16)
    .backgroundColor($r('app.color.ngh_background'))
  }
}
