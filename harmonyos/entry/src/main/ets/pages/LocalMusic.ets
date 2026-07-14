// 职责：本地音乐页 ArkUI 组件，按歌曲/专辑/艺术家/文件夹浏览（卡片化，豆包风格 ngh_* token）。
// 对齐 docs/api/sound-source-api.md：MusicCoreBridge.listLocalSongs() 返回 Song[]。
// 占位：脚手架本地数据；点击条目调用 playerManager.playSong 拉起播放。

import { Song } from '../models/Models';
import { MusicCoreBridge } from '../bridge/MusicCoreBridge';
import { playerManager } from '../player/PlayerManager';

@Component
export struct LocalMusicPage {
  @State filter: number = 0;
  private filters: Array<string> = ['歌曲', '专辑', '艺术家', '文件夹'];
  @State songs: Array<Song> = [
    {
      id: 'lo1', source_id: 'local', title: '本地示例一', artists: ['未知艺术家'],
      album: '本地专辑', duration_ms: 198000, local_path: '/music/a.mp3',
      origin: { type: 'Local', path: '/music/a.mp3' }
    },
    {
      id: 'lo2', source_id: 'local', title: '本地示例二', artists: ['艺术家C'],
      duration_ms: 165000, local_path: '/music/b.flac',
      origin: { type: 'Local', path: '/music/b.flac' }
    }
  ];
  @State pressedId: string = '';

  build() {
    Column() {
      Text('本地音乐').fontSize(22).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_primary'))
      Text('扫描本地目录并播放').fontSize(12)
        .fontColor($r('app.color.ngh_text_secondary')).margin({ top: 2 })

      // 筛选分段
      Row() {
        ForEach(this.filters, (label: string, i: number) => {
          Text(label)
            .fontSize(12)
            .layoutWeight(1).textAlign(TextAlign.Center)
            .padding({ top: 6, bottom: 6 })
            .fontColor(i === this.filter ? $r('app.color.ngh_surface') : $r('app.color.ngh_text_primary'))
            .backgroundColor(i === this.filter ? $r('app.color.ngh_primary') : $r('app.color.ngh_surface_alt'))
            .borderRadius(8)
            .onClick(() => { this.filter = i; })
        }, (label: string) => label)
      }
      .margin({ top: 16 })

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
      Text('本地').fontSize(11).fontColor($r('app.color.ngh_text_secondary'))
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
      SymbolGlyph($r('sys.symbol.folder'))
        .fontSize(40)
        .fontColor([$r('app.color.ngh_text_tertiary')])
      Text('未找到本地音乐').fontSize(14).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: 12 })
      Text('在设置中添加本地目录并扫描').fontSize(12)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: 4 })
    }
    .width('100%').padding({ top: 64, bottom: 64 })
    .alignItems(HorizontalAlign.Center)
  }

  private playSong(song: Song): void {
    // 本地歌曲 play_url 通常为空，使用 local_path 作为播放 URL
    const url = song.play_url ?? '';
    const finalUrl = url.length > 0 ? url : (song.local_path ?? '');
    playerManager.playSong({
      id: song.id,
      title: song.title,
      artist: song.artists.join(' / '),
      playUrl: finalUrl,
    });
  }

  // 接入核心后的刷新入口（脚手架阶段未触发）
  private refreshLocal(): void {
    const list = MusicCoreBridge.listLocalSongs();
    if (list && list.length > 0) {
      this.songs = list;
    }
  }
}
