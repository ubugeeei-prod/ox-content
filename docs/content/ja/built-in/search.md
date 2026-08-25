---
title: 検索
description: SSG ビルドでは既定でオンの、静的 BM25 検索インデックスとクライアント API。
---

# 検索

全文検索は既定でオンです。インデックスはビルド中に — Rust で、BM25 スコア付きで — 作られ、静的 JSON として配られるので、サーバ部品なしでどの静的ホストでも検索が動きます。

このサイトで試せます。<kbd>/</kbd> または <kbd>⌘K</kbd> を押すか、ヘッダーの検索ボックスをクリックしてください。

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

| オプション    | 既定                        | 目的                                                         |
| ------------- | --------------------------- | ------------------------------------------------------------ |
| `enabled`     | `true`                      | 完全に切るときは `search: false`。                           |
| `limit`       | `10`                        | クライアントが返す結果の上限。                               |
| `prefix`      | `true`                      | 最後のクエリトークンをプレフィックス一致（タイプアヘッド）。 |
| `fuzzy`       | `false`                     | ローカル BM25 で小さなタイプミスを拾う。                     |
| `placeholder` | `"Search documentation..."` | 既定テーマの入力プレースホルダ。                             |
| `hotkey`      | `"/"`                       | フォーカス用ホットキー。`""` で登録しない。                  |
| `provider`    | `"local"`                   | `"local"` は BM25 のまま。`"hosted"` はオプトイン。          |

インデックスは生成ページの横の `search-index.json` に書き、読者が初めて検索したときに遅延取得します。開発中はメモリから配信し、ページが変わるたびに作り直します。

サイトにロケールが2つ以上あるときは、ダイアログに **Language** の `<select>` が出て、今見ているページの言語が既定になります。**All languages** はインデックス全体を探します。ドキュメントバージョンが有効なら、**Version** の `<select>` がその版の `search-index.json` を読みます。どちらもネイティブの select なので、Tab・矢印・タイプアヘッド・Space・Enter が追加ウィジェットなしで動きます。

## ホスト済みプロバイダ

`provider` を `"hosted"` にしない限り、検索はローカル BM25 インデックスのままです。ホスト済みアダプタは、設定または環境変数からアプリケーション ID、インデックス名、**公開の検索専用キー** を受け取ります。書き込みキーや管理キーは渡さないでください。`adminKey`、`writeKey`、`apiKey` という名前のフィールドは拒否します。

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

| オプション  | 出典                                      | 目的                            |
| ----------- | ----------------------------------------- | ------------------------------- |
| `appId`     | 設定または `OX_CONTENT_SEARCH_APP_ID`     | ホスト済みアプリケーション ID。 |
| `indexName` | 設定または `OX_CONTENT_SEARCH_INDEX_NAME` | リモートインデックス名。        |
| `searchKey` | 設定または `OX_CONTENT_SEARCH_KEY`        | 公開の検索専用キー。            |
| `publicKey` | 設定または `OX_CONTENT_SEARCH_PUBLIC_KEY` | `searchKey` の別名。            |
| `endpoint`  | 設定または `OX_CONTENT_SEARCH_ENDPOINT`   | 検索クエリを受ける HTTP URL。   |

`publicKey` は `searchKey` の別名です。`endpoint` を省略すると、クライアントは `/search` に POST します。リクエストは `query`、`limit`、`indexName` を持つ JSON `POST` で、ヘッダーに `x-app-id`、`x-index-name`、`x-search-key` が付きます。アダプタは `hits`（または `results`）を、ローカル検索と同じ `{ id, title, url, score, matches, snippet }` 形に写します。

ホスト済み検索を選んでも `appId`、`indexName`、公開検索キーのいずれかが欠けていると、クライアントは閉じて失敗します。`search()` は空配列を返し、壊れたエンドポイントは呼びません。秘密はログに出しません。ローカルの `search-index.json` パスはそのままです。

プレースホルダとホットキーは、独自 UI 向けに引き続き `searchOptions` からです。

## クライアント API

既定の SSG テーマが検索 UI を配線します。独自 UI では、同じインデックスを仮想モジュール経由で任意のクライアントコードから使えます。

```ts
import { search, searchOptions } from "virtual:ox-content/search";

const results = await search("code annotations", { limit: 5 });

for (const result of results) {
  // { id, title, url, score, matches, snippet }
  console.log(result.title, result.url, result.snippet);
}
```

- `search(query, options?)` は `provider` に応じてローカル BM25 またはホスト済みアダプタを使います。`options.limit`、`options.prefix`、`options.fuzzy` は呼び出しごとに設定済みの既定を上書きします。`fuzzy` はローカル専用で、巨大な静的インデックスの exact/prefix 経路を速いまま保つため既定ではオフです。`options.locale` は、`localeCodes` と `defaultLocale` も渡したときに結果を一つの言語に保ちます。`versionPrefixes` は、ロケール区分を読む前に文書パスから取り除きます。
- `searchOptions` は解決済みの `{ enabled, limit, prefix, placeholder, hotkey, provider }` を出すので、独自 UI がサイト設定を尊重できます。
- `@api transform` のようなスコープ付きクエリは、結果をサイトの一区画に限定します。

検索をオフにしても仮想モジュールは解決します。`search()` は空配列を返し、`searchOptions.enabled` は `false` です。独自 UI で条件付き import は不要です。

## 関連

- [サイト生成](./site-generation.md) — 既定の検索 UI を載せる SSG ビルド。
- [テーマ](../theming.md)
