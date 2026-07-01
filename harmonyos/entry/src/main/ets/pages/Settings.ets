// 职责：设置页 ArkUI 组件。音源管理（LXMusic 风格列表 + 文件导入）、核心版本、本地目录、缓存。
// 音源管理调用 MusicCoreBridge 的 SourceManager 方法；文件导入使用 @ohos.file.picker + @ohos.file.fs。

import { MusicCoreBridge, SourceInfo } from '../bridge/MusicCoreBridge';
import picker from '@ohos.file.picker';
import fs from '@ohos.file.fs';
import util from '@ohos.util';

@Component
export struct SettingsPage {
  @State version: string = '（未连接）';
  @State sources: SourceInfo[] = [];
  @State expandedId: string = '';
  @State feedback: string = '';
  @State feedbackOk: boolean = false;

  aboutToAppear(): void {
    this.version = MusicCoreBridge.appVersion();
    this.refreshSources();
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

          // 音源管理
          this.SourceManagementCard()

          // 本地音乐目录
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

  @Builder
  SourceManagementCard() {
    Column() {
      // 标题行
      Row() {
        Column() {
          Text('音源管理').fontSize(16).fontWeight(FontWeight.Medium)
            .fontColor($r('app.color.ngh_text_primary'))
          Text(`已导入 ${this.sources.length} 个音源`).fontSize(11)
            .fontColor($r('app.color.ngh_text_tertiary')).margin({ top: 2 })
        }.alignItems(HorizontalAlign.Start)

        Blank()

        // 导入按钮（系统符号，无 emoji）
        SymbolGlyph($r('sys.symbol.square_and_arrow_down'))
          .fontSize(20)
          .fontColor([$r('app.color.ngh_text_primary')])
          .padding(8)
          .backgroundColor($r('app.color.ngh_surface_alt'))
          .borderRadius(20)
          .onClick(() => this.pickAndImportFile())
      }
      .width('100%')

      // 反馈条
      if (this.feedback.length > 0) {
        Text(this.feedback)
          .fontSize(12)
          .fontColor(this.feedbackOk ? $r('app.color.ngh_success') : $r('app.color.ngh_danger'))
          .width('100%').padding(10).margin({ top: 8 })
          .backgroundColor($r('app.color.ngh_surface_alt')).borderRadius(8)
      }

      // 列表 / 空状态
      if (this.sources.length === 0) {
        Text('暂无音源，点击右上角导入')
          .fontSize(12).fontColor($r('app.color.ngh_text_tertiary'))
          .width('100%').padding({ top: 32, bottom: 32 })
          .textAlign(TextAlign.Center)
      } else {
        List() {
          ForEach(this.sources, (item: SourceInfo, index: number) => {
            ListItem() {
              SourceItemRow({
                item: item,
                isExpanded: this.expandedId === item.id,
                isFirst: index === 0,
                isLast: index === this.sources.length - 1,
                onToggleExpand: () => {
                  this.expandedId = this.expandedId === item.id ? '' : item.id;
                },
                onToggleEnabled: (on: boolean) => this.setEnabled(item.id, on),
                onMoveUp: () => this.moveUp(index),
                onMoveDown: () => this.moveDown(index),
                onDelete: () => this.confirmDelete(item),
              })
            }
            .animation({ duration: 200, curve: Curve.EaseOut })
            .transition({ type: TransitionType.All, opacity: 0, translate: { y: 12 } })
          }, (item: SourceInfo) => item.id)
        }
        .width('100%').margin({ top: 8 })
        .divider({ strokeWidth: 1, color: $r('app.color.ngh_border_soft'), startMargin: 8, endMargin: 8 })
      }
    }
    .width('100%').margin({ top: 24 })
    .padding(12)
    .backgroundColor($r('app.color.ngh_surface'))
    .borderRadius(12)
    .border({ width: 1, color: $r('app.color.ngh_border') })
    .shadow({ radius: 8, color: '#14000000', offsetX: 0, offsetY: 2 })
  }

  private refreshSources(): void {
    this.sources = MusicCoreBridge.listSourcesOrdered();
  }

  private setFeedback(msg: string, ok: boolean): void {
    this.feedback = msg;
    this.feedbackOk = ok;
  }

  private setEnabled(id: string, on: boolean): void {
    MusicCoreBridge.setSourceEnabled(id, on);
    // 用新对象引用替换被切换项，确保 @Prop 子组件刷新
    this.sources = this.sources.map((s: SourceInfo): SourceInfo => {
      if (s.id === id) {
        return {
          id: s.id, name: s.name, version: s.version, enabled: on,
          sourceType: s.sourceType, priority: s.priority, description: s.description,
        };
      }
      return s;
    });
  }

  private moveUp(index: number): void {
    if (index <= 0) return;
    const list = this.sources.slice();
    const tmp = list[index - 1];
    list[index - 1] = list[index];
    list[index] = tmp;
    this.sources = list;
    MusicCoreBridge.reorderSources(list.map((s: SourceInfo) => s.id));
  }

  private moveDown(index: number): void {
    if (index >= this.sources.length - 1) return;
    const list = this.sources.slice();
    const tmp = list[index + 1];
    list[index + 1] = list[index];
    list[index] = tmp;
    this.sources = list;
    MusicCoreBridge.reorderSources(list.map((s: SourceInfo) => s.id));
  }

  private confirmDelete(item: SourceInfo): void {
    AlertDialog.show({
      title: '删除音源',
      message: `确定删除音源「${item.name}」？`,
      primaryButton: {
        value: '取消',
        action: () => {}
      },
      secondaryButton: {
        value: '删除',
        fontColor: $r('app.color.ngh_danger'),
        action: () => {
          MusicCoreBridge.deleteSource(item.id);
          this.refreshSources();
          this.setFeedback(`已删除音源「${item.name}」`, true);
        }
      }
    });
  }

  private pickAndImportFile(): void {
    const documentSelectOptions = new picker.DocumentSelectOptions();
    documentSelectOptions.maxSelectNumber = 1;
    documentSelectOptions.fileSuffixFilters = ['.json'];
    const documentPicker = new picker.DocumentViewPicker();
    documentPicker.select(documentSelectOptions).then((uris: string[]) => {
      if (uris === undefined || uris === null || uris.length === 0) {
        return;
      }
      const uri = uris[0];
      let file: fs.File | undefined;
      try {
        file = fs.openSync(uri, fs.OpenMode.READ_ONLY);
        const stat = fs.statSync(file.fd);
        const ab = new ArrayBuffer(stat.size);
        fs.readSync(file.fd, ab);
        const text = util.TextDecoder.create('utf-8').decodeWithStream(new Uint8Array(ab));
        const info = MusicCoreBridge.importSourceFromJson(text);
        if (info) {
          this.refreshSources();
          this.setFeedback(`已导入音源「${info.name}」`, true);
        } else {
          this.setFeedback('导入失败：JSON 解析错误', false);
        }
      } catch (e) {
        this.setFeedback('导入失败：' + (e as Error).message, false);
      } finally {
        if (file !== undefined) {
          fs.closeSync(file);
        }
      }
    }).catch(() => {
      // 用户取消或选择失败，不提示
    });
  }
}

// 音源列表项（单独组件，确保展开/开关状态可响应式刷新）
@Component
struct SourceItemRow {
  @Prop item: SourceInfo;
  @Prop isExpanded: boolean;
  @Prop isFirst: boolean;
  @Prop isLast: boolean;
  @State pressed: boolean = false;
  onToggleExpand: () => void = () => {};
  onToggleEnabled: (on: boolean) => void = () => {};
  onMoveUp: () => void = () => {};
  onMoveDown: () => void = () => {};
  onDelete: () => void = () => {};

