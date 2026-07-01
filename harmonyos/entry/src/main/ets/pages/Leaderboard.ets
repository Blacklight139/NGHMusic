// 职责：排行榜页 ArkUI 组件，榜单卡片网格（豆包风格 ngh_* token）。

interface BoardItem { id: string; name: string; count: number }

@Component
export struct LeaderboardPage {
  @State boards: Array<BoardItem> = [
    { id: 'l1', name: '热歌榜', count: 0 },
    { id: 'l2', name: '新歌榜', count: 0 },
    { id: 'l3', name: '飙升榜', count: 0 }
  ];
  @State pressedId: string = '';

  build() {
    Column() {
      Text('排行榜').fontSize(22).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_primary'))
      Text('各音源热门榜单').fontSize(12)
        .fontColor($r('app.color.ngh_text_secondary')).margin({ top: 2 })

      if (this.boards.length === 0) {
        this.EmptyState()
      } else {
        Scroll() {
          Grid() {
            ForEach(this.boards, (b: BoardItem) => {
              GridItem() { this.Card(b.id, b.name, `${b.count} 首`) }
            }, (b: BoardItem) => b.id)
          }
          .columnsTemplate('1fr 1fr')
          .columnsGap(12).rowsGap(12)
        }
        .layoutWeight(1).margin({ top: 16 })
      }
    }
    .width('100%').height('100%').padding(16)
    .backgroundColor($r('app.color.ngh_background'))
  }

  @Builder
  Card(id: string, title: string, subtitle: string) {
    Column() {
      SymbolGlyph($r('sys.symbol.trophy'))
        .fontSize(24)
        .fontColor([$r('app.color.ngh_primary')])
      Text(title).fontSize(14).fontWeight(FontWeight.SemiBold)
        .fontColor($r('app.color.ngh_text_primary'))
        .margin({ top: 8 })
      Text(subtitle).fontSize(12).fontColor($r('app.color.ngh_text_secondary'))
        .margin({ top: 2 })
    }
    .width('100%').padding(16).alignItems(HorizontalAlign.Start)
    .backgroundColor($r('app.color.ngh_surface')).borderRadius(12)
    .border({ width: 1, color: $r('app.color.ngh_border') })
    .shadow({ radius: 8, color: '#14000000', offsetX: 0, offsetY: 2 })
    .onTouch((event: TouchEvent) => {
      if (event.type === TouchType.Down) {
        this.pressedId = id;
      } else if (event.type === TouchType.Up || event.type === TouchType.Cancel) {
        this.pressedId = '';
      }
    })
    .scale(this.pressedId === id ? { x: 0.97, y: 0.97 } : { x: 1, y: 1 })
    .animation({ duration: 150, curve: Curve.EaseOut })
  }

  @Builder
  EmptyState() {
    Column() {
      SymbolGlyph($r('sys.symbol.trophy'))
        .fontSize(40)
        .fontColor([$r('app.color.ngh_text_tertiary')])
      Text('暂无榜单').fontSize(14).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: 12 })
      Text('连接音源后将显示热门榜单').fontSize(12)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: 4 })
    }
    .width('100%').padding({ top: 64, bottom: 64 })
    .alignItems(HorizontalAlign.Center)
    .margin({ top: 16 })
  }
}
