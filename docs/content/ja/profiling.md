---
title: プロファイリング
description: Markdown エンジンの割り当てと時間を追う、組み込みプロファイラ。
---

# プロファイリング

Ox Content は、Markdown エンジンの割り当てと時間を追う組み込みプロファイラ — `ox_content_profiler` — を載せます。壁時計ベンチマークではなく、「このコードは実際にどれだけ仕事をしているか」向けに意図して調整しています。壁時計は `criterion` と `benchmarks/` の JS ベンチマークハーネスが担当します。

プロファイラは、次のような問いに答えたいときに使います。

- このファイルのパースは何回割り当て、どの span からか。
- コーパス内の文書で、どのブロックレベルパーサ関数が時間を支配するか。
- この変更は本当に割り当てを減らしたか、それとも場所を入れ替えただけか。

## 何を測るか

独立した層が 3 つあり、どれも 1 つの CLI から出ます。

1. **カウントするグローバルアロケータ**（`ox_content_profiler::CountingAllocator`）は `std::alloc::System` を包み、すべての割り当て、解放、バイトカウンタ、ピーク生存バイト、2 の累乗サイズクラスヒストグラムをアトミックに記録します。`ox-content-profile` バイナリで `#[global_allocator]` として入るので、計測窓のあいだプロセスがすること **すべて** が数に入ります。

2. **階層タイミング span**（`ox_content_profiler::scope`）は、self / inclusive 時間の集約と、span ごとの割り当て差分を持つスレッドローカル span スタックを保ちます。パーサ、レンダラ、docs 生成 crate はそれぞれ `profile` Cargo feature を持ち、熱い入口に本物の `profile_span!` ガードを差し込みます。Markdown エンジンでは `parse_block`、`parse_html_block`、`visit_heading`、`write_escaped`、JS/TS docs 生成器では `docs::oxc_parse`、`docs::parse_jsdoc`、`docs::visit_ast`、`docs::render_entry_page` などです。feature がオフ（既定）のとき、`profile_span!` はオプティマイザが落とすゼロサイズ束縛に展開します。

3. **報告整形**（`ox_content_profiler::Report`）は反復ごとの記録を、パーセンタイル時間、割り当て要約、span 内訳、ヒストグラムに畳みます。等幅表、または CI 向けの 1 行 JSON として描画します。

## CLI クイックスタート

CLI は `crates/ox_content_profile_cli` にあり、`ox_content_parser` と `ox_content_renderer` の `profile` feature をビルドします。

```bash
# 埋め込みコーパスに対するパイプライン（パース + 描画）
cargo run --release -p ox_content_profile_cli -- pipeline

# 特定ファイルを、GFM オン、計測 200 回でプロファイル
cargo run --release -p ox_content_profile_cli -- \
    pipeline --gfm --iters 200 --warmup 20 \
    docs/content/api/types.md

# パースのみ — パーサ仕事を切り出すときに便利
cargo run --release -p ox_content_profile_cli -- parse path/to/file.md

# 描画のみ — 入力は計測ループの外で一度パースする
cargo run --release -p ox_content_profile_cli -- render path/to/file.md

# CI で差分を取るための機械可読出力
cargo run --release -p ox_content_profile_cli -- pipeline --json path/to/file.md

# すべて、どこでも: ノードごとのインラインハンドラ、行ごとの走査、
# エスケープパス。正直な読み方は下の「詳細トレース」。
cargo run --release -p ox_content_profile_cli -- pipeline --gfm --detail path/to/file.md
```

必ず `--release` でビルドしてください。マクロ展開した `profile_span!` ガードは安いですが、デバッグビルドでは本物の仕事を支配します。

### 詳細トレース（`--detail`）

span は 2 段です。既定段は、本体が十分仕事をする相レベルの入口（`parse_block`、`parse_inline`、`visit_heading`、…）を計装するので、ガードコストはノイズに消えます。第 2 段 — `profile_span_detail!` — はノードごと・行ごとの熱い経路に座ります。インライン特殊バイト走査、すべての強調区切りラン、各 HTML エスケープパス、各表セルです。それらのガードは `--detail` を渡したときだけ記録するので、既定実行の相レベル数字は歪みません。

詳細数字は正直に読む必要があります。ガードあたり数十ナノ秒だと、何百万回も当たる span は計測自体を測っている部分があります。CLI は起動時にヒットあたりのガードコストを校正し、表は `~ovh` 列（`hits × ガードコスト`）と、測ったコストの脚注として出します。`self` から `~ovh` を頭の中で引いてください。self 時間のほとんどが `~ovh` の行は、どれだけ上位でも安いです。親 span の `self` / `inclusive` も子のガードコストを吸収するので、同類同士で比べてください。詳細実行対詳細実行、既定実行対既定実行です。

