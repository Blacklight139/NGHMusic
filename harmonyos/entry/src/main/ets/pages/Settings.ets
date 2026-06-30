// 职责：设置页 ArkUI 组件，核心版本、音源导入（JSON）、本地目录、缓存，简约风格占位。
// 对齐桌面端 pages/settings.js：调用 MusicCoreBridge.importSource。

import { MusicCoreBridge } from '../bridge/MusicCoreBridge';

@Component
export struct SettingsPage {
  @State sourceJson: string = '';
  @State result: string = '';
  @State resultOk: boolean = false;
  @State version: string = '（未连接）';

  aboutToAppear(): void {
    this.version = MusicCoreBridge.appVersion();
  }

  build() {
    Column() {
      Text('设置').fontSize(22).fontWeight(FontWeight.Medium)
        .fontColor($r('app.color.text_primary'))
      Text('音源管理 / 缓存 / 本地目录').fontSize(12)
        .fontColor($r('app.color.text_muted')).margin({ top: 2 })

      Scroll() {
        Column() {
          // 版本
          Row() {
            Text('核心版本').fontSize(12).fontColor($r('app.color.text_muted'))
            Blank()
            Text(this.version).fontSize(12).fontColor($r('app.color.text_primary'))
          }
          .width('100%').margin({ top: 16 })

          // 音源导入
          Text('音源导入').fontSize(14).fontWeight(FontWeight.Medium)
            .fontColor($r('app.color.text_primary')).margin({ top: 24 })
          Text('粘贴标准音源 JSON Schema 内容').fontSize(12)
            .fontColor($r('app.color.text_muted')).margin({ top: 2 })
          TextArea({ text: this.sourceJson })
            .width('100%').height(160).margin({ top: 8 })
            .fontSize(12).fontFamily('monospace')
            .backgroundColor($r('app.color.bg'))
            .border({ width: 1, color: $r('app.color.border') })
            .borderRadius(8)
            .onChange((v: string) => { this.sourceJson = v; })
          Button('导入').margin({ top: 8 }).backgroundColor($r('app.color.primary'))
            .onClick(() => this.doImport())

          if (this.result.length > 0) {
            Text(this.result)
              .fontSize(12)
              .fontColor(this.resultOk ? $r('app.color.primary') : '#CC3333')
              .width('100%').padding(12).margin({ top: 8 })
              .backgroundColor($r('app.color.bg')).borderRadius(8)
              .border({ width: 1, color: this.resultOk ? $r('app.color.primary') : $r('app.color.border') })
          }

          // 本地目录
          Text('本地音乐目录').fontSize(14).fontWeight(FontWeight.Medium)
            .fontColor($r('app.color.text_primary')).margin({ top: 24 })
          Text('暂无目录，将在此添加 / 移除本地目录并触发扫描')
            .fontSize(12).fontColor($r('app.color.text_muted'))
            .width('100%').padding(16).margin({ top: 8 })
            .backgroundColor($r('app.color.bg')).borderRadius(8)
            .border({ width: 1, color: $r('app.color.border') })

          // 缓存
          Text('播放缓存').fontSize(14).fontWeight(FontWeight.Medium)
            .fontColor($r('app.color.text_primary')).margin({ top: 24 })
          Text('缓存容量与清理功能将在此提供')
            .fontSize(12).fontColor($r('app.color.text_muted'))
            .width('100%').padding(16).margin({ top: 8 })
            .backgroundColor($r('app.color.bg')).borderRadius(8)
            .border({ width: 1, color: $r('app.color.border') })
        }
      }
      .layoutWeight(1).margin({ top: 16 })
    }
    .width('100%').height('100%').padding(16)
    .backgroundColor($r('app.color.bg_alt'))
  }

  private doImport(): void {
    if (this.sourceJson.length === 0) {
      this.result = '请先粘贴音源 JSON';
      this.resultOk = false;
      return;
    }
    this.result = MusicCoreBridge.importSource(this.sourceJson);
    this.resultOk = true;
  }
}
