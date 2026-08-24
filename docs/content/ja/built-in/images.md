---
title: 画像
description: オプトインの図、キャプション、遅延読み込み、安全な width / height 属性です。
---

# 画像

Markdown 画像は、オプトインするまで既定のレンダラーのままです。`images` を有効にすると
遅延読み込みを足し、title テキストをキャプションにし、安全な
`width` / `height` 属性を受け付けます。

| オプション | 型                         | 既定    |
| ---------- | -------------------------- | ------- |
| `images`   | `boolean` / `ImageOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      images: true,
    }),
  ],
};
```

`true` と `{}` はどちらも既定を有効にします。追加のつまみは `lazy`
だけです（既定 `true`）。

```ts
oxContent({
  images: { lazy: false },
});
```

## 遅延読み込み画像

title のない Markdown 画像は、`loading="lazy"` 付きの `<img>` になります。

```md
![Diagram](/architecture.png)
```

![Ox Content](/logo-icon.svg)

## キャプション

title テキストがキャプションです。画像は
`<figure class="ox-figure">` で包まれ、title はエスケープされて `<figcaption>` に入ります。

```md
![Diagram](/architecture.png "The transform pipeline")
```

![Ox Content](/logo-icon.svg "Ox Content のマーク")

## 寸法

任意の末尾 `{width=N height=M}` はこの機能が消費します。
`attrs` は不要です。符号なし整数だけを受け付け、それ以外は捨てられます。

```md
![Diagram](/architecture.png){width=320 height=180}
```

## 安全性

代替テキスト、キャプション、`src` 値は HTML エスケープされます。`javascript:`、
`data:`、`vbscript:`、またはプロトコル相対の `//` を使う宛先は、その値で
`<img src>` を出しません。フェンス付きコード、インデントコード、
インラインコード内の画像はそのままです。

## 関連

- [構文拡張](./syntax-extensions.md)
- [組み込み機能の概要](../built-in-features.md)
