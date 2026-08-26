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

## クエリ文法

Rust の BM25 エンジンと生成される `virtual:ox-content/search` ランタイムは、同じ正規化済みクエリモデルを使います。入力された本文と絞り込みを分けるので、UI は安定したチップを出せ、エンジンは順位の理由を説明できます。

| クエリ           | 意味                                                    |
| ---------------- | ------------------------------------------------------- |
| `install cli`    | BM25 の完全一致 posting を採点する通常の単語。          |
| `"static index"` | その連続した文字列を含む文書をブーストするフレーズ。    |
| `render*`        | 最後のトークンでなくても効く明示的なプレフィックス。    |
| `@api`           | 文書 ID または URL のパス区切りから作るスコープ。       |
| `scope:api`      | 同じスコープ絞り込みのフィルタ表記。                    |
| `lang:ja`        | ロケール絞り込み。`language:` と `locale:` も同じです。 |
| `version:2.90`   | バージョン絞り込み。`v:` も同じです。                   |

閉じていない引用符は入力中のフレーズとして扱うため、キーボードで絞り込んでいる途中や IME 変換中でも UI は状態を保てます。ホスト済み検索にはリクエスト本文で `rawQuery` と `parsedQuery` を渡します。ローカル検索では、静的文書のパスから分かる範囲でスコープ、ロケール、バージョンの絞り込みを直接適用します。

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

`publicKey` は `searchKey` の別名です。`endpoint` を省略すると、クライアントは `/search` に POST します。リクエストは `query`、`rawQuery`、`parsedQuery`、`limit`、`indexName` を持つ JSON `POST` で、ヘッダーに `x-app-id`、`x-index-name`、`x-search-key` が付きます。アダプタは `hits`（または `results`）を、ローカル検索と同じ `{ id, title, url, score, matches, snippet }` 形に写し、ホスト側が返した `metadata`、`ranking`、`ariaLabel` も保持します。

ホスト済み検索を選んでも `appId`、`indexName`、公開検索キーのいずれかが欠けていると、クライアントは閉じて失敗します。`search()` は空配列を返し、壊れたエンドポイントは呼びません。秘密はログに出しません。ローカルの `search-index.json` パスはそのままです。

プレースホルダとホットキーは、独自 UI 向けに引き続き `searchOptions` からです。

## クライアント API

既定の SSG テーマが検索 UI を配線します。独自 UI では、同じインデックスを仮想モジュール経由で任意のクライアントコードから使えます。

```ts
import {
  createSearchUiState,
  parseSearchQuery,
  search,
  searchOptions,
} from "virtual:ox-content/search";

const results = await search("code annotations", { limit: 5 });
const query = parseSearchQuery('@api "static index" lang:ja');
const ui = createSearchUiState(query.raw, results);

for (const result of results) {
  // { id, title, url, score, matches, snippet, metadata, ranking, ariaLabel }
  console.log(result.title, result.url, result.snippet);
}
```

- `search(query, options?)` は `provider` に応じてローカル BM25 またはホスト済みアダプタを使います。`options.limit`、`options.prefix`、`options.fuzzy` は呼び出しごとに設定済みの既定を上書きします。`fuzzy` はローカル専用で、巨大な静的インデックスの exact/prefix 経路を速いまま保つため既定ではオフです。`options.locale` は、`localeCodes` と `defaultLocale` も渡したときに結果を一つの言語に保ちます。`versionPrefixes` は、ロケール区分を読む前に文書パスから取り除きます。
- `searchOptions` は解決済みの `{ enabled, limit, prefix, placeholder, hotkey, provider }` を出すので、独自 UI がサイト設定を尊重できます。
- `@api transform` のようなスコープ付きクエリは、結果をサイトの一区画に限定します。
- 結果は基本フィールドに加えて、カード向けの `scopes`、`metadata`、`ranking`、`ariaLabel` を返します。`metadata` にはセクション文脈、有効なフィルタ、取得できた言語/バージョンが入ります。`ranking.reasons` は `title term match: install` や `body phrase match: static index` のような安定した文字列です。
- `parseSearchQuery(query)` は正規化済みの単語、フレーズ、プレフィックス、フィルタ、スコープを返します。`createSearchUiState(query, results, options)` は `empty`、`loading`、`no-results`、`results`、`composing` を扱い、`aria-activedescendant` 用のリストボックス向けカード ID を返します。
- `formatSearchResultForCard(result, index)` はローカル/ホスト済みの結果を、`role: "option"`、バッジ、アクセシブルラベルを持つ安定した結果カード用モデルへ変換します。

生成されるローカルランタイムには、36 KB のバイト予算をユニットテストで置いています。PR の benchmark/output-size レポートでは、コンテンツ増加によるペイロード増を見えるように、docs コーパスの `search-index.json` サイズも併記してください。

検索をオフにしても仮想モジュールは解決します。`search()` は空配列を返し、`searchOptions.enabled` は `false` です。独自 UI で条件付き import は不要です。

## 関連

- [サイト生成](./site-generation.md) — 既定の検索 UI を載せる SSG ビルド。
- [テーマ](../theming.md)
