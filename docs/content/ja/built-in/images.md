---
title: 画像
description: オプトインの図、キャプション、遅延読み込み、安全な width / height 属性。
---

# 画像

Markdown の画像は、オプトインするまで既定のレンダラのままです。`images` をオンにすると、遅延読み込みを付け、title をキャプションにし、安全な `width` / `height` 属性を受け付けます。

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

`true` と `{}` はどちらも既定でオンです。追加のつまみは `lazy` だけです（既定 `true`）。

```ts
oxContent({
  images: { lazy: false },
});
```

## 遅延読み込み

title のない Markdown 画像は、`loading="lazy"` 付きの `<img>` になります。

```md
![Diagram](/architecture.png)
```

![Ox Content](/logo-icon.svg)

## キャプション

title がキャプションです。画像は `<figure class="ox-figure">` で包み、title はエスケープして `<figcaption>` に入れます。

```md
![Diagram](/architecture.png "The transform pipeline")
```

![Ox Content](/logo-icon.svg "The Ox Content mark")

## 寸法

任意の末尾 `{width=N height=M}` はこの機能が消費します。`attrs` は不要です。符号なし整数だけを受け付け、それ以外は捨てます。

```md
![Diagram](/architecture.png){width=320 height=180}
```

## 安全性

代替テキスト、キャプション、`src` は HTML エスケープされます。`javascript:`、`data:`、`vbscript:`、またはプロトコル相対の `//` を使う行き先では、その値を持つ `<img src>` を出しません。フェンスコード、インデントコード、インラインコードの中の画像はそのままです。

## 関連

- [構文拡張](./syntax-extensions.md)
- [組み込み機能の一覧](../built-in-features.md)
