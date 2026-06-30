// 职责：排行榜页 ArkUI 组件，榜单卡片网格，简约风格占位。

interface BoardItem { id: string; name: string; count: number }

@Component
export struct LeaderboardPage {
  @State boards: Array<BoardItem> = [
    { id: 'l1', name: '热歌榜', count: 0 },
    { id: 'l2', name: '新歌榜', count: 0 },
    { id: 'l3', name: '飙升榜', count: 0 }
  ];

  build() {
    Column() {
      Text('排行榜').fontSize(22).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.text_primary'))
      Text('各音源热门榜单').fontSize(12)
        .fontColor($r('app.color.text_muted')).margin({ top: 2 })

      Grid() {
        ForEach(this.boards, (b: BoardItem) => {
          GridItem() { this.Card(b.name, `${b.count} 首`) }
        }, (b: BoardItem) => b.id)
      }
      .columnsTemplate('1fr 1fr')
      .columnsGap(12).rowsGap(12)
      .layoutWeight(1).margin({ top: 16 })
    }
    .width('100%').height('100%').padding(16)
    .backgroundColor($r('app.color.bg_alt'))
  }

  @Builder
  Card(title: string, subtitle: string) {
    Column() {
      Text(title).fontSize(14).fontWeight(FontWeight.SemiBold)
        .fontColor($r('app.color.text_primary'))
      Text(subtitle).fontSize(12).fontColor($r('app.color.text_muted'))
        .margin({ top: 2 })
    }
    .width('100%').padding(16).alignItems(HorizontalAlign.Start)
    .backgroundColor($r('app.color.bg')).borderRadius(8)
    .border({ width: 1, color: $r('app.color.border') })
  }
}
