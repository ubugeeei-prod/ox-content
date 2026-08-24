---
title: 品質チェック
description: フェンス lint、TypeScript 型チェック、docs テスト、HTML サニタイズ。
---

# 品質チェック

サンプルが腐るとドキュメントも腐ります。これらの検査は Markdown 変換中に走り、壊れたスニペットは読者に届く前にビルドを落とします。

| オプション           | 既定    | 検査                                      |
| -------------------- | ------- | ----------------------------------------- |
| `codeBlockLint`      | `false` | 言語欠落、末尾空白                        |
| `codeBlockTypecheck` | `false` | TypeScript フェンスを `tsgo` でコンパイル |
| `docsTests`          | `false` | 実行可能なフェンスを Vitest で通す        |
| `sanitize`           | `false` | 描画 HTML を許可リストで検査              |

変換中にドキュメントのコードは実行しません。`docsTests` だけが実行しますが、CI から呼ぶ別の Vitest ハーネスの中です。

```ts
oxContent({
  codeBlockLint: {
    requireLanguage: true,
    trailingSpaces: true,
    mode: "error",
  },
  codeBlockTypecheck: true,
  docsTests: true,
  sanitize: true,
});
```

`mode: "warn"` はログ、`"error"` は変換失敗です。

## 関連

- [英語版ガイド](/built-in/quality-checks.md)
- [コードブロック](./code-blocks.md)
