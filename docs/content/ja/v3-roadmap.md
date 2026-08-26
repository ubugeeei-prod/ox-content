---
title: Ox Content 3.0 ロードマップ
description: テーマパッケージ、完全な MDX、Code Play、オプトイン組み込み機能、tree-sitter ハイライトのリリーストラッカーです。
---

# Ox Content 3.0 ロードマップ

トラッキング issue: [#699](https://github.com/ubugeeei-prod/ox-content/issues/699)。

3.0 は、実験的な面を卒業し、ハイライトの破壊的変更を受け入れるリリースです。新しい Markdown と追加のクロムは **オプトイン** のままです。
作業は、まず落ちるテストを置いた小さな conventional PR で入ります。

| 柱                       | Issue                                                          | メモ                                                                                                        |
| ------------------------ | -------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| 安定したテーマパッケージ | [#700](https://github.com/ubugeeei-prod/ox-content/issues/700) | スキンとカラースキームが公式の契約になります                                                                |
| 完全な MDX 対応          | [#701](https://github.com/ubugeeei-prod/ox-content/issues/701) | JSX、import、JSX 内 Markdown。island は既定で JS なしのまま                                                 |
| Code Play                | [#648](https://github.com/ubugeeei-prod/ox-content/issues/648) | 連動する柱。[Code Play ロードマップ](/code-play-roadmap.md) を見てください                                  |
| 組み込み docs サイト機能 | [#650](https://github.com/ubugeeei-prod/ox-content/issues/650) | 執筆、サイト出力、テーマクロム — 既定は OFF                                                                 |
| Tree-sitter ハイライト   | [#702](https://github.com/ubugeeei-prod/ox-content/issues/702) | [#710](https://github.com/ubugeeei-prod/ox-content/pull/710) で投入済み。`highlight: true` はネイティブのみ |

ほかで追跡しており、ここには重複しません。

- i18n / MF2 コア: [#451](https://github.com/ubugeeei-prod/ox-content/issues/451)

## 破壊的変更

- `highlightTheme` と `highlightLangs` はなくなります。ハイライターはひとつです。
- tree-sitter 文法がない言語はプレーンのままです。
- テーマパッケージの peer 範囲は 3.x へ移ります。
- 組み込み機能と Code Play は、メジャーバージョンが変わっただけでは **有効になりません**。それぞれ明示的なインストールかオプションが必要です。
- `redirects.netlify` は削除されました。`provider: "netlify"` にするか、CI に Netlify / Cloudflare を検出させるなら `provider` を省略してください。

シンタックストークンの CSS は `<pre class="ox-highlight css-variables">` 上の `--octc-syntax-*` カスタムプロパティです。カラーパッケージは対応する `syntax-*` トークンを定義します。

## 組み込み機能

機能一覧は [ドキュメントサイト機能のロードマップ](/docs-site-feature-roadmap.md) と
[#650](https://github.com/ubugeeei-prod/ox-content/issues/650) にあります。
