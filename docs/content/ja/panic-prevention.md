# Panic 防止

これは [issue #774](https://github.com/ubugeeei-prod/ox-content/issues/774) の最初の証拠台帳です。壊れた Markdown、設定、パス、プラグインデータは、ホストプロセスを abort するのではなく、エラーまたは診断を返さなければなりません。

#774 は **完了していません**。このページは、最初にマージできるスライスを記録します。棚卸し、CI ゲート、公開面で最も危険な修正、残りの除外です。

リリースバイナリは `panic = "abort"` です。公開 N-API 成果物の中で Rust が panic すると Node が落ちます。`catch_unwind` が効くのは debug / test ビルドだけです。根本対応は、ユーザー入力で panic しないことです。

## コマンド

```bash
# 棚卸し + allowlist ゲート（crates/ 配下の非テスト Rust）
node scripts/check-panic-constructs.mjs

# このスライス向けの回帰テスト
cargo test -p ox_content_parser --test input_panics
cargo test -p ox_content_ssg --lib paths -- --nocapture
cargo test -p ox_content_transform --lib hostile_user_content
cargo test -p ox_content_renderer --lib svelte_public_codegen
cargo test -p ox_content_napi hostile_markdown

# transform パイプライン全体に対する有界な fuzz レーン
cargo test -p ox_content_transform --test pipeline_fuzz
```

CI は `.github/workflows/ci.yml` の `Panic constructs` ジョブでゲートを実行します。`vp run check:panic-constructs` と `vp run workspace:check` も同じスクリプトです。

## 監査したサブシステム（このスライス）

| サブシステム                       | Crate                  | 結果                                                                                                                                                                                                      |
| ---------------------------------- | ---------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Markdown パース                    | `ox_content_parser`    | 公開 `parse` は `ParseResult` を返します。サブパーサーは入れ子深さを引き継ぐので、深い引用は際限なく伸びず `NestingTooDeep` を返します。delimiter / list ヘルパーは回復可能な不一致で `expect` しません。 |
| HTML 描画 / フレームワーク codegen | `ox_content_renderer`  | `String` への書き込みは `expect` しません。Svelte の公開 codegen は `unreachable!` に到達しません。                                                                                                       |
| Transform / frontmatter            | `ox_content_transform` | 壊れた YAML は空の frontmatter のままです。敵対的な Markdown は abort せず `errors` を返します。コンパイル時 YouTube 正規表現の `expect` は allowlist に残します。                                        |
| SSG ルート / エントリリンク        | `ox_content_ssg`       | パス接尾辞の除去はバイト安全です。`😀` のようなマルチバイトパスで文字境界の途中をスライスしません。                                                                                                       |
| N-API 公開 parse / transform       | `ox_content_napi`      | キャッシュミスは `napi::Error` です。unwind ビルドでは、想定外 panic を `errors` 配列に落とします。                                                                                                       |

テスト専用の `unwrap` / `expect` / `panic!` は対象外です。

## このスライスの修正

- **SSG パス**: `strip_markdown_extension` は末尾 N バイトを `&str` スライスで比較していました。4 バイトの絵文字パス（`😀`）は、比較の前に文字境界以外で panic していました。ヘルパーは ASCII 接尾辞をバイト列で比較します。
- **SSG エントリリンク**: `.md` の除去も同じバイト安全な接尾辞チェックです。
- **パーサーの list / emphasis**: 初期化後や retain 後の局所 `expect` は `get_or_insert_with` / `if let` になりました。
- **パーサーの入れ子**: サブソースのパーサー（引用・list item・脚注本文・JSX の子）はすべて親より 1 段深い深さで生成されるので、構文の組み合わせによらず `max_nesting_depth` が再帰の深さを縛ります。GFM プロファイルなしでも既定値は 100 です。`0`（無制限）だと深く入れ子になった文書がスタックを溢れさせ、スタックオーバーフローは巻き戻しではなく abort になるためです。
- **レンダラー / SWAR スキャン**: 長さ確認後の 8 バイト `try_into().unwrap()` はスタック配列へのコピーです。
- **N-API キャッシュ**: 変換済みファイルの欠落は `expect` ではなく `Result` エラーです。
- **N-API FFI**: `parse`、`parse_and_render`、`transform` は unwind ビルドで想定外 panic から回復します。

## CI ゲートと allowlist

`scripts/check-panic-constructs.mjs` は `crates/**/*.rs` を歩き、`tests/`、`benches/`、`examples/`、`tests.rs`、`#[cfg(test)]` モジュールを除いて次を数えます。

`unwrap`、`unwrap_err`、`unwrap_unchecked`、`expect`、`panic!`、`unreachable!`、`todo!`、`unimplemented!`

件数は `config/panic-allowlist.json` と比較します。新規ヒットは失敗です。実件数が下がった場合も、allowlist を減らすまで失敗します。ワークスペース全体の `allow(clippy::unwrap_used)` ではありません。

対象 5 crate は、非テストビルドで `clippy::unwrap_used`、`expect_used`、`panic`、`todo`、`unimplemented` を `deny` します。これらの crate でレビュー済みの例外は、`ox_content_transform` のコンパイル時 YouTube 正規表現だけです。

## 残作業（後続 PR）

- 残りのワークスペースを終える: `ox_content_docs`、`ox_content_lsp`、`ox_content_i18n`、`ox_content_search`、`ox_content_highlight`、`ox_content_wasm`、Vite バインディング、エディタ crate。
- 有界な fuzz レーンを transform パイプライン以外にも広げる。`cargo test -p ox_content_transform --test pipeline_fuzz` は通常のテストジョブで動き、トークンの寄せ集めと実際のブロックテンプレートの 2 通りで文書を生成して全機能を同時に通します（上記の `{.class}` と定義リストの abort はこれで見つかりました）。SSG・docs・エディタ側にはまだ同等のものがなく、`fuzz/` ターゲットは依然として nightly が必要で必須 CI ジョブではありません。
- 公開成果物の FFI 境界で `panic = "abort"` ではなく unwind を使えるかを決める。
- 残サイトを証明または書き換えるたびに `config/panic-allowlist.json` を縮める。

これらが終わるまで #774 は閉じません。
