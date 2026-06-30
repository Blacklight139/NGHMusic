// 职责：HarmonyOS ArkUI 主页，Tabs 导航（搜索/列表/收藏/排行榜/本地/设置）+ 底部播放组件。
// 简约风格：主色 #1db954，背景 #fff/#f5f5f5，与桌面端布局对齐。
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
      Tabs({ barPosition: BarPosition.End, index: this.currentIndex }) {
        TabContent() { SearchPage() }.tabBar(this.tabBuilder('搜索', 0))
        TabContent() { PlaylistPage() }.tabBar(this.tabBuilder('播放列表', 1))
        TabContent() { FavoritesPage() }.tabBar(this.tabBuilder('收藏', 2))
        TabContent() { LeaderboardPage() }.tabBar(this.tabBuilder('排行榜', 3))
        TabContent() { LocalMusicPage() }.tabBar(this.tabBuilder('本地', 4))
        TabContent() { SettingsPage() }.tabBar(this.tabBuilder('设置', 5))
      }
      .barMode(BarMode.Fixed)
      .onChange((index: number) => { this.currentIndex = index; })

      // 底部迷你播放组件
      this.MiniPlayer()
    }
    .width('100%').height('100%')
    .backgroundColor($r('app.color.bg_alt'))
  }

  @Builder
  tabBuilder(title: string, index: number) {
    Column() {
      Text(this.tabIcon(index))
        .fontSize(18)
        .fontColor(index === this.currentIndex ? $r('app.color.primary') : $r('app.color.text_muted'))
      Text(title)
        .fontSize(10)
        .margin({ top: 2 })
        .fontColor(index === this.currentIndex ? $r('app.color.primary') : $r('app.color.text_muted'))
    }.width('100%').justifyContent(FlexAlign.Center).padding({ top: 6, bottom: 6 })
  }

  private tabIcon(index: number): string {
    // 用文字符号代替图标（脚手架阶段无图标资源）
    const icons = ['🔍', '☰', '★', '🏆', '📁', '⚙'];
    return icons[index] ?? '·';
  }

  @Builder
  MiniPlayer() {
    Column() {
      // 进度条
      Progress({ value: this.playerStatus.position,
                total: Math.max(this.playerStatus.duration, 1), type: ProgressType.Linear })
        .width('100%').height(2)
        .color($r('app.color.primary'))
        .backgroundColor($r('app.color.border'))

      Row() {
        // 封面
        Column()
          .width(40).height(40).borderRadius(8)
          .linearGradient({ angle: 135, colors: [[$r('app.color.primary'), 0], [$r('app.color.primary_dark'), 1]] })
        Column() {
          Text(this.playerStatus.currentTitle)
            .fontSize(13).fontWeight(FontWeight.Medium).maxLines(1)
            .textOverflow({ overflow: TextOverflow.Ellipsis })
            .fontColor($r('app.color.text_primary'))
          Text(this.playerStatus.currentArtist)
            .fontSize(11).maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
            .fontColor($r('app.color.text_muted'))
        }
        .layoutWeight(1).margin({ left: 8 })

        // 控制
        Text('⏮').fontSize(18).margin({ right: 12 }).onClick(() => playerManager.toPrev())
        Text(this.playerStatus.isPlaying ? '⏸' : '▶').fontSize(20).margin({ right: 12 })
          .onClick(() => { this.playerStatus.isPlaying ? playerManager.pause() : playerManager.resume(); })
        Text('⏭').fontSize(18).margin({ right: 12 }).onClick(() => playerManager.toNext())
        Text(this.modeIcon()).fontSize(16).margin({ right: 8 })
          .onClick(() => playerManager.toggleMode())
        Text('🔊').fontSize(14).fontColor($r('app.color.text_muted'))
        Slider({ value: this.playerStatus.volume, min: 0, max: 1, step: 0.01 })
          .width(80).blockColor($r('app.color.primary')).selectedColor($r('app.color.primary'))
          .onChange((v: number) => playerManager.setVolume(v))
      }
      .width('100%').height(56).padding({ left: 16, right: 16 })
      .backgroundColor($r('app.color.bg'))
    }
  }

  private modeIcon(): string {
    switch (this.playerStatus.mode) {
      case PlayMode.SEQUENTIAL: return '🔁';
      case PlayMode.SINGLE_LOOP: return '🔂';
      case PlayMode.RANDOM: return '🔀';
      default: return '🔁';
    }
  }
}
