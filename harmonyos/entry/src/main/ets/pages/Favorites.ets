// 职责：收藏页 ArkUI 组件，收藏分组卡片网格（豆包风格 ngh_* token）。
// 对齐 core/src/models.rs::FavoriteGroup（id/name/song_ids）。
// 占位实现：脚手架分组数据；接入核心后由 library 持久化加载。

import { FavoriteGroup } from '../models/Models';

@Component
export struct FavoritesPage {
  @State groups: Array<FavoriteGroup> = [
    { id: 'f1', name: '我的收藏', song_ids: ['s1', 's2', 's3'] },
    { id: 'f2', name: '睡前音乐', song_ids: ['s4', 's5'] }
  ];
  @State pressedId: string = '';

  build() {
    Column() {
      Text('收藏').fontSize(22).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_primary'))
      Text('多分组管理，支持导入/导出').fontSize(12)
        .fontColor($r('app.color.ngh_text_secondary')).margin({ top: 2 })

      if (this.groups.length === 0) {
        this.EmptyState()
      } else {
        Scroll() {
          Grid() {
            ForEach(this.groups, (g: FavoriteGroup) => {
              GridItem() { this.Card(g.id, g.name, `${g.song_ids.length} 首`) }
            }, (g: FavoriteGroup) => g.id)
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
      SymbolGlyph($r('sys.symbol.heart'))
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
    .transition({ type: TransitionType.All, opacity: 0, translate: { y: 12 } })
  }

  @Builder
  EmptyState() {
    Column() {
      SymbolGlyph($r('sys.symbol.heart'))
        .fontSize(40)
        .fontColor([$r('app.color.ngh_text_tertiary')])
      Text('还没有收藏分组').fontSize(14).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: 12 })
      Text('点击右上角创建分组').fontSize(12)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: 4 })
    }
    .width('100%').padding({ top: 64, bottom: 64 })
    .alignItems(HorizontalAlign.Center)
    .margin({ top: 16 })
  }
}
