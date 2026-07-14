// 职责：播放列表页 ArkUI 组件，展示当前队列歌曲（卡片化，豆包风格 ngh_* token）。
// 队列来源：playerManager.getQueue 不暴露 song 列表，故本页维护本地副本，
// 由 Search/LocalMusic 等页面通过 playerManager.setQueue 设置；
// 点击条目调用 playerManager.playSong 拉起播放。

import { Song } from '../models/Models';
import { playerManager } from '../player/PlayerManager';

@Component
export struct PlaylistPage {
  @State songs: Array<Song> = [
    {
      id: 'p1', source_id: 'demo', title: '播放列表曲目一', artists: ['艺术家A'],
      duration_ms: 200000, origin: { type: 'Online', source_id: 'demo', play_url: '' }
    },
    {
      id: 'p2', source_id: 'demo', title: '播放列表曲目二', artists: ['艺术家B'],
      duration_ms: 175000, origin: { type: 'Online', source_id: 'demo', play_url: '' }
    }
  ];
  @State pressedId: string = '';

  build() {
    Column() {
      Text('播放列表').fontSize(22).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_primary'))
      Text(`当前队列共 ${this.songs.length} 首`).fontSize(12)
        .fontColor($r('app.color.ngh_text_secondary')).margin({ top: 2 })

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
      .layoutWeight(1).margin({ top: 16 })
    }
    .width('100%').height('100%').padding(16)
    .backgroundColor($r('app.color.ngh_background'))
  }

  @Builder
  SongRow(index: number, song: Song) {
    Row() {
      Text(`${index}`).fontSize(12).fontColor($r('app.color.ngh_text_tertiary')).width(24)
      Column() {
        Text(song.title).fontSize(14).fontWeight(FontWeight.Medium)
          .fontColor($r('app.color.ngh_text_primary')).maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
        Text(song.artists.join(' / ')).fontSize(11)
          .fontColor($r('app.color.ngh_text_secondary')).maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
      }.layoutWeight(1).margin({ left: 8 })
      Text(this.originLabel(song)).fontSize(11)
        .fontColor($r('app.color.ngh_text_secondary'))
        .padding({ left: 8, right: 8, top: 2, bottom: 2 })
        .backgroundColor($r('app.color.ngh_surface_alt')).borderRadius(999)
    }
    .width('100%').padding(16)
    .backgroundColor($r('app.color.ngh_surface'))
    .borderRadius(12)
    .border({ width: 1, color: $r('app.color.ngh_border') })
    .shadow({ radius: 8, color: '#14000000', offsetX: 0, offsetY: 2 })
    .margin({ bottom: 12 })
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
  }

  @Builder
  EmptyState() {
    Column() {
      SymbolGlyph($r('sys.symbol.list_bullet'))
        .fontSize(40)
        .fontColor([$r('app.color.ngh_text_tertiary')])
      Text('播放列表为空').fontSize(14).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: 12 })
      Text('从搜索或本地音乐中添加歌曲').fontSize(12)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: 4 })
    }
    .width('100%').padding({ top: 64, bottom: 64 })
    .alignItems(HorizontalAlign.Center)
  }

  private playSong(song: Song): void {
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
