// 职责：HarmonyOS ArkUI 主页，Tabs 导航（搜索/列表/收藏/排行榜/本地/设置）+ 底部播放组件。
// 豆包风格设计 token（ngh_*），品牌位「逆光音乐」，Tabs/播放控制图标统一使用 sys.symbol.* 系统符号。
// 集成方式：作为 EntryAbility 的入口页面（main_pages.json 第一项）。

import { playerManager, PlayMode } from '../player/PlayerManager';
import { SearchPage } from './Search';
import { PlaylistPage } from './Playlist';
import { FavoritesPage } from './Favorites';
import { LeaderboardPage } from './Leaderboard';
import { LocalMusicPage } from './LocalMusic';
import { SettingsPage } from './Settings';

// 页面状态结构（与 PlayerManager 一致）
interface PlayerStatus {
  currentTitle: string;
  currentArtist: string;
  position: number;
  duration: number;
  isPlaying: boolean;
  volume: number;
  mode: PlayMode;
}

@Entry
@Component
struct Index {
  @State currentIndex: number = 0;
  @StorageLink('player_status') playerStatus: PlayerStatus = {
    currentTitle: '未在播放', currentArtist: '—', position: 0, duration: 0,
    isPlaying: false, volume: 0.8, mode: PlayMode.SEQUENTIAL
  };

  build() {
    Column() {
      // 品牌位
      Row() {
        Text($r('app.string.app_name'))
          .fontSize(20)
          .fontWeight(FontWeight.Bold)
          .fontColor($r('app.color.ngh_text_primary'))
      }
      .width('100%')
      .height(48)
      .padding({ left: 16, right: 16 })
      .backgroundColor($r('app.color.ngh_surface'))

      Tabs({ barPosition: BarPosition.End, index: this.currentIndex }) {
        TabContent() { SearchPage() }.tabBar(this.tabBuilder('搜索', 0))
        TabContent() { PlaylistPage() }.tabBar(this.tabBuilder('播放列表', 1))
        TabContent() { FavoritesPage() }.tabBar(this.tabBuilder('收藏', 2))
        TabContent() { LeaderboardPage() }.tabBar(this.tabBuilder('排行榜', 3))
        TabContent() { LocalMusicPage() }.tabBar(this.tabBuilder('本地', 4))
        TabContent() { SettingsPage() }.tabBar(this.tabBuilder('设置', 5))
      }
      .barMode(BarMode.Fixed)
      .animationMode(AnimationMode.CONTENT_ANIMATION)
      .onChange((index: number) => {
        animateTo({ duration: 200, curve: Curve.EaseInOut }, () => {
          this.currentIndex = index;
        })
      })

      // 底部迷你播放组件
      if (this.hasCurrentSong()) {
        this.MiniPlayer()
      }
    }
    .width('100%').height('100%')
    .backgroundColor($r('app.color.ngh_background'))
  }

  @Builder
  tabBuilder(title: string, index: number) {
    Column() {
      SymbolGlyph(this.tabSysIcon(index))
        .fontSize(20)
        .fontColor([index === this.currentIndex ? $r('app.color.ngh_primary') : $r('app.color.ngh_text_tertiary')])
      Text(title)
        .fontSize(10)
        .margin({ top: 2 })
        .fontColor(index === this.currentIndex ? $r('app.color.ngh_primary') : $r('app.color.ngh_text_tertiary'))
    }
    .width('100%').justifyContent(FlexAlign.Center).padding({ top: 6, bottom: 6 })
    .backgroundColor($r('app.color.ngh_surface'))
  }

  private tabSysIcon(index: number): Resource {
    // 使用 HarmonyOS 系统符号 sys.symbol.*（API 10+），无需外部图标资源
    switch (index) {
      case 0: return $r('sys.symbol.magnifyingglass');
      case 1: return $r('sys.symbol.list_bullet');
      case 2: return $r('sys.symbol.heart');
      case 3: return $r('sys.symbol.trophy');
      case 4: return $r('sys.symbol.folder');
      case 5: return $r('sys.symbol.gearshape');
      default: return $r('sys.symbol.circle');
    }
  }

  // 是否有当前播放歌曲：用于 MiniPlayer 条件渲染（出现/消失过渡）
  private hasCurrentSong(): boolean {
    return this.playerStatus.duration > 0 || this.playerStatus.isPlaying;
  }

  @Builder
  MiniPlayer() {
    Column() {
      // 进度条
      Progress({ value: this.playerStatus.position,
                total: Math.max(this.playerStatus.duration, 1), type: ProgressType.Linear })
        .width('100%').height(2)
        .color($r('app.color.ngh_primary'))
        .backgroundColor($r('app.color.ngh_border_soft'))

      Row() {
        // 封面
        Column()
          .width(40).height(40).borderRadius(8)
          .linearGradient({ angle: 135, colors: [[$r('app.color.ngh_primary'), 0], [$r('app.color.ngh_primary_hover'), 1]] })
        Column() {
          Text(this.playerStatus.currentTitle)
            .fontSize(13).fontWeight(FontWeight.Medium).maxLines(1)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
            .fontColor($r('app.color.ngh_text_primary'))
          Text(this.playerStatus.currentArtist)
            .fontSize(11).maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
            .fontColor($r('app.color.ngh_text_secondary'))
        }
        .layoutWeight(1).margin({ left: 8 })

        // 控制（系统符号；media 控制类符号名需在目标设备核对，如缺失可改用 Image($r('app.media.xxx')) SVG）
        SymbolGlyph($r('sys.symbol.backward_end'))
          .fontSize(20)
          .fontColor([$r('app.color.ngh_text_primary')])
          .margin({ right: 12 }).onClick(() => playerManager.toPrev())
        SymbolGlyph(this.playerStatus.isPlaying ? $r('sys.symbol.pause') : $r('sys.symbol.play'))
          .fontSize(22)
          .fontColor([$r('app.color.ngh_text_primary')])
          .margin({ right: 12 })
          .onClick(() => { this.playerStatus.isPlaying ? playerManager.pause() : playerManager.resume(); })
        SymbolGlyph($r('sys.symbol.forward_end'))
          .fontSize(20)
          .fontColor([$r('app.color.ngh_text_primary')])
          .margin({ right: 12 }).onClick(() => playerManager.toNext())
        SymbolGlyph(this.modeSysIcon())
          .fontSize(18)
          .fontColor([$r('app.color.ngh_text_secondary')])
          .margin({ right: 8 }).onClick(() => playerManager.toggleMode())
        SymbolGlyph($r('sys.symbol.speaker'))
          .fontSize(16)
          .fontColor([$r('app.color.ngh_text_secondary')])
        Slider({ value: this.playerStatus.volume, min: 0, max: 1, step: 0.01 })
          .width(80).blockColor($r('app.color.ngh_primary')).selectedColor($r('app.color.ngh_primary'))
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

  private modeSysIcon(): Resource {
    switch (this.playerStatus.mode) {
      case PlayMode.SEQUENTIAL: return $r('sys.symbol.repeat');
      case PlayMode.SINGLE_LOOP: return $r('sys.symbol.repeat_1');
      case PlayMode.RANDOM: return $r('sys.symbol.shuffle');
      default: return $r('sys.symbol.repeat');
    }
  }
}
