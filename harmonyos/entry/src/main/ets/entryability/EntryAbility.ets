// 职责：HarmonyOS UIAbility 入口，加载 pages/Index 页面。
// 集成方式：在 module.json5 中作为 mainElement 注册。
//
// 链接 music-core（Rust via NAPI/napi-rs）步骤见 bridge/MusicCoreBridge.ets 顶部注释。

import UIAbility from '@ohos.app.ability.UIAbility';
import type { AbilityConstant, Want } from '@ohos.app.ability.UIAbility';
import window from '@ohos.window';

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
  }
}
