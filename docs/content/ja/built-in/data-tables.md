---
title: データテーブル
description: csv-table / json-table フェンスから作る、オプトインの静的テーブル。
---

# データテーブル

`csv-table` と `json-table` フェンスはオプトインです。有効にすると、それらの
フェンスが静的 HTML の表になります。セルのテキストはエスケープされます。
クライアント JavaScript は不要です。無効時は普通のコードブロックのままです。

| オプション           | 型                             | 既定         |
| -------------------- | ------------------------------ | ------------ |
| `dataTables`         | `boolean` / `DataTableOptions` | `false`      |
| `dataTables.rootDir` | `string`                       | プロジェクト |
| `dataTables.missing` | `"error"` / `"warn"`           | `"error"`    |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      dataTables: true,
    }),
  ],
};
```

## 書き方

このサイトは `dataTables` をオンにしているので、次のブロックはライブの表です。

```csv-table title="Options"
Option,Type,Default
highlight,boolean,false
```

````md
```csv-table title="Options"
Option,Type,Default
highlight,boolean,false
```
````

- フェンス言語は `csv-table` と `json-table` です。
- CSV の先頭行が見出しです。引用符で囲んだフィールドにはカンマを含められます。
- JSON はオブジェクトの配列、配列の配列、または
  `{ "headers": [...], "rows": [[...]] }` です。
- `title="..."` は `<caption>` になります。
- 他のフェンスの中、インデントコード、インラインコードは変換しません。

## ファイルを取り込む

インラインが動いたら、フェンスからファイルを読めます。`@/` と `/` は
`rootDir`（未指定なら Vite のプロジェクトルート）から解決します。相対パスは
今の Markdown ファイルから解決します。`..` でそのルートから外には出られません。

````md
```csv-table src="@/content/data/options.csv" title="From CSV"

```
````

```csv-table src="@/content/data/options.csv" title="From CSV"

```

フェンス本体がパス 1 行だけのときも取り込みです。

````md
```json-table
./options.json
```
````

```json-table
./options.json
```

欠けたファイルは `missing` に従います。既定は `"error"`（変換診断として報告）
です。`"warn"` はフェンスをコードブロックのまま残し、エラーにはしません。
壊れた CSV/JSON は常に対処できる診断を出します。

```ts
oxContent({
  dataTables: {
    rootDir: "docs",
    missing: "warn",
  },
});
```

## 関連

- [ファイルツリー](./file-tree.md)
- [ファイル取り込み](./includes.md)
- [Markdown の土台](./markdown.md)
- [組み込み機能の概要](../built-in-features.md)
