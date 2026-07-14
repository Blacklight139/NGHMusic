// 职责：HarmonyOS UIAbility 入口，加载 pages/Index 页面。
// 集成方式：在 module.json5 中作为 mainElement 注册。
//
// 链接 music-core（Rust via NAPI/napi-rs）步骤见 bridge/MusicCoreBridge.ets 顶部注释。
//
// HAR-004 配套：onDestroy 中调用 playerManager.release() 显式释放 AVPlayer，
//              保证 AVPlayer 状态机进入 released 状态，不依赖 GC。

import UIAbility from '@ohos.app.ability.UIAbility';
import type { AbilityConstant, Want } from '@ohos.app.ability.UIAbility';
import window from '@ohos.window';
import { playerManager } from '../player/PlayerManager';

export default class EntryAbility extends UIAbility {
  onCreate(want: Want, launchParam: AbilityConstant.LaunchParam): void {
    console.info('[MusicPlayer] EntryAbility onCreate');
  }

  onWindowStageCreate(windowStage: window.WindowStage): void {
    windowStage.loadContent('pages/Index', (err) => {
      if (err.code) {
        console.error(`[MusicPlayer] 加载页面失败: ${JSON.stringify(err)}`);
        return;
      }
      console.info('[MusicPlayer] 页面加载完成');
    });
  }

  onDestroy(): void {
    console.info('[MusicPlayer] EntryAbility onDestroy');
    // HAR-004：显式释放 AVPlayer，避免依赖 GC
    playerManager.release();
  }
}
