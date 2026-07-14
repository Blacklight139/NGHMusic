// 职责：播放列表页 ArkUI 组件，展示当前队列歌曲（线性列表，无卡片，豆包风格 ngh_* token）。
// 队列来源：playerManager.getQueue 不暴露 song 列表，故本页维护本地副本，
// 由 Search/LocalMusic 等页面通过 playerManager.setQueue 设置；
// 点击条目调用 playerManager.playSong 拉起播放；当前播放歌曲以主色 + 加粗高亮。

import { Song } from '../models/Models';
import { playerManager, PlayMode } from '../player/PlayerManager';
import { NghRadius, NghSpacing, NghFontSize } from '../common/Dimensions';

interface PlayerStatus {
  currentTitle: string;
  currentArtist: string;
  currentSongId: string;
  position: number;
  duration: number;
  isPlaying: boolean;
  volume: number;
  mode: PlayMode;
  hasPlayable: boolean;
}

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
  @StorageLink('player_status') playerStatus: PlayerStatus = {
    currentTitle: '未在播放', currentArtist: '—', currentSongId: '',
    position: 0, duration: 0, isPlaying: false, volume: 0.8,
    mode: PlayMode.SEQUENTIAL, hasPlayable: false,
  };

  build() {
    Column() {
      Text('播放列表').fontSize(NghFontSize.HEADLINE).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_primary'))
      Text(`当前队列共 ${this.songs.length} 首`).fontSize(NghFontSize.BODY_SM)
        .fontColor($r('app.color.ngh_text_secondary')).margin({ top: NghSpacing.S1 })

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
      .layoutWeight(1).margin({ top: NghSpacing.S4 })
    }
    .width('100%').height('100%').padding(NghSpacing.S4)
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
          Text(song.title).fontSize(NghFontSize.BODY)
            .fontWeight(song.id === this.playerStatus.currentSongId ? FontWeight.Bold : FontWeight.Medium)
            .fontColor(song.id === this.playerStatus.currentSongId ? $r('app.color.ngh_primary') : $r('app.color.ngh_text_primary'))
            .maxLines(1)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
          Text(song.artists.join(' / ')).fontSize(NghFontSize.CAPTION)
            .fontColor(song.id === this.playerStatus.currentSongId ? $r('app.color.ngh_primary') : $r('app.color.ngh_text_secondary'))
            .maxLines(1)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
        }.layoutWeight(1).margin({ left: NghSpacing.S3 })
        Text(this.originLabel(song)).fontSize(NghFontSize.CAPTION)
          .fontColor($r('app.color.ngh_text_secondary'))
          .padding({ left: NghSpacing.S2, right: NghSpacing.S2, top: NghSpacing.S1, bottom: NghSpacing.S1 })
          .backgroundColor($r('app.color.ngh_surface_alt')).borderRadius(NghRadius.PILL)
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
      SymbolGlyph($r('sys.symbol.list_bullet'))
        .fontSize(40)
        .fontColor([$r('app.color.ngh_text_tertiary')])
      Text('播放列表为空').fontSize(NghFontSize.BODY).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: NghSpacing.S3 })
      Text('从搜索或本地音乐中添加歌曲').fontSize(NghFontSize.BODY_SM)
        .fontColor($r('app.color.ngh_text_tertiary'))
        .margin({ top: NghSpacing.S1 })
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