  build() {
    Column() {
      Row() {
        // 拖动手柄
        SymbolGlyph($r('sys.symbol.line_3_horizontal'))
          .fontSize(16)
          .fontColor([$r('app.color.ngh_text_tertiary')])

        // 名称 + 来源标签
        Column() {
          Text(this.item.name)
            .fontSize(14).fontWeight(FontWeight.Medium)
            .fontColor($r('app.color.ngh_text_primary'))
            .maxLines(1).textOverflow({ overflow: TextOverflow.Ellipsis })
          Text(this.item.sourceType)
            .fontSize(10)
            .fontColor($r('app.color.ngh_surface'))
            .padding({ left: 6, right: 6, top: 2, bottom: 2 })
            .backgroundColor(this.tagColor(this.item.sourceType))
            .borderRadius(4)
            .margin({ top: 4 })
        }
        .alignItems(HorizontalAlign.Start)
        .layoutWeight(1).margin({ left: 8 })

        // 启用开关
        Toggle({ type: ToggleType.Switch, isOn: this.item.enabled })
          .selectedColor($r('app.color.ngh_primary'))
          .margin({ right: 8 })
          .onChange((on: boolean) => this.onToggleEnabled(on))

        // 上移
        SymbolGlyph($r('sys.symbol.chevron_up'))
          .fontSize(16)
          .fontColor([this.isFirst ? $r('app.color.ngh_text_tertiary') : $r('app.color.ngh_text_secondary')])
          .opacity(this.isFirst ? 0.4 : 1)
          .margin({ right: 6 })
          .onClick(() => { if (!this.isFirst) this.onMoveUp(); })

        // 下移
        SymbolGlyph($r('sys.symbol.chevron_down'))
          .fontSize(16)
          .fontColor([this.isLast ? $r('app.color.ngh_text_tertiary') : $r('app.color.ngh_text_secondary')])
          .opacity(this.isLast ? 0.4 : 1)
          .margin({ right: 6 })
          .onClick(() => { if (!this.isLast) this.onMoveDown(); })

        // 删除
        SymbolGlyph($r('sys.symbol.trash'))
          .fontSize(16)
          .fontColor([$r('app.color.ngh_danger')])
          .onClick(() => this.onDelete())
      }
      .width('100%')
      .padding({ top: 10, bottom: 10 })

      // 展开详情
      if (this.isExpanded) {
        Column() {
          this.DetailRow('ID', this.item.id)
          this.DetailRow('版本', this.item.version)
          this.DetailRow('类型', this.item.sourceType)
          this.DetailRow('优先级', String(this.item.priority))
          if (this.item.description.length > 0) {
            this.DetailRow('描述', this.item.description)
          }
        }
        .width('100%')
        .padding({ top: 8, bottom: 8, left: 24, right: 8 })
        .backgroundColor($r('app.color.ngh_surface_alt'))
        .borderRadius(8)
        .margin({ top: 4 })
      }
    }
    .onClick(() => this.onToggleExpand())
    .onTouch((event: TouchEvent) => {
      if (event.type === TouchType.Down) {
        this.pressed = true;
      } else if (event.type === TouchType.Up || event.type === TouchType.Cancel) {
        this.pressed = false;
      }
    })
    .scale(this.pressed ? { x: 0.97, y: 0.97 } : { x: 1, y: 1 })
    .animation({ duration: 150, curve: Curve.EaseOut })
  }

  @Builder
  DetailRow(label: string, value: string) {
    Row() {
      Text(label).fontSize(11).fontColor($r('app.color.ngh_text_tertiary')).width(56)
      Text(value).fontSize(11).fontColor($r('app.color.ngh_text_secondary'))
        .layoutWeight(1).maxLines(2).textOverflow({ overflow: TextOverflow.Ellipsis })
    }.width('100%').margin({ top: 2, bottom: 2 })
  }

  private tagColor(t: string): Resource {
    switch (t) {
      case 'json': return $r('app.color.ngh_primary');
      case 'community': return $r('app.color.ngh_warning');
      case 'local': return $r('app.color.ngh_success');
      default: return $r('app.color.ngh_text_tertiary');
    }
  }
}
