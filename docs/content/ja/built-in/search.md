---
title: 検索
description: SSG ビルドで既定オンの、静的 BM25 検索インデックスとクライアント API です。
---

# 検索

全文検索は既定で有効です。インデックスはビルド中に —
Rust で、BM25 スコア付きで — 組み立てられ、静的 JSON ファイルとして出荷されます。
そのため検索は、サーバー部品なしでどの静的ホストでも動きます。

このサイトで試してください。<kbd>/</kbd> または <kbd>⌘K</kbd> を押すか、ヘッダーの検索
ボックスをクリックします。

![このサイトの検索ダイアログ](/screenshots/search-modal.png)

## 設定

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      search: {
        limit: 8,
        hotkey: "/",
        placeholder: "Search documentation...",
      },
    }),
  ],
};
```

| オプション    | 既定                        | 目的                                           |
| ------------- | --------------------------- | ---------------------------------------------- |
| `enabled`     | `true`                      | 完全に切るには `search: false`。               |
| `limit`       | `10`                        | クライアントが返す結果の上限。                 |
| `prefix`      | `true`                      | 最後のクエリトークンを接頭辞一致（タイプアヘッド）。 |
| `placeholder` | `"Search documentation..."` | 既定テーマの入力プレースホルダー。             |
| `hotkey`      | `"/"`                       | フォーカスホットキー。`""` で登録をオプトアウト。 |
| `provider`    | `"local"`                   | `"local"` は BM25 のまま。`"hosted"` はオプトイン。 |

インデックスは生成ページの隣の `search-index.json` に書かれ、
読者が初めて検索したときに遅延取得されます。開発中はメモリから提供され、ページ変更に合わせて再構築されます。

## ホスト型プロバイダー

`provider` を `"hosted"` にしない限り、検索はローカル BM25 インデックスのままです。
ホスト型アダプターは、設定または環境変数からアプリケーション id、インデックス名、
**公開の検索専用キー** を受け付けます。書き込みや管理キーは渡さないでください。
`adminKey`、`writeKey`、`apiKey` という名前の欄は拒否されます。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      search: {
        provider: "hosted",
        appId: process.env.OX_CONTENT_SEARCH_APP_ID,
        indexName: process.env.OX_CONTENT_SEARCH_INDEX_NAME,
        searchKey: process.env.OX_CONTENT_SEARCH_KEY,
        endpoint: process.env.OX_CONTENT_SEARCH_ENDPOINT,
      },
    }),
  ],
};
```

| オプション  | 出典                                     | 目的                                   |
| ----------- | ---------------------------------------- | -------------------------------------- |
| `appId`     | 設定または `OX_CONTENT_SEARCH_APP_ID`    | ホスト型アプリケーション id。          |
| `indexName` | 設定または `OX_CONTENT_SEARCH_INDEX_NAME` | リモートインデックス名。              |
| `searchKey` | 設定または `OX_CONTENT_SEARCH_KEY`       | 公開の検索専用キー。                   |
| `publicKey` | 設定または `OX_CONTENT_SEARCH_PUBLIC_KEY` | `searchKey` のエイリアス。            |
| `endpoint`  | 設定または `OX_CONTENT_SEARCH_ENDPOINT`  | 検索クエリを受け取る HTTP URL。        |

`publicKey` は `searchKey` のエイリアスです。`endpoint` を省略すると、
クライアントは `/search` へ POST します。リクエストは `query`、
`limit`、`indexName` 付きの JSON `POST` で、加えて `x-app-id`、`x-index-name`、
`x-search-key` ヘッダーです。アダプターは `hits`（または `results`）を、ローカル検索と同じ
`{ id, title, url, score, matches, snippet }` 形へ写します。

ホスト型検索を選んだのに `appId`、`indexName`、または公開検索
キーがない場合、クライアントは fail closed します。`search()` は空配列を返し、
壊れたエンドポイントは呼びません。シークレットはログしません。ローカルの
`search-index.json` 経路は変わりません。

プレースホルダーとホットキーは、カスタム UI 向けにいまも `searchOptions` から来ます。

## クライアント API

既定の SSG テーマが検索 UI を配線します。カスタム UI では、同じ
インデックスが仮想モジュール経由でどのクライアントコードからも使えます。

```ts
import { search, searchOptions } from "virtual:ox-content/search";

const results = await search("code annotations", { limit: 5 });

for (const result of results) {
  // { id, title, url, score, matches, snippet }
  console.log(result.title, result.url, result.snippet);
}
```

- `search(query, options?)` は `provider` に応じてローカル BM25 またはホスト型アダプターを使います。
  `options.limit` と `options.prefix` は呼び出しごとに設定済み既定を上書きします。
- `searchOptions` は解決済みの
  `{ enabled, limit, prefix, placeholder, hotkey, provider }` を公開するので、カスタム UI は
  サイト設定を尊重できます。
- `@api transform` のようなスコープ付きクエリは、結果をサイトの一区画に制限します。

検索が無効でも仮想モジュールは解決します。`search()` は
空配列を返し、`searchOptions.enabled` は `false` です。そのためカスタム UI は
条件付き import が不要です。

## 関連

- [サイト生成](./site-generation.md) — 既定の検索 UI をホストする
  SSG ビルド。
- [テーマ](../theming.md)
