---
title: ファイルツリー
description: file-tree フェンスから作る、オプトインの静的ディレクトリ図。
---

# ファイルツリー

`file-tree` フェンスはオプトインです。有効にすると、言語が `file-tree` のフェンスブロックが静的 HTML のツリーになります。名前はエスケープされます。実際のファイルシステムは読みません。

| オプション | 型                            | 既定    |
| ---------- | ----------------------------- | ------- |
| `fileTree` | `boolean` / `FileTreeOptions` | `false` |

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

このサイトは `fileTree` をオンにしているので、次のブロックはライブのツリーです。

```file-tree
- src/
  - index.ts **
  - lib/
    - util.ts
- …
```

- `/` で終わる名前はディレクトリです。
- それ以外の名前はファイルです。
- 末尾の ` **`、または `**name**` で囲むとその項目を強調します。
- `…` または `...` はプレースホルダで、他の名前と同じくエスケープされます。
- インデントは 1 レベルあたりスペース 2 つです。余分な空行は無視します。

````md
```file-tree
- src/
  - index.ts **
  - lib/
    - util.ts
- …
```
````

`true` または `{}` で既定がオンになります。オブジェクト形式では、オプション自体は残したまま変換だけ切るときに `enabled: false` を渡せます。

他のフェンスの中、インデントコード、インラインコードにあるフェンスはそのままです。

## 関連

- [ファイル取り込み](./includes.md)
- [カスタムコンテナ](./containers.md)
- [組み込み機能の一覧](../built-in-features.md)
