---
title: 性能
description: ベンチマーク結果、バンドルサイズ、出荷物に効く最適化。
---

# 性能

このページはベンチマーク結果、バンドルサイズ検査、出荷物に効く最適化を追います。アーキテクチャとパッケージのページは境界と API を述べ、測った性能はここに置きます。

パーサやレンダラの変更を開発しているときの、割り当てと span 単位の調査は [プロファイリング](./profiling.md) です。プロファイリングは「仕事はどこで起きているか」に答え、ベンチマークは「この負荷はどれだけ速いか」に答えます。

## 何を測るか

Ox Content には性能の面が 4 つあり、どれも pull request で報告します。

- Markdown のパースと描画の実行時スループット。
- フィクスチャの本番ビルド時間。
- 生成サイトの静的出力の重さ。gzip サイズと、描画 HTML の gzip を含みます。
- `index.html` と、ローカルのクリティカルパスアセットの初期リクエスト数。

実行時は CLI、開発サーバ、エディタ連携、バッチビルドで効きます。出力の重さはドキュメントサイトで効きます。生成 HTML、CSS、JS が、遷移のたびに利用者が取るものだからです。絶対上限は
[`benchmarks/perf-budgets.json`](https://github.com/ubugeeei-prod/ox-content/blob/main/benchmarks/perf-budgets.json)
にあります。base / head の相対差は、これまでどおり PR Benchmark コメントです。

### 対象外

予算ファイルは、次を意図してゲートしません。

- 開発者のノート PC でのホスト絶対 ops/sec。公開している相対順位を使い、下限は Blacksmith ランナー級向けです。
- 失敗で落とす天井としての競合アプリサイズ（VitePress、Astro）。
- ドキュメントサイト全体の出力（`docs/dist`）、OG 画像バイナリ、取得したフォント。
- `vp run build:npm` の壁時計時間。このハーネスが測るビルド時間はフィクスチャの `buildMs` です。
- 検索、テーマ、埋め込み、Code Play、MDX island の単独ペイロード。既定フィクスチャはそれらをオンにしないので、既定シェルの数字の外に置きます。

## 実行時スナップショット

Ox Content は文書生成器でもあり、高性能 Markdown ツールキットでもあります。下の数字は Markdown エンジン側に寄せています。

速さは公平な比較の半分にすぎません。Markdown エンジンは CommonMark の実装範囲が違い、仕様カバーをスループットと意図して交換するものもあります。だから各行は速さの横に、測った CommonMark 適合率を持ちます。仕様挙動を飛ばして速いエンジンは、単に上位に並ぶのではなく、そう見えるようにします。Ox Content の点数と測り方は [CommonMark 適合](#commonmark-適合) を見てください。

<!-- benchmark:tables:start -->

_2026-08-17 生成のベンチマーク掃引（7 回の中央値）。数字はホスト機に追従します。エンジン間の相対順位が安定した信号です。`scripts/render-benchmark-tables.mjs` で再生成します。_

_環境: runner `blacksmith-32vcpu-ubuntu-2404`、Node `v24.19.0`、Bun `1.3.14`、CPU `Intel(R) Xeon(R) Processor`、論理コア 32。_

_CommonMark 列: エンジンが正しく描画する CommonMark 0.31.2 仕様例 652 件の割合。`benchmarks/commonmark-conformance/run.mjs` で測ります。各エンジンは、出すいちばん仕様に忠実な設定で走り、比較の両側は適合スイートの HTML 正規化器を通るので、マークアップの綴りではなく挙動で順位が付きます。_

### パースのみ (48.7 KB)

| Library                               | ops/sec | avg time |  throughput | CommonMark |
| ------------------------------------- | ------: | -------: | ----------: | ---------: |
| `ox-content (native)`                 |   7,739 |  0.13 ms | 368.27 MB/s |     100.0% |
| `pulldown-cmark`                      |   4,941 |  0.20 ms | 235.14 MB/s |     100.0% |
| `xai-grok-markdown-core (Grok Build)` |   4,233 |  0.24 ms | 201.44 MB/s |     100.0% |
| `@ox-content/napi`                    |   3,479 |  0.29 ms | 165.57 MB/s |      99.5% |
| `satteri`                             |   1,521 |  0.66 ms |  72.37 MB/s |      98.9% |
| `md4x (napi)`                         |   1,157 |  0.86 ms |  55.08 MB/s |      99.5% |
| `md4x (wasm)`                         |     967 |  1.03 ms |  46.02 MB/s |      99.5% |
| `md4w (md4c)`                         |     931 |  1.07 ms |  44.30 MB/s |      91.7% |
| `markdown-it-ts`                      |     876 |  1.14 ms |  41.68 MB/s |     100.0% |
| `@tanstack/markdown`                  |     684 |  1.46 ms |  32.53 MB/s |      47.4% |
| `marked`                              |     420 |  2.38 ms |  19.96 MB/s |      93.4% |
| `markdown-it`                         |     285 |  3.51 ms |  13.56 MB/s |     100.0% |
| `@mizchi/markdown`                    |      59 | 16.81 ms |   2.83 MB/s |      45.9% |
| `remark`                              |      33 | 30.28 ms |   1.57 MB/s |      99.8% |

### パース + 描画 (48.7 KB)

| Library                      | ops/sec | avg time |  throughput | CommonMark |
| ---------------------------- | ------: | -------: | ----------: | ---------: |
| `ox-content (native)`        |   5,879 |  0.17 ms | 279.78 MB/s |     100.0% |
| `@ox-content/napi`           |   5,424 |  0.18 ms | 258.13 MB/s |      99.5% |
| `pulldown-cmark + push_html` |   4,733 |  0.21 ms | 225.24 MB/s |     100.0% |
| `md4x (napi)`                |   3,122 |  0.32 ms | 148.56 MB/s |      99.5% |
| `Bun.markdown.html`          |   2,394 |  0.42 ms | 113.93 MB/s |     100.0% |
| `md4x (wasm)`                |   2,089 |  0.48 ms |  99.43 MB/s |      99.5% |
| `md4w (md4c)`                |   1,856 |  0.54 ms |  88.31 MB/s |      91.7% |
| `satteri`                    |   1,200 |  0.83 ms |  57.10 MB/s |      98.9% |
| `markdown-it-ts`             |     767 |  1.30 ms |  36.48 MB/s |     100.0% |
| `@mizchi/markdown`           |     660 |  1.51 ms |  31.42 MB/s |      45.9% |
| `@tanstack/markdown`         |     450 |  2.22 ms |  21.43 MB/s |      47.4% |
| `marked`                     |     381 |  2.62 ms |  18.13 MB/s |      93.4% |
| `markdown-it`                |     259 |  3.86 ms |  12.34 MB/s |     100.0% |
| `micromark`                  |      35 | 28.83 ms |   1.65 MB/s |     100.0% |
| `remark`                     |      28 | 35.95 ms |   1.32 MB/s |      99.8% |

### パースのみ (~1 MB)

| Library                               | ops/sec |   avg time |  throughput | CommonMark |
| ------------------------------------- | ------: | ---------: | ----------: | ---------: |
| `ox-content (native)`                 |     362 |    2.76 ms | 370.17 MB/s |     100.0% |
| `pulldown-cmark`                      |     234 |    4.28 ms | 239.19 MB/s |     100.0% |
| `xai-grok-markdown-core (Grok Build)` |     201 |    4.98 ms | 205.39 MB/s |     100.0% |
| `@ox-content/napi`                    |     199 |    5.03 ms | 203.45 MB/s |      99.5% |
| `satteri`                             |      69 |   14.54 ms |  70.36 MB/s |      98.9% |
| `md4w (md4c)`                         |      44 |   22.61 ms |  45.26 MB/s |      91.7% |
| `md4x (napi)`                         |      42 |   24.05 ms |  42.54 MB/s |      99.5% |
| `md4x (wasm)`                         |      39 |   25.73 ms |  39.76 MB/s |      99.5% |
| `@tanstack/markdown`                  |      26 |   39.15 ms |  26.13 MB/s |      47.4% |
| `markdown-it-ts`                      |      23 |   43.74 ms |  23.39 MB/s |     100.0% |
| `marked`                              |      17 |   60.21 ms |  16.99 MB/s |      93.4% |
| `markdown-it`                         |      10 |   95.62 ms |  10.70 MB/s |     100.0% |
| `@mizchi/markdown`                    |       2 |  561.04 ms |   1.82 MB/s |      45.9% |
| `remark`                              |       1 | 1533.48 ms |   0.67 MB/s |      99.8% |

### パース + 描画 (~1 MB)

| Library                      | ops/sec |   avg time |  throughput | CommonMark |
| ---------------------------- | ------: | ---------: | ----------: | ---------: |
| `ox-content (native)`        |     261 |    3.83 ms | 267.26 MB/s |     100.0% |
| `@ox-content/napi`           |     216 |    4.62 ms | 221.33 MB/s |      99.5% |
| `pulldown-cmark + push_html` |     216 |    4.63 ms | 221.16 MB/s |     100.0% |
| `md4x (napi)`                |     133 |    7.51 ms | 136.22 MB/s |      99.5% |
| `Bun.markdown.html`          |     115 |    8.72 ms | 117.27 MB/s |     100.0% |
| `md4w (md4c)`                |     109 |    9.17 ms | 111.55 MB/s |      91.7% |
| `md4x (wasm)`                |      92 |   10.91 ms |  93.79 MB/s |      99.5% |
| `satteri`                    |      64 |   15.56 ms |  65.77 MB/s |      98.9% |
| `@mizchi/markdown`           |      23 |   43.92 ms |  23.30 MB/s |      45.9% |
| `markdown-it-ts`             |      19 |   52.18 ms |  19.61 MB/s |     100.0% |
| `@tanstack/markdown`         |      18 |   55.33 ms |  18.49 MB/s |      47.4% |
| `marked`                     |      15 |   68.43 ms |  14.95 MB/s |      93.4% |
| `markdown-it`                |      10 |  103.65 ms |   9.87 MB/s |     100.0% |
| `micromark`                  |       1 |  746.23 ms |   1.37 MB/s |     100.0% |
| `remark`                     |       1 | 1712.02 ms |   0.60 MB/s |      99.8% |

<!-- benchmark:tables:end -->

上の表は、[Benchmark docs workflow](https://github.com/ubugeeei-prod/ox-content/blob/main/.github/workflows/benchmark-docs.yml) がきれいな Blacksmith 32 vCPU CI 環境から再生成します。ローカルで更新するには `OX_CONTENT_BENCHMARK_RUNS=7 vp run bench:docs` です。絶対 ops/sec はホストに追従します（より速いハードウェアでの以前の掃引はより高い数字を出します）。エンジン間の相対順位が安定した信号です。

48.7 KB 文書では、境界なしのネイティブ行が、次に速い他エンジン（`pulldown-cmark`）に対してパースのみで約 1.6 倍、パース + 描画で約 1.2 倍です。~1 MB ではその差は約 1.5 倍と約 1.2 倍です。パース + 描画では 2 行目は第三者エンジンではなく `@ox-content/napi` なので、そこで上位 2 行を分けるのはエンジンではなく N-API 境界です。JavaScript 向けの `@ox-content/napi` 行は、パースのみで 2 つの TypeScript レンダラより 4.0–5.1 倍、パース + 描画で 7.1–12.1 倍速いです。~1 MB ではその N-API の差はそれぞれ 7.7–8.7 倍と 11.4–12.0 倍に広がり、ネイティブパイプラインは 267–370 MB/s を維持します。その大きさでは、増分 CST パーサ（`@mizchi/markdown`、一括パースではなくリアルタイム編集向け）はパースのみで約 2 ops/sec に落ち、パース + 描画では約 23 ops/sec に戻ります。一方 `unified` / `remark` パイプラインは両表で約 1 op/sec のまま、`micromark` はパース + 描画で約 1 op/sec です。

実行時掃引は上の表より広いです。ハーネスは小さい / 中くらいの Markdown 入力、N-API パッケージ向けの非同期パース + 描画ターゲット（PR 検査が JavaScript 境界のオーバーヘッド回帰を拾えるように）、そしてハーネスを Bun で走らせたときの任意の `Bun.markdown` 比較も走ります。

TypeScript レンダラ比較は、各パッケージの公開既定 API を使います。`@tanstack/markdown` はパースのみで `parseMarkdown`、パース + 描画で `renderHtml` です。`markdown-it-ts` は初期化済みインスタンスを再利用し、`parse` と `render` を測ります。既存の `markdown-it` 設定と同じです。パーサ構築、モジュール読み込み、ベンチマークウォームアップは計時ループの外です。各パース + 描画操作は同じ Markdown 文字列から始まります。

## CommonMark 適合

Ox Content は完全な CommonMark 適合を狙います。エンジンは、ベンチマーク表を更新するときだけでなく、毎回の CI で同梱の [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/) 仕様に照らします。

- **コアプロファイル: 652 / 652 例。** `cargo test -p ox_content_renderer --test spec_commonmark` は、通っていた例が退行したときも、記録済み失敗が通り始めたときも失敗するので、ベースラインが静かにずれません。
- **GFM プロファイル: 649 / 652 例。** 違いは仕様例 608、611、612 です。GFM autolink 拡張が、素の CommonMark ではテキストのままにする裸 URL とメールを意図してリンク化します。`crates/ox_content_renderer/tests/spec_fixtures/commonmark-known-failures.txt` に列挙しています。
- **GFM 拡張: すべての例。** GitHub Flavored Markdown 0.29-gfm 仕様の表、タスクリスト、取り消し線、autolink、禁止生 HTML の各節。`spec_gfm.rs` が駆動します。

CommonMark を超える拡張 — GFM 表、タスクリスト、取り消し線、脚注、組み込み埋め込み — はオプトインではなくオプトアウトなので、どれも使わない文書は、下の正規化規則の下で仕様に適合します（ox-content が見出しに slug の `id` 属性を付けるので、HTML は仕様とバイト一致ではありません）。各トグルは [Markdown の土台](./built-in/markdown.md) にあります。

### CommonMark 列の読み方

各エンジンの横の率は、主張ではなく測定です。`benchmarks/commonmark-conformance/run.mjs` は表のすべてのエンジンで仕様例 652 件を描画し、出力を仕様と比較します。

比較を公平にする選択が 2 つあり、数字を読むときどちらも効きます。

- **各エンジンは、出すいちばん仕様に忠実な設定で走る**のであり、ベンチマーク既定ではありません。`markdown-it` と `markdown-it-ts` は `commonmark` プリセット、`micromark` と `remark-html` は生 HTML をエスケープせず通すよう指示し、`md4w` は `parseFlags: 0` です。GFM 拡張をオンにする既定プリセットでエンジンを判定すると、エンジンではなくプリセットを測ることになります。そのようなモードを出さないエンジン — `marked`、`md4x`、`@tanstack/markdown`、`@mizchi/markdown` — は出荷どおりに点数を付けます。
- **両側は適合スイートの HTML 正規化器を通る**ので、文書の描画を変えない差（実体の綴り、属性順、`<br />` 対 `<br>`、ブロックタグ間の空白）は適合の隙間として数えません。なければ表はマークアップの綴りで順位を付けます。ox-content は見出しに slug の `id` を付けるだけで、バイト一致比較では 82.5% になります。

比較は対称です。ox-content とは無関係な独立 Rust 実装 `pulldown-cmark` も 100% なので、正規化器は特定エンジン向けに調整していません。

2 行は同じエンジンの異なる層です。`ox-content (native)` はコアプロファイルで 100%、`@ox-content/napi` は 99.5% です。既定が裸 URL の autolink 組み込みをオンにし、例 602、608、611 をリンク化するからです。切るときは `autolinkUrls: false` です。

結果を再生成するには次です。

```bash
node benchmarks/commonmark-conformance/run.mjs --json benchmarks/commonmark-conformance/results.json
```

### CJK 強調

CommonMark の強調規則は、CJK 句読点の直後に置いた `**` を認識しないので、`A**強調。**B` は太字ではなくリテラルになります。これは仕様適合のエンジンすべてに効きます。実装の隙間ではなく、仕様の左右隣接区切り規則の性質です。CJK **文字** の隣に座るだけの強調 — `これは**重要**です。` — は仕様が許しており、どこでも動きます。

Ox Content はこれ向けのオプトイン逸脱を載せます。`cjkEmphasis` は、区切りランが開閉してよいかを決めるとき、東アジア句読点を普通の文字として分類するので、それらのランが対になります。半角 ASCII 句読点はそのままなので、ラテン文書のパースは同じです。既定はオフで、それが上の 652/652 を出荷既定の真実に保ちます。[CJK Emphasis](/examples/cjk-emphasis.md) を見てください。

## バンドルサイズ

バンドルサイズベンチマークは代表的な docs アプリをビルドし、生成した本番出力を測ります。報告するのは次です。

- 生成出力ディレクトリ全体のバイト。
- JS、CSS、HTML、JSON アセットの gzip バイト。
- 出力ディレクトリのファイル数。
- `index.html` と、HTML または CSS から参照されるローカルアセットの、推定初期リクエスト数。

最新の Blacksmith フィクスチャ掃引は 2026-08-26（`blacksmith-32vcpu-ubuntu-2404`、Node `v26.7.0`）です。PR Benchmark コメント（#1001、#1005 など）から取りました。

| App                   |  Gzipped | HTML gzip | Requests | Files |
| --------------------- | -------: | --------: | -------: | ----: |
| `ox-content (bare)`   |   5.8 KB |    2.9 KB |        1 |     5 |
| `ox-content`          |  31.4 KB |    9.8 KB |        5 |    10 |
| `ox-content + Vue`    |  53.9 KB |    9.8 KB |        5 |    10 |
| `VitePress (bare)`    |  47.3 KB |    8.4 KB |        6 |    14 |
| `Astro + Vue`         |  33.1 KB |    5.4 KB |        3 |     7 |
| `VitePress (default)` | 717.2 KB |   14.2 KB |       21 |    29 |

`ox-content (bare)` は JS なしのベースラインです。`ox-content` は組み込み docs シェルを含みます。`ox-content + Vue` はフレームワーク island 対応を足します。VitePress 行は同じベンチマーク本文を使うので、比較は執筆内容ではなく生成出力の形に寄ります。

バンドルサイズ比較は、意図してリクエスト数の横に出します。gzip が小さくても、ブロックするリクエストが多すぎれば必ずしも良くありません。繰り返しバイトを各生成ページから除き、遷移をまたいでキャッシュできるなら、より大きな共有チャンクのほうが望ましいことがあります。

## チャンク最適化

SSG パイプラインはまず完全な HTML ページを描画し、すべてのページが分かってから共有アセットを取り出します。TypeScript プラグインは Rust バックの `externalizeSsgAssets` を呼び、各生成ページを書き換え、ハッシュ付きアセットを `assets/` 以下に書きます。

最適化の規則は控えめです。

- 同じ CSS / JS 内容は内容で重複排除し、ファイル名に内容ハッシュを付けて一度だけ出します。
- base や footer スタイルのようなコア CSS 区画は、共有の `ox-content-core-*.css` としてリンクします。
- テーマ CSS は小さいとき、または相対 `url(...)` 参照を含むときはインラインのままです。パス解決が変わらないようにするためです。
- 検索ペイロードコードは、生成スクリプトに検索プレースホルダがあるとき、本体の起動スクリプトから別の `ox-content-search-*.js` チャンクに分けます。
- 生成スクリプトは `defer` で出し、公開アセットパスは設定したサイト `base` を尊重します。

目標は、すべての機能を別ファイルに分けることではありません。docs シェルは十分小さいので、過剰なチャンクは起動仕事を増やせます。いまの方針は、繰り返しバイト向けの安定した共有チャンクを好み、パス解決リスクや余分なリクエストを生む移動は避けます。

## PR 回帰ゲート

Pull request は、`blacksmith-32vcpu-ubuntu-2404` 上で base コミットと head コミットの両方に対してベンチマークワークフローを走らせます。ワークフローは実行時、競争スナップショット、環境、バンドルサイズ、予算の各節を持つ報告を 1 つ投稿します。

実行時行は、`@ox-content/napi` と `@ox-content/napi (async)` の大きなベンチマークターゲットを比較します。+/-5% 以内はノイズとし、head のスループットが base より 10% 超遅いと検査は失敗します。

競争スナップショットはゲートではありません。同じ大きな入力コーパスで、head コミットの対象パッケージを次に速い比較パッケージと並べます。

バンドル行は、成功した各ベンチマークアプリの gzip 出力を比較します。gzip サイズが 5% 超増えると検査は失敗します。head の測定が
[`benchmarks/perf-budgets.json`](https://github.com/ubugeeei-prod/ox-content/blob/main/benchmarks/perf-budgets.json)
を超えても失敗します。どちらの失敗も、メンテナは `benchmark-regression-accepted` PR ラベルで意図して受け入れられます。

## 領域の監査

主要な面ごとに、測ったベースラインと、目標または明示の no-op があります。数字は上の 2026-08-26 Blacksmith 報告であり、手元のノート PC の走行ではありません。

| 領域                                     | ベースライン                                                                   | 目標または no-op                                                                                                                                         |
| ---------------------------------------- | ------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| フィクスチャのビルド時間                 | 既定シェル約 370–430 ms。冷えた走行は約 750 ms                                 | 天井 5 秒。より狭いゲートはしない。#1003 は出力サイズ不変のまま同じフィクスチャが +45% 振れた。                                                          |
| 実行時パース / 描画                      | large `@ox-content/napi` パース約 5.3k–6.1k ops/sec、パース + 描画約 6.5k–7.3k | 下限 2500 / 4000 ops/sec。公開表のパース + 描画で `pulldown-cmark` より速いままにする。                                                                  |
| バンドル gzip                            | bare 5.8 KB、既定 31.4 KB、+Vue 53.9 KB                                        | 既定を 48 KB 未満に保つ。共有チャンクはすでに VitePress 既定（717.2 KB）より小さい。既定シェルの追加分割は測定済み no-op: リクエスト 5 対 VitePress 21。 |
| 描画 HTML gzip                           | bare 2.9 KB、既定 / Vue 9.8 KB                                                 | 既定シェルの天井 16 KB。                                                                                                                                 |
| 初期リクエスト                           | bare 1、既定 / Vue 5                                                           | 天井 8。既定フィクスチャにブロックするリクエストを足さない。                                                                                             |
| 公開パッケージの重さ                     | `measure.mjs` は測らない                                                       | no-op: CI がゲートするのは生成サイト出力であり、npm tarball の重さではない。                                                                             |
| 検索 / 埋め込み / Code Play / MDX island | フィクスチャアプリはこれらをオンにしない                                       | no-op: フィクスチャが行使するまで、機能ペイロードを 31.4 KB の既定シェル数字の外に置く。                                                                 |

## 再現

リポジトリルートから JavaScript ベンチマークハーネスを走らせます。

```bash
node benchmarks/bundle-size/parse-benchmark.mjs
```

ベンチマークは既定で `@tanstack/markdown`、`markdown-it-ts`、`md4w (md4c)`、`md4x (napi)` を含み、`bun` があれば `Bun.markdown.html` を自動で足します。

リポジトリルートからバンドルサイズベンチマークを走らせます。

```bash
node benchmarks/bundle-size/measure.mjs
```

依存を入れたあとの速い再実行は次です。

```bash
node benchmarks/bundle-size/measure.mjs --skip-install
```

予算検査向けに JSON を書き、任意で専用のビルド時間掃引もリポジトリルートから走らせます。

```bash
node benchmarks/bundle-size/measure.mjs --json /tmp/bundle.json
node benchmarks/bundle-size/build-time-benchmark.mjs --json /tmp/build.json
node benchmarks/bundle-size/check-budgets.mjs --bundle /tmp/bundle.json --build /tmp/build.json
```

Rust 側のパーサベンチマークは次です。

```bash
cargo bench -p ox_content_parser
```

実世界の Markdown コーパスベンチマークでは、まず任意コーパスを用意します。

```bash
node scripts/fetch-bench-corpus.mjs
cargo bench -p ox_content_parser --bench corpus
```

N-API 転送形式のマイクロベンチマークは [@ox-content/napi](./packages/napi.md#mdast-転送マイクロベンチマーク) を見てください。
