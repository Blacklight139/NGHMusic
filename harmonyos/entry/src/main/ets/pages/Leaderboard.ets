// 职责：排行榜页 ArkUI 组件，榜单卡片网格（豆包风格 ngh_* token）。
// 对齐 docs/api/sound-source-api.md「获取排行榜」：MusicCoreBridge.getLeaderboards(sourceId)
// 返回 Leaderboard[] JSON（id/source_id/name/cover_url/songs）。
// 占位：脚手架榜单数据；接入核心后由音源 id 驱动加载。

import { Leaderboard } from '../models/Models';
import { MusicCoreBridge } from '../bridge/MusicCoreBridge';

@Component
export struct LeaderboardPage {
  @State boards: Array<Leaderboard> = [
    { id: 'l1', source_id: 'demo', name: '热歌榜', cover_url: undefined, songs: [] },
    { id: 'l2', source_id: 'demo', name: '新歌榜', cover_url: undefined, songs: [] },
    { id: 'l3', source_id: 'demo', name: '飙升榜', cover_url: undefined, songs: [] }
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
            ForEach(this.boards, (b: Leaderboard) => {
              GridItem() { this.Card(b.id, b.name, `${b.songs.length} 首`) }
            }, (b: Leaderboard) => b.id)
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

  // 接入核心后的刷新入口（脚手架阶段未触发）
  private refreshBoards(sourceId: string): void {
    const json = MusicCoreBridge.getLeaderboards(sourceId);
    if (json === null || json.length === 0) {
      return;
    }
    try {
      const arr = JSON.parse(json) as Leaderboard[];
      if (Array.isArray(arr)) {
        this.boards = arr;
      }
    } catch (e) {
      console.warn(`[Leaderboard] 解析榜单失败: ${(e as Error).message ?? e}`);
    }
  }
}
