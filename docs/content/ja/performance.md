---
title: 性能
description: CommonMark 適合と、Markdown パイプラインの速さ。
---

# 性能

Ox Content のコアは Rust です。パース、描画、検索インデックス、サイト生成の重い部分をネイティブで処理します。

CommonMark 0.31.2 のコアプロファイルは CI で全例を確認します。見出しに slug `id` が付くため、マークアップは仕様 HTML とバイト一致ではありません。適合の数値とプロファイル比較は [英語の Performance](/performance.md) を見てください。

プロファイリング手順は [Profiling Mode (英語)](/profiling.md) です。アーキテクチャの地図は [アーキテクチャ](./architecture.md) です。
