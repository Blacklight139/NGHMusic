// 职责：HarmonyOS ArkUI 主页，Tabs 导航（搜索/列表/收藏/歌词/排行榜/本地/设置）+ 底部 PlaybackBar。
// 豆包风格设计 token（ngh_*），品牌位「逆光音乐」，Tabs/播放控制图标统一使用 sys.symbol.* 系统符号。
// 集成方式：作为 EntryAbility 的入口页面（main_pages.json 第一项）。
//
// 重构：原内联 MiniPlayer 已抽出为 components/PlaybackBar.ets，Index 仅负责 Tabs + 引用 PlaybackBar。

import { PlaybackBar } from '../components/PlaybackBar';
import { SearchPage } from './Search';
import { PlaylistPage } from './Playlist';
import { FavoritesPage } from './Favorites';
import { LyricsPage } from './Lyrics';
import { LeaderboardPage } from './Leaderboard';
import { LocalMusicPage } from './LocalMusic';
import { NasPage } from './Nas';
import { SettingsPage } from './Settings';

@Entry
@Component
struct Index {
  @State currentIndex: number = 0;

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

      // Tabs 占据剩余空间（PlaybackBar 在其下方常驻渲染）
      Tabs({ barPosition: BarPosition.End, index: this.currentIndex }) {
        TabContent() { SearchPage() }.tabBar(this.tabBuilder('搜索', 0))
        TabContent() { PlaylistPage() }.tabBar(this.tabBuilder('播放列表', 1))
        TabContent() { FavoritesPage() }.tabBar(this.tabBuilder('收藏', 2))
        TabContent() { LyricsPage() }.tabBar(this.tabBuilder('歌词', 3))
        TabContent() { LeaderboardPage() }.tabBar(this.tabBuilder('排行榜', 4))
        TabContent() { LocalMusicPage() }.tabBar(this.tabBuilder('本地', 5))
        TabContent() { NasPage() }.tabBar(this.tabBuilder('NAS', 6))
        TabContent() { SettingsPage() }.tabBar(this.tabBuilder('设置', 7))
      }
      .layoutWeight(1)
      .barMode(BarMode.Scrollable)
      .animationMode(AnimationMode.CONTENT_ANIMATION)
      .onChange((index: number) => {
        animateTo({ duration: 200, curve: Curve.EaseInOut }, () => {
          this.currentIndex = index;
        });
      })

      // 底部播放控制条（仅在持有可播放曲目时显示）
      PlaybackBar()
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
      case 3: return $r('sys.symbol.text_alignleft');
      case 4: return $r('sys.symbol.trophy');
      case 5: return $r('sys.symbol.folder');
      case 6: return $r('sys.symbol.externaldrive');
      case 7: return $r('sys.symbol.gearshape');
      default: return $r('sys.symbol.circle');
    }
  }
}
