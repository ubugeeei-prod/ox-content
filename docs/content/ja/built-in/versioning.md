---
title: ドキュメントのバージョン管理
description: オプトインの URL プレフィックス、凍結スナップショット、ヘッダーのバージョンドロップダウン。
---

# ドキュメントのバージョン管理

`versions` を有効にすると、公開中のドキュメントツリーの横に凍結スナップショットを置き、ヘッダーでバージョンを切り替えられます。

省略または `false` ではオフです。バージョニングは **ディスク上のコンテンツを複製** します。過去スナップショットのディレクトリはビルド時に読むだけで、書き換えません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      versions: {
        current: "3.0.0-alpha",
        entries: [
          {
            id: "3.0.0-alpha",
            label: "3.0.0-alpha",
            prefix: "",
            banner: "unreleased",
          },
          {
            id: "2.90.0",
            label: "2.90.0",
            prefix: "2.90",
            dir: "versions/2.90",
          },
        ],
      },
    }),
  ],
};
```

`true` は `Latest` という現在エントリだけを有効にします。オブジェクトはオンにしたうえで指定したフィールドだけ上書きします。

| オプション | 型                            | 既定                             |
| ---------- | ----------------------------- | -------------------------------- |
| `versions` | `boolean` / `VersionsOptions` | `false`                          |
| `current`  | `string`                      | 先頭エントリ、または `"current"` |
| `switcher` | `boolean`                     | `true`                           |
| `badge`    | `boolean`                     | `true`                           |
| `entries`  | `VersionEntry[]`              | 現在の `Latest` 1 件             |

各エントリ:

| フィールド | 意味                                                                     |
| ---------- | ------------------------------------------------------------------------ |
| `id`       | `current` が参照する安定キー                                             |
| `label`    | ドロップダウン表示（HTML エスケープ）                                    |
| `prefix`   | `2.90` や `next` などの URL セグメント。空文字はサイトルート             |
| `dir`      | Vite ルートからのスナップショットディレクトリ。省略時はライブの `srcDir` |
| `banner`   | `"unreleased"` / `"unmaintained"` / 省略                                 |

プレフィックス付きツリーの検索は、ルートの `search-index.json` ではなく `{prefix}/search-index.json` を読みます。`javascript:` / `data:` / `vbscript:` / `//` / `..` は拒否します。

凍結スナップショット内では、安全な内部サイドバー・ヘッダーリンクも同じバージョンプレフィックス内に留まります。生成・手動サイドバー、ネスト、permalink、frontmatter alias、設定済み redirect、パンくずリストのルート、前後ページリンクを同じ規則で解決します。ロケールを先に解決するため、`/2.90/ja/` から翻訳済み sibling へ移動しても `2.90` と `ja` の両方を維持します。

スナップショットに遷移先が存在しない場合は、公開中のルートへ戻さず、そのバージョンのルート（例: `/2.90/`）へフォールバックします。外部 URL、`mailto:`、ページ内リンク、危険な scheme、protocol-relative URL にはバージョンプレフィックスを付けません。公開中のツリーは従来どおりプレフィックスなしです。

git タグからスナップショットを作り直す:

```bash
node scripts/snapshot-docs-version.mjs --tag v2.90.0 --prefix 2.90
```

このサイトでは最新の 2.x（`2.90.0`）を `/2.90/` に凍結し、ルートは 3.0.0-alpha の開発中ドキュメントです。

## 関連

- [ロケールスイッチャー](./locale-switcher.md)
- [英語版ガイド](/built-in/versioning.md)
