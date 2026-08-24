---
title: Markdown の土台
description: 既定でオンの GFM、frontmatter、目次。
---

# Markdown の土台

よく使う GitHub Flavored Markdown は既定で有効です。このページの機能に追加設定は不要です。

| オプション      | 型        | 既定         | 役割                   |
| --------------- | --------- | ------------ | ---------------------- |
| `gfm`           | `boolean` | `true`       | GFM 拡張               |
| `tables`        | `boolean` | `true`       | GFM 表                 |
| `taskLists`     | `boolean` | `true`       | `- [ ]` / `- [x]`      |
| `strikethrough` | `boolean` | `true`       | `~~text~~`             |
| `autolinks`     | `boolean` | `gfm` に追随 | 裸 URL をリンク化      |
| `footnotes`     | `boolean` | `true`       | `[^1]` 脚注            |
| `frontmatter`   | `boolean` | `true`       | YAML frontmatter       |
| `toc`           | `boolean` | `true`       | 見出しから目次         |
| `tocMaxDepth`   | `number`  | `3`          | TOC に含める最深レベル |

上はすべて CommonMark の上の拡張で、オプトアウトです。コアは [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/) の 652 例を CI で確認します。見出しには slug の `id` が付くため、マークアップはバイト一致ではありません。数値は [性能 (英語)](/performance.md#commonmark-conformance) を見てください。

より厳しい CommonMark に寄せるときは明示的に切ります。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      strikethrough: false,
      taskLists: false,
    }),
  ],
};
```

## 表

```md
| 機能         | 状態 |
| ------------ | ---- |
| 表           | 既定 |
| タスクリスト | 既定 |
```

## タスクリスト

```md
- [x] Rust で Markdown をパース
- [ ] 世界征服
```

## 打ち消し線

```md
~~古い API~~ は使わない。
```

## 脚注

```md
本文[^note]

[^note]: 脚注の本文。
```

## Frontmatter

```md
---
title: はじめに
description: 最短の導入。
---
```

実例つきの英語ガイドは [Markdown Baseline](/built-in/markdown.md) です。
