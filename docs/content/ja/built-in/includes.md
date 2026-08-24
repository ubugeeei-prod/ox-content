---
title: ファイル取り込み
description: 共有スニペット向けのオプトイン Markdown include。
---

# ファイル取り込み

Markdown のファイル取り込みはオプトインです。有効にすると、HTML コメントの指示がホスト文書のパース前にもう一つの Markdown をインラインします。

| オプション | 型                           | 既定    |
| ---------- | ---------------------------- | ------- |
| `includes` | `boolean` / `IncludeOptions` | `false` |

```ts
oxContent({
  includes: true,
});
```

```md
<!-- @include: ./shared/warning.md -->
```

パスは引用符で囲んでもよく、前後の空白は落ちます。ホストの外や `..` を含むパスは拒否されます。循環 include はエラーです。

## 関連

- [英語版ガイド](/built-in/includes.md)
