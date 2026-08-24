---
title: ファイルツリー
description: file-tree フェンスから静的なディレクトリ図を作るオプトイン。
---

# ファイルツリー

`file-tree` フェンスはオプトインです。言語 `file-tree` のフェンスが静的 HTML の木になります。名前はエスケープされ、実ファイルシステムは読みません。

| オプション | 型                            | 既定    |
| ---------- | ----------------------------- | ------- |
| `fileTree` | `boolean` / `FileTreeOptions` | `false` |

```ts
oxContent({
  fileTree: true,
});
```

````md
```file-tree
- src/
  - index.ts **
  - lib/
    - util.ts
- …
```
````

`/` で終わる名前はディレクトリです。`**` は強調、`…` は省略です。

## 関連

- [英語版ガイド](/built-in/file-tree.md)
