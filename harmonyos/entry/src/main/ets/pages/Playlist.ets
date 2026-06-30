// 职责：播放列表页 ArkUI 组件，展示当前队列歌曲，简约风格占位。

import { Song } from '../bridge/MusicCoreBridge';

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

  build() {
    Column() {
      Text('播放列表').fontSize(22).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.text_primary'))
      Text(`当前队列共 ${this.songs.length} 首`).fontSize(12)
        .fontColor($r('app.color.text_muted')).margin({ top: 2 })

      Scroll() {
        Column() {
          ForEach(this.songs, (song: Song, i: number) => {
            this.SongRow(i + 1, song)
          }, (song: Song) => song.id)
        }
      }
      .layoutWeight(1).margin({ top: 16 })
    }
    .width('100%').height('100%').padding(16)
    .backgroundColor($r('app.color.bg_alt'))
  }

  @Builder
  SongRow(index: number, song: Song) {
    Row() {
      Text(`${index}`).fontSize(12).fontColor($r('app.color.text_muted')).width(24)
      Column() {
        Text(song.title).fontSize(14).fontWeight(FontWeight.Medium)
          .fontColor($r('app.color.text_primary')).maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
        Text(song.artists.join(' / ')).fontSize(11)
          .fontColor($r('app.color.text_muted')).maxLines(1)
          .textOverflow({ overflow: TextOverflow.Ellipsis })
      }.layoutWeight(1).margin({ left: 8 })
    }
    .width('100%').padding(12).backgroundColor($r('app.color.bg'))
    .borderRadius(8).border({ width: 1, color: $r('app.color.border') })
    .margin({ bottom: 8 })
  }
}
