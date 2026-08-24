---
title: 検索
description: 既定の静的 BM25 インデックスとクライアント API。
---

# 検索

全文検索は既定でオンです。インデックスはビルド時に Rust で BM25 付きで作られ、静的 JSON として配られます。サーバ部品は不要です。

`/` または `⌘K`、ヘッダーの検索欄から試せます。

```ts
oxContent({
  search: {
    limit: 8,
    hotkey: "/",
    placeholder: "ドキュメントを検索…",
  },
});
```

| オプション    | 既定                        | 役割                                       |
| ------------- | --------------------------- | ------------------------------------------ |
| `enabled`     | `true`                      | `search: false` で完全オフ                 |
| `limit`       | `10`                        | クライアントが返す件数                     |
| `prefix`      | `true`                      | 末尾トークンの前方一致                     |
| `placeholder` | `"Search documentation..."` | デフォルトテーマのプレースホルダ           |
| `hotkey`      | `"/"`                       | フォーカス。`""` で登録しない              |
| `provider`    | `"local"`                   | `"local"` は BM25。`"hosted"` はオプトイン |

インデックスは生成ページの横の `search-index.json` です。初回検索で遅延取得します。dev ではメモリから出し、ページ変更で作り直します。

`provider: "hosted"` にするまでローカル BM25 のままです。バージョン付きツリーでは `{prefix}/search-index.json` を読みます。

## 関連

- [英語版ガイド](/built-in/search.md)
- [ドキュメントのバージョン管理](./versioning.md)