### JS/TS docs 生成器のプロファイリング

`docs-*` サブコマンドは、単一 Markdown ではなくソース **ディレクトリ** に対して `ox_content_docs`（JavaScript 向けの "cargo doc"）をプロファイルします。本番パイプラインを再現します。OXC パース → JSDoc パース → AST 訪問 → 正規化 → TypeDoc / 純 Markdown 描画です。

```bash
# 取り出しのみ: OXC パース + JSDoc パース + AST 訪問 + 正規化。
# ディレクトリ下のすべての .ts/.tsx/.mts/.cts に対して。
cargo run --release -p ox_content_profile_cli -- docs-extract path/to/src

# 描画のみ: 取り出しは計測ループの外に上げ、時間は Markdown 描画経路だけを反映する。
cargo run --release -p ox_content_profile_cli -- docs-render path/to/src

# 全パイプライン: 取り出し + 正規化 + Markdown 描画。
cargo run --release -p ox_content_profile_cli -- docs-pipeline path/to/src --json
```

報告のスループットは取り込んだソースの総バイトから計算し、span 行には `docs::` が付くので、Markdown エンジンの span と区別しやすいです。

## 報告の読み方

```text
 Timing
   min   15.50 µs
   p50   35.83 µs
   p95   38.38 µs
   ...
   throughput     680.30 MB/s

 Allocations (per iteration)
   count               46.0
   bytes           57.01 KB
   peak (max)      46.25 KB
   largest         90.00 KB

 Spans (sorted by total inclusive time)
   name                          hits      self  inclusive  share   allocs  bytes
   parser::parse_html_block      7600   3.56 ms    3.56 ms  55.4%      0     0 B
   ...
```

- **時間パーセンタイル** は、最初の `--warmup` を捨てたあとの `--iters` 反復から計算します。コールドキャッシュ効果を除くためです。
- **反復あたりの割り当て** は、それらの反復の平均回数 + バイトと、どの単一反復も開始ベースラインより上で達した最大ピーク生存バイトです。
- **span** は計測したすべての反復で集約します。`self` は inclusive から子 span の inclusive 時間を引いたものなので、関数が自分の本体で過ごす時間です。`share` は、すべての span の総 self 時間に対するその span の self 時間の割合で、「ここで使った CPU の割合」の速い代理です。
- **サイズクラスヒストグラム** は、最後の反復の割り当てを 2 の累乗サイズで桶分けします。小さく短命な割り当てのスパイクを見つけるのに使えます。

## profile feature の構造

計装フックは crate ごとの Cargo feature でゲートします。パーサ / レンダラの別の消費者をプロファイルするには次です。

```toml
[dependencies]
ox_content_parser    = { workspace = true, features = ["profile"] }
ox_content_renderer  = { workspace = true, features = ["profile"] }
ox_content_profiler  = { workspace = true }
```

バイナリ内で、負荷の前にグローバルアロケータと両層を入れ、それから報告を排出します。

```rust
use ox_content_profiler::{CountingAllocator, Recorder, scope};

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator::new();

fn main() {
    CountingAllocator::enable();
    scope::enable();

    let mut recorder = Recorder::new("my-workload");
    for _ in 0..100 {
        recorder.record(|| {
            // ...exercise parser + renderer...
        });
    }
    let report = recorder.finish();
    println!("{}", report.render_table());
}
```

## 性能作業の推奨流れ

1. 何かを変える **前** に、代表コーパスに対してプロファイラを走らせます。表または JSON を残します。
2. 明らかにメモリ帯域で縛られていない、いちばん高い `share` の span を見つけます。`allocs` と `bytes` 列を見てください。span 単位の割り当てはたいてい低い実を取れます。
3. その span を狙う変更をします。
4. 同じフラグでプロファイラを再実行し、span 回数、割り当て、末尾パーセンタイルを比べます。
5. `cargo bench -p ox_content_parser` を走らせ、合成ベンチマークが退行していないことを確認します。

これが [issue #159](https://github.com/ubugeeei-prod/ox-content/issues/159) を着地させたループです。`docs/content/api/types.md` の最初の実行では、`parse_html_block` がパイプライン時間の 86.9% を食い、`to_ascii_lowercase()` が行ごとに割り当てていました。それをバイトレベルの大文字小文字を無視する検索に置き、`consume_line` の改行走査をインライン化すると、同じファイルは端から端まで 240 MB/s → 803 MB/s になり、反復あたりの割り当ては 122 → 32 に減りました。
