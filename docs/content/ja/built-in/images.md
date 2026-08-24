---
title: 画像
description: 図、キャプション、遅延読み込み、安全な width / height。
---

# 画像

Markdown の画像は、オプトインするまで既定のレンダラのままです。`images` を有効にすると遅延読み込み、title のキャプション化、安全な `width` / `height` が使えます。

| オプション | 型                         | 既定    |
| ---------- | -------------------------- | ------- |
| `images`   | `boolean` / `ImageOptions` | `false` |

```ts
oxContent({
  images: true,
});
```

`true` と `{}` は同じデフォルトです。追加のつまみは `lazy`（既定 `true`）だけです。

```ts
oxContent({
  images: { lazy: false },
});
```

title のない画像は `loading="lazy"` の `<img>` になります。title があると figure + caption になります。危険な属性やスキームは落ちます。

## 関連

- [英語版ガイド](/built-in/images.md)
