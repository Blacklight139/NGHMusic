// 职责：搜索页 ArkUI 组件，搜索栏 + 结果列表（卡片化，豆包风格 ngh_* token）。
// 对齐桌面端 pages/search.js：调用 MusicCoreBridge.search。
// 点击结果调用 playerManager.playSong 拉起播放。

import { MusicCoreBridge } from '../bridge/MusicCoreBridge';
import { Song } from '../models/Models';
import { playerManager } from '../player/PlayerManager';
import { NghRadius, NghSpacing, NghFontSize } from '../common/Dimensions';

@Component
export struct SearchPage {
  @State keyword: string = '';
  @State pressedId: string = '';
  @State loading: boolean = false;
  @State total: number = 0;
  @State songs: Array<Song> = [
    {
      id: 's1', source_id: 'demo', title: '示例歌曲一', artists: ['艺术家A'],
      album: '专辑X', duration_ms: 210000,
      origin: { type: 'Online', source_id: 'demo', play_url: '' }
    },
    {
      id: 's2', source_id: 'demo', title: '示例歌曲二', artists: ['艺术家B'],
      duration_ms: 184000, origin: { type: 'Online', source_id: 'demo', play_url: '' }
    }
  ];

  build() {
    Column() {
      Text('搜索').fontSize(22).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_primary'))
      Text('跨音源聚合检索').fontSize(12).fontColor($r('app.color.ngh_text_secondary'))
        .margin({ top: 2 })

      Row() {
        TextInput({ placeholder: '输入歌曲 / 艺术家 / 专辑', text: this.keyword })
          .layoutWeight(1)
          .height(40)
          .borderRadius(8)
          .backgroundColor($r('app.color.ngh_surface'))
          .border({ width: 1, color: $r('app.color.ngh_border') })
          .onChange((v: string) => { this.keyword = v; })
        Button('搜索')
          .margin({ left: 8 })
          .backgroundColor($r('app.color.ngh_primary'))
          .onClick(() => this.doSearch())
      }
      .margin({ top: 16 })

      if (this.total > 0) {
        Text(`共 ${this.total} 条结果`).fontSize(11)
          .fontColor($r('app.color.ngh_text_tertiary'))
          .margin({ top: 8, bottom: 4 })
      }

      Scroll() {
        Column() {
          if (this.songs.length === 0) {
            this.EmptyState()
          } else {
            ForEach(this.songs, (song: Song, i: number) => {
              this.SongRow(i + 1, song)
            }, (song: Song) => song.id)
          }
        }
      }
      .layoutWeight(1).margin({ top: 12 })
    }
    .width('100%').height('100%').padding(16)
    .backgroundColor($r('app.color.ngh_background'))
  }

  @Builder
  SongRow(index: number, song: Song) {
    Column() {
      Row() {
        // 封面占位（含序号）：ngh_primary_soft 底 + ngh_primary 前景，圆角 8。
        Text(`${index}`)
          .fontSize(NghFontSize.BODY_SM)
          .fontWeight(FontWeight.Medium)
          .fontColor($r('app.color.ngh_primary'))
          .textAlign(TextAlign.Center)
          .width(36).height(36)
          .backgroundColor($r('app.color.ngh_primary_soft'))
          .borderRadius(NghRadius.SM)
        Column() {
          Text(song.title).fontSize(NghFontSize.BODY).fontWeight(FontWeight.Medium)
            .fontColor($r('app.color.ngh_text_primary'))
            .maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
          Text(song.artists.join(' / ')).fontSize(NghFontSize.CAPTION)
            .fontColor($r('app.color.ngh_text_secondary'))
            .maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
        }.layoutWeight(1).margin({ left: NghSpacing.S3 })
        Text(this.originLabel(song)).fontSize(NghFontSize.CAPTION)
          .fontColor($r('app.color.ngh_text_secondary'))
          .padding({ left: NghSpacing.S2, right: NghSpacing.S2, top: 2, bottom: 2 })
          .backgroundColor($r('app.color.ngh_surface_alt'))
          .borderRadius(NghRadius.PILL)
      }
      .width('100%')
      .padding({ left: NghSpacing.S4, right: NghSpacing.S4, top: NghSpacing.S3, bottom: NghSpacing.S3 })
      .onClick(() => this.playSong(song))
      .onTouch((event: TouchEvent) => {
        if (event.type === TouchType.Down) {
          this.pressedId = song.id;
        } else if (event.type === TouchType.Up || event.type === TouchType.Cancel) {
          this.pressedId = '';
        }
      })
      .scale(this.pressedId === song.id ? { x: 0.97, y: 0.97 } : { x: 1, y: 1 })
      .animation({ duration: 150, curve: Curve.EaseOut })
      .transition({ type: TransitionType.All, opacity: 0, translate: { y: 12 } })

      // 底部分隔线：替代卡片描边，Linear 风格。
      Row()
        .width('100%')
        .height(0.5)
        .backgroundColor($r('app.color.ngh_border'))
        .margin({ left: NghSpacing.S4, right: NghSpacing.S4 })
    }
  }

  @Builder
  EmptyState() {
    Column() {
      SymbolGlyph($r('sys.symbol.magnifyingglass'))
        .fontSize(40)
        .fontColor([$r('app.color.ngh_text_tertiary')])
      Text('暂无搜索结果').fontSize(14).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: 12 })
      Text('试试输入歌曲 / 艺术家 / 专辑').fontSize(12)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: 4 })
    }
    .width('100%').padding({ top: 64, bottom: 64 })
    .alignItems(HorizontalAlign.Center)
  }

  private doSearch(): void {
    if (this.keyword.length === 0) {
      return;
    }
    this.loading = true;
    const result = MusicCoreBridge.search(this.keyword, 1, 20);
    this.loading = false;
    if (result) {
      this.songs = result.songs;
      this.total = result.total;
    }
  }

  private playSong(song: Song): void {
    // 将 Song 模型（snake_case play_url）映射为 PlayerManager QueueItem（camelCase playUrl）
    playerManager.playSong({
      id: song.id,
      title: song.title,
      artist: song.artists.join(' / '),
      playUrl: song.play_url ?? '',
    });
  }

  private originLabel(song: Song): string {
    switch (song.origin.type) {
      case 'Online': return '在线';
      case 'Local': return '本地';
      case 'Nas': return 'NAS';
      default: return '未知';
    }
  }
}
