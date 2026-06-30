// 职责：搜索页 ArkUI 组件，搜索栏 + 结果列表，简约风格占位。
// 对齐桌面端 pages/search.js：调用 MusicCoreBridge.search。

import { MusicCoreBridge, Song } from '../bridge/MusicCoreBridge';

@Component
export struct SearchPage {
  @State keyword: string = '';
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
        .fontColor($r('app.color.text_primary'))
      Text('跨音源聚合检索').fontSize(12).fontColor($r('app.color.text_muted'))
        .margin({ top: 2 })

      Row() {
        TextInput({ placeholder: '输入歌曲 / 艺术家 / 专辑', text: this.keyword })
          .layoutWeight(1)
          .height(40)
          .borderRadius(8)
          .backgroundColor($r('app.color.bg'))
          .border({ width: 1, color: $r('app.color.border') })
          .onChange((v: string) => { this.keyword = v; })
        Button('搜索')
          .margin({ left: 8 })
          .backgroundColor($r('app.color.primary'))
          .onClick(() => this.doSearch())
      }
      .margin({ top: 16 })

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
          .fontColor($r('app.color.text_primary'))
          .maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
        Text(song.artists.join(' / ')).fontSize(11).fontColor($r('app.color.text_muted'))
          .maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
      }.layoutWeight(1).margin({ left: 8 })
      Text('在线').fontSize(11).fontColor($r('app.color.text_muted'))
        .padding({ left: 8, right: 8, top: 2, bottom: 2 })
        .backgroundColor($r('app.color.bg_alt'))
        .borderRadius(999)
        .border({ width: 1, color: $r('app.color.border') })
    }
    .width('100%').padding(12)
    .backgroundColor($r('app.color.bg'))
    .borderRadius(8)
    .border({ width: 1, color: $r('app.color.border') })
    .margin({ bottom: 8 })
  }

  private doSearch(): void {
    if (this.keyword.length === 0) {
      return;
    }
    const result = MusicCoreBridge.search(this.keyword, 1, 20);
    if (result && result.songs.length > 0) {
      this.songs = result.songs;
    }
  }
}
