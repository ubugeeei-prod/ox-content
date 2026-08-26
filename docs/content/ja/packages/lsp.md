---
title: チェッカーと言語サーバー
description: ox-content-lsp と CLI チェッカーが共有する診断、対応機能、既知のギャップ。
---

# チェッカーと言語サーバー

`ox-content-lsp` と CLI チェッカーは、Markdown / MDC / ローカルリンクについて
同じ診断ケースを共有します。エディタは
`textDocument/publishDiagnostics` で受け取り、CI は同じコード・範囲・メッセージを
チェッカーバイナリから実行できます。

## 共有する診断ケース

| Source               | CLI                             | コード                                                                                  |
| -------------------- | ------------------------------- | --------------------------------------------------------------------------------------- |
| `ox-content-mdc`     | `ox-content-mdc-check`          | `mdc-unquoted-prop`, `mdc-mismatched-tag`, `mdc-orphan-close`, `mdc-unclosed-tag`       |
| `ox-content-link`    | `ox-content-link-check`         | `link-missing-file`, `link-missing-anchor`, `link-cross-file-anchor`, `link-unresolved` |
| `ox-content`         | なし（frontmatter は LSP のみ） | `frontmatter-unknown`, `frontmatter-type`, `frontmatter-enum`, `frontmatter-required`   |
| `ox-content-spacing` | なし（spacing は LSP のみ）     | `space-between-half-and-full-width`, `require-space-between-half-and-full-width`        |
| `textlint`           | 設定した `textlint` コマンド    | sidecar の rule id                                                                      |

MDC CLI は YAML frontmatter をスキップし、言語サーバーと同じ行にタグ診断を出します。
リンク検査はどちらの面でも文書全体を対象にします。

```bash
ox-content-mdc-check --format json docs/page.mdc
ox-content-link-check --format json docs/page.md
cargo run -p ox_content_lsp --bin ox-content-lsp
```

## 言語サーバーの振る舞い

- 増分の `textDocument/didChange` は編集範囲だけを適用し、変わったスライスだけを
  再計算します。本文だけの編集では frontmatter 診断を再利用します。
- 各 publish は文書バージョンを載せます。後続の編集は実行中のジョブをキャンセルし、
  古い結果や重複は公開しません。
- textlint はオプトインで、保存時だけ走ります。Markdown / MDC / リンク診断は
  変更のたびに更新されます。

## 既知のギャップ

- 外部 HTTP リンクは取得しません。リンクチェッカーはオフライン専用です。
- ソース側のファイル間アンカー（`./other.md#section`）は
  `link-cross-file-anchor` 警告になります。生成サイト検査（`--site-dir`）が
  ビルド後に検証します。
- 参照リンク（`[ok][ref]`）は、まだパーサーが展開しません。
- frontmatter スキーマと半角/全角スペース検査には CLI がありません。
- コードブロック lint、`tsgo` 型チェック、docs-as-tests は Vite transform 側です。
  LSP 診断にはなりません。
- `.mdx` のパースエラーは報告しますが、式の型チェックはしません。
- i18n 診断は JavaScript/TypeScript ソースにだけ出し、Markdown のガターには混ぜません。

[アーキテクチャ](../architecture.md) も参照してください。
英語の [Editor Extension Roadmap](/editor-extension-roadmap.md) もあります。
