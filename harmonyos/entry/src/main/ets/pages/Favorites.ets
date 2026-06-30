// 职责：收藏页 ArkUI 组件，收藏分组卡片网格，简约风格占位。

interface GroupItem { id: string; name: string; count: number }

@Component
export struct FavoritesPage {
  @State groups: Array<GroupItem> = [
    { id: 'f1', name: '我的收藏', count: 3 },
    { id: 'f2', name: '睡前音乐', count: 2 }
  ];

  build() {
    Column() {
      Text('收藏').fontSize(22).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.text_primary'))
      Text('多分组管理，支持导入/导出').fontSize(12)
        .fontColor($r('app.color.text_muted')).margin({ top: 2 })

      if (this.groups.length === 0) {
        this.EmptyState('还没有收藏分组，点击右上角创建')
      } else {
        Grid() {
          ForEach(this.groups, (g: GroupItem) => {
            GridItem() { this.Card(g.name, `${g.count} 首`) }
          }, (g: GroupItem) => g.id)
        }
        .columnsTemplate('1fr 1fr')
        .columnsGap(12).rowsGap(12)
        .layoutWeight(1).margin({ top: 16 })
      }
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

  @Builder
  EmptyState(text: string) {
    Text(text).fontSize(13).fontColor($r('app.color.text_muted'))
      .width('100%').padding(24).textAlign(TextAlign.Center)
      .backgroundColor($r('app.color.bg')).borderRadius(8)
      .border({ width: 1, color: $r('app.color.border') })
      .margin({ top: 16 })
  }
}
