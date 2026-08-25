---
title: ファイルツリー
description: file-tree フェンスから作る、オプトインの静的ディレクトリ図。
---

# ファイルツリー

`file-tree` フェンスはオプトインです。有効にすると、言語が `file-tree` のフェンスブロックが静的 HTML のツリーになります。名前はエスケープされます。実際のファイルシステムは読みません。子があるディレクトリは `<details>` / `<summary>` で開閉します。フォルダとファイルのアイコンは既定でオンで、サイト設定から差し替えできます。

| オプション             | 型                                | 既定    |
| ---------------------- | --------------------------------- | ------- |
| `fileTree`             | `boolean` / `FileTreeOptions`     | `false` |
| `fileTree.defaultOpen` | `boolean`                         | `true`  |
| `fileTree.icons`       | `boolean` / `FileTreeIconOptions` | `true`  |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      fileTree: true,
    }),
  ],
};
```

## 書き方

このサイトは `fileTree` をオンにしているので、次のブロックはライブのツリーです。ディレクトリをクリックすると開閉します。

```file-tree
- src/
  - index.ts **
  - lib/
    - util.ts
- empty/
- …
```

- `/` で終わる名前はディレクトリです。
- それ以外の名前はファイルです。
- 末尾の ` **`、または `**name**` で囲むとその項目を強調します。
- `…` または `...` はプレースホルダで、他の名前と同じくエスケープされます。
- インデントは 1 レベルあたりスペース 2 つです。余分な空行は無視します。
- 子があるディレクトリは最初から開いています。空のディレクトリは 1 行です。

````md
```file-tree
- src/
  - index.ts **
  - lib/
    - util.ts
- empty/
- …
```
````

`true` または `{}` で既定がオンになります。オブジェクト形式では、オプション自体は残したまま変換だけ切るときに `enabled: false` を渡せます。

他のフェンスの中、インデントコード、インラインコードにあるフェンスはそのままです。

## 開閉

子があるディレクトリは `<details>` になります。クライアントの JavaScript は不要です。最初から閉じるには `defaultOpen: false` を指定します。

```ts
oxContent({
  fileTree: {
    defaultOpen: false,
  },
});
```

## アイコン

変換がオンならアイコンもオンです。グリフなしにするには `icons: false` を渡します。

```ts
oxContent({
  fileTree: {
    icons: false,
  },
});
```

既定アイコンは、サイト設定の信頼できる SVG または CSS クラストークンで差し替えます。フェンスの名前は HTML として扱いません。

```ts
oxContent({
  fileTree: {
    icons: {
      folder: `<svg viewBox="0 0 16 16"><path fill="currentColor" d="M2 3h5l1 2h6v8H2z"/></svg>`,
      folderOpen: `<svg viewBox="0 0 16 16"><path fill="currentColor" d="M2 4h5l1 2H3l2 7h9L12 6H8z"/></svg>`,
      file: "codicon-file",
      files: {
        ts: `<svg viewBox="0 0 16 16"><path fill="currentColor" d="M3 2h7l3 3v9H3z"/></svg>`,
      },
    },
  },
});
```

`files` のキーは拡張子です。`.ts` も `ts` も `index.ts` に一致します。

## 関連

- [ファイル取り込み](./includes.md)
- [カスタムコンテナ](./containers.md)
- [組み込み機能の一覧](../built-in-features.md)
