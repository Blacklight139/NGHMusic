// 职责：HarmonyOS 尺寸 token（豆包风格），与 Android/iOS/macOS 对齐。
// 圆角与间距统一引用，替代各页面硬编码值。

export class NghRadius {
  static readonly SM: number = 8;
  static readonly MD: number = 12;
  static readonly LG: number = 16;
  static readonly PILL: number = 999;
}

export class NghSpacing {
  static readonly S1: number = 4;
  static readonly S2: number = 8;
  static readonly S3: number = 12;
  static readonly S4: number = 16;
  static readonly S5: number = 20;
  static readonly S6: number = 24;
  static readonly S7: number = 32;
  static readonly S8: number = 40;
}

export class NghFontSize {
  static readonly CAPTION: number = 11;
  static readonly BODY_SM: number = 12;
  static readonly BODY: number = 14;
  static readonly TITLE_SM: number = 14;
  static readonly TITLE: number = 16;
  static readonly HEADLINE: number = 18;
}
