// 职责：底部播放控制条组件（PlaybackBar）。
// 设计：豆包风格 ngh_* token，品牌位「逆光音乐」；图标统一使用 HarmonyOS 系统符号 sys.symbol.*。
// 集成方式：在 Index.ets 中作为 Tabs 下方常驻组件渲染；通过 @StorageLink 与 PlayerManager 同步状态。
//
// 规范：
//   - 无 emoji，全部图标使用 sys.symbol.*（API 10+）；目标设备若缺失某符号，可改用 Image($r('app.media.xxx')) SVG。
//   - 仅在 PlayerManager.hasPlayable 为 true（持有可播放曲目）时显示。
//   - HAR-001 配合：hasPlayable 替代旧 duration>0||isPlaying 判定，避免「假播放」时显示进度条。

import { playerManager, PlayMode } from '../player/PlayerManager';

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
export struct PlaybackBar {
  @StorageLink('player_status') playerStatus: PlayerStatus = {
    currentTitle: '未在播放', currentArtist: '—', currentSongId: '',
    position: 0, duration: 0, isPlaying: false, volume: 0.8,
    mode: PlayMode.SEQUENTIAL, hasPlayable: false,
  };

  build() {
    if (this.playerStatus.hasPlayable) {
      Column() {
        // 进度条
        Progress({
          value: this.playerStatus.position,
          total: Math.max(this.playerStatus.duration, 1),
          type: ProgressType.Linear,
        })
          .width('100%').height(2)
          .color($r('app.color.ngh_primary'))
          .backgroundColor($r('app.color.ngh_border_soft'))

        Row() {
          // 封面占位（线性渐变；接入封面后改为 Image(song.cover_url)）
          Column()
            .width(40).height(40).borderRadius(8)
            .linearGradient({
              angle: 135,
              colors: [[$r('app.color.ngh_primary'), 0], [$r('app.color.ngh_primary_hover'), 1]],
            })

          // 标题 / 艺术家
          Column() {
            Text(this.playerStatus.currentTitle)
              .fontSize(13).fontWeight(FontWeight.Medium).maxLines(1)
              .textOverflow({ overflow: TextOverflow.Ellipsis })
              .fontColor($r('app.color.ngh_text_primary'))
            Text(this.playerStatus.currentArtist)
              .fontSize(11).maxLines(1)
              .textOverflow({ overflow: TextOverflow.Ellipsis })
              .fontColor($r('app.color.ngh_text_secondary'))
          }
          .layoutWeight(1).margin({ left: 8 })

          // 上一首
          SymbolGlyph($r('sys.symbol.backward_end'))
            .fontSize(20)
            .fontColor([$r('app.color.ngh_text_primary')])
            .margin({ right: 12 })
            .onClick(() => playerManager.toPrev())

          // 播放 / 暂停
          SymbolGlyph(this.playerStatus.isPlaying ? $r('sys.symbol.pause') : $r('sys.symbol.play'))
            .fontSize(22)
            .fontColor([$r('app.color.ngh_text_primary')])
            .margin({ right: 12 })
            .onClick(() => {
              if (this.playerStatus.isPlaying) {
                playerManager.pause();
              } else {
                playerManager.resume();
              }
            })

          // 下一首
          SymbolGlyph($r('sys.symbol.forward_end'))
            .fontSize(20)
            .fontColor([$r('app.color.ngh_text_primary')])
            .margin({ right: 12 })
            .onClick(() => playerManager.toNext())

          // 播放模式
          SymbolGlyph(this.modeSysIcon())
            .fontSize(18)
            .fontColor([$r('app.color.ngh_text_secondary')])
            .margin({ right: 8 })
            .onClick(() => playerManager.toggleMode())

          // 音量
          SymbolGlyph($r('sys.symbol.waveform'))
            .fontSize(16)
            .fontColor([$r('app.color.ngh_text_secondary')])
          Slider({ value: this.playerStatus.volume, min: 0, max: 1, step: 0.01 })
            .width(80)
            .blockColor($r('app.color.ngh_primary'))
            .selectedColor($r('app.color.ngh_primary'))
            .onChange((v: number) => playerManager.setVolume(v))
        }
        .width('100%').height(56).padding({ left: 16, right: 16 })
      }
      .width('100%')
      .backgroundColor($r('app.color.ngh_surface'))
      .borderRadius(16)
      .shadow({ radius: 12, color: '#1F000000', offsetX: 0, offsetY: 4 })
      .margin({ left: 12, right: 12, bottom: 12 })
      .clip(true)
      .animation({ duration: 200, curve: Curve.EaseInOut })
      .transition({ type: TransitionType.All, opacity: 0, translate: { y: 100 } })
    }
  }

  private modeSysIcon(): Resource {
    switch (this.playerStatus.mode) {
      case PlayMode.SEQUENTIAL: return $r('sys.symbol.repeat');
      case PlayMode.SINGLE_LOOP: return $r('sys.symbol.repeat_1');
      case PlayMode.RANDOM: return $r('sys.symbol.shuffle');
      default: return $r('sys.symbol.repeat');
    }
  }
}
