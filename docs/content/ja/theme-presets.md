---
title: テーマプリセット
description: 公式カタログ。形はスキン、色は配色パッケージ。
---

# テーマプリセット

このページと [テーマギャラリー](/theme-gallery.html) が、公開済みスキンと配色の公式カタログです。どのスキンもどの配色と組めます。

テーマは独立した 2 軸です。

- **スキン** (`@ox-content/theme-*`) は形です。幾何、質感、タイポ、モーション。色は `--octc-*` だけを参照し、色名を直書きしません。
- **配色** (`@ox-content/theme-color-*`) は色だけです。light と dark のパレット。レイアウトやフォントは持ちません。

**27 スキン × 45 配色 = 1215 通り** です。

## ギャラリー

**[組み合わせを見る →](/theme-gallery.html)**

ギャラリーは本物の SSG スタイルシートを iframe で描画します。light / dark、ランディング、記事、WebGL 背景まで、ビルド後の見た目です。

```ts
import { oxContent, defineTheme } from "@ox-content/vite-plugin";
import pixel from "@ox-content/theme-pixel";
import tokyoNight from "@ox-content/theme-color-tokyo-night";

oxContent({
  ssg: {
    theme: defineTheme([pixel, tokyoNight]),
  },
});
```

パッケージの互換契約（必須トークン、light / dark、スクリーンショット、スキンは色を直書きしない）は [英語の Theme Presets](/theme-presets.md#authoring-a-package) にあります。見た目の上書きは [テーマ](./theming.md) です。
