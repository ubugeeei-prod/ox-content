---
title: "@ox-content/code-play"
description: オンデマンドのドキュメントサンプル実行向け、オプトイン API と UI です。
---

# @ox-content/code-play

Code Play は、ドキュメントのサンプルをオンデマンドで実行します。別プラグインです。
`@ox-content/vite-plugin` は有効にせず、このパッケージを入れただけでは何も起きません。
言語を列挙するまで動きません。

このサイトの [ドキュメント例](/examples/code-play.md) とスタンドアロンの
[`examples/code-play`](https://github.com/ubugeeei-prod/ox-content/tree/main/examples/code-play)
アプリは、JavaScript と TypeScript だけを有効にしています。Rust、Go、リモート言語は
オプトインするまでオフです。

## インストール

<pm>npm install @ox-content/code-play</pm>

```ts
import { oxContent } from "@ox-content/vite-plugin";
import { codePlay } from "@ox-content/code-play";

export default {
  plugins: [
    oxContent({ highlight: true }),
    codePlay({
      languages: {
        typescript: { execute: true, typecheck: true },
        javascript: true,
      },
      ui: "default",
      viewers: { config: true, stdio: true, stderr: true, provenance: true, timing: true },
      srcDir: "content",
    }),
  ],
};
```

このプラグインは、パッケージインストールの上に乗る二段目のオプトインです。
`play` のないフェンス、または一覧にない言語は、普通のハイライト付きブロックのままです。

## プラグインオプション

| オプション  | 型                                              | 既定       | 役割                                           |
| ----------- | ----------------------------------------------- | ---------- | ---------------------------------------------- |
| `languages` | `Record<string, true \| LanguageEnableOptions>` | `{}`       | execute / typecheck / `endpoint` を有効化      |
| `ui`        | `"default" \| "compact" \| "headless"`          | `default`  | サンプル周りのクロム                           |
| `viewers`   | `Partial<ViewerFlags>`                          | すべてオン | stdio / stderr / config / … の表示             |
| `timeoutMs` | `number`                                        | `10000`    | 実行ごとのタイムアウト                         |
| `endpoints` | `{ rust?, go?, typecheck? }`                    | official   | プレイグラウンド / typecheck の URL            |
| `proxy`     | `boolean`                                       | `true`     | Vite **dev** の `/__ox-code-play/*` をマウント |
| `srcDir`    | `string`                                        | `"docs"`   | play フェンス照合に使う Markdown ルート        |
| `outDir`    | `string`                                        | Vite out   | SSG 後に拡張する書き出し HTML                  |
| `base`      | `string`                                        | `"/"`      | `ox-code-play.js` の公開パス                   |

`LanguageEnableOptions` は、その言語のスキーマ向けに `execute`、`typecheck`、`endpoint`、
`config` の上書きを受け付けます（TypeScript の `strict`、Rust の
`crateType`、Go の `withVet`、…）。

## 執筆

フェンスに `play` を付けます。言語が対応しているときは `typecheck` も足せます。

````md
```ts play
const n: number = 1;
console.log(n);
```

```rust play typecheck
fn main() {
    println!("ok");
}
```
````

HTML / MDX 形式:

```html
<CodePlay lang="python"> print("hello") </CodePlay>
```

## Headless API

```ts
import { createCodePlay } from "@ox-content/code-play";

const play = createCodePlay({ languages: { typescript: true } });
const session = play.createSession({
  language: "ts",
  code: "const n: number = 1;\nconsole.log(n);",
});

const check = await session.typecheck();
const run = await session.run();

run.stdio; // timestamped stdin / stdout / stderr events
run.stdout; // concatenated stdout text
run.stderr; // concatenated stderr text
run.provenance; // where it compiled, where it ran
run.timing; // phase durations and totalMs
session.config; // editable language config
```

有効にしていない言語を求めると `createCodePlay()` は throw します。
`session.setConfig({ strict: false })` は、config ビューアーが編集するのと同じオブジェクトを更新します。
`session.cancel()` は進行中の run または typecheck を中止し、
`status: "cancelled"` を返します。既定のツールバーは、実行中に **Cancel** を出します。
テストでは `transport`（たとえば `createMemoryTransport`）を注入し、
CI がライブのプレイグラウンドに触れないようにします。

| フィールド        | 意味                                                        |
| ----------------- | ----------------------------------------------------------- |
| `run.status`      | `ok` / `error` / `timeout` / `cancelled` / `unsupported`    |
| `run.stdio`       | タイムスタンプ付きの `stdin` / `stdout` / `stderr` イベント |
| `run.stdout`      | 連結した stdout テキスト                                    |
| `run.stderr`      | 連結した stderr テキスト                                    |
| `run.diagnostics` | 任意の行 / 列付きのコンパイラ / ランタイムメッセージ        |
| `run.provenance`  | どこでコンパイルし、どこで実行したか                        |
| `run.timing`      | フェーズ時間と `totalMs`                                    |
| `run.preview`     | バックエンドが UI のときのフレームワーク iframe `srcdoc`    |
| `session.stdout`  | `lastResult.stdout` と同じ                                  |
| `session.stderr`  | `lastResult.stderr` と同じ                                  |

## UI

| プリセット | 振る舞い                                                        |
| ---------- | --------------------------------------------------------------- |
| `default`  | ツールバーと stdio / stderr / config / provenance / timing タブ |
| `compact`  | Run / type-check と stdio、stderr                               |
| `headless` | DOM クロムなし。セッション API を使う                           |

ビューアーは `viewers` で個別に切り替えられます。

## 言語

| 言語                      | 実行 | 型チェック | バックエンド                                    |
| ------------------------- | ---- | ---------- | ----------------------------------------------- |
| TypeScript                | yes  | yes        | ローカル strip-types + `tsgo` + `node:vm`       |
| Rust                      | yes  | yes        | `play.rust-lang.org`（または `endpoints.rust`） |
| Go                        | yes  | yes        | `play.golang.org`（または `endpoints.go`）      |
| JavaScript                | yes  | no         | `node:vm` / サンドボックス iframe               |
| Vue、React、Svelte、Solid | yes  | no         | iframe `srcdoc` + esm.sh import map             |
| Python、PHP、Ruby、sh、…  | yes  | no         | Piston 互換の `languages.<id>.endpoint`         |

完全なカタログは [ロードマップ](/code-play-roadmap.md) と同じ一覧です。
`ts`、`c++`、`bash`、`coq` のようなエイリアスは正規 id に解決されます。

## プレイグラウンドプロキシ

Vite の **dev サーバー** のみです。`codePlay({ proxy: true })`（既定）は次をマウントします。

| パス                             | 転送先                                                        |
| -------------------------------- | ------------------------------------------------------------- |
| `POST /__ox-code-play/rust`      | `endpoints.rust`（既定 `https://play.rust-lang.org/execute`） |
| `POST /__ox-code-play/go`        | `endpoints.go`（既定 `https://play.golang.org/compile`）      |
| `POST /__ox-code-play/typecheck` | ローカル `tsgo`（リモートコンパイラなし）                     |

これらのルートは **POST** のみを受け付け、本文を 256 KiB で上限し、
`http(s)` 以外の宛先や埋め込み認証情報付き URL を拒否します。上流の
失敗は汎用 JSON `{ "error": "..." }` を返し、fetch の詳細は漏らしません。

プロキシは本番の SSG 出力には入りません。公開ページでは `endpoints` を公式
プレイグラウンド（または自分の HTTPS 実行器）に向けるか、
dev ミドルウェアが不要なら `proxy: false` にしてください。

静的ホストは `POST /__ox-code-play/typecheck` を提供しません。TypeScript の
**Run** はブラウザ内で動きます（型を剥がしてからサンドボックス iframe）。
到達可能な `endpoints.typecheck` を設定しない限り、公開ウィジェットから
**Typecheck** ボタンは省かれます。Vite プロキシ経路は `vite dev` のあいだだけ使います。

公開ページ上の Rust と Go は、ブラウザから直接 `endpoints.rust` / `endpoints.go`
を呼びます。公式プレイグラウンドは CORS で拒否することがあります。
ローカル docs では Vite プロキシを使い続けるか、`endpoints` を自分で制御する実行器へ向けてください。

## セキュリティ

`play` フェンスは、出荷する他のスクリプトと同じ **信頼できるサイトコンテンツ** です。
訪問者が書いたものや未レビューの断片に `play` を付けないでください。

- サンプルは Markdown transform や SSG のあいだには実行されません。
- **JavaScript / TypeScript の実行** は Node では `node:vm`、ブラウザでは
  `<iframe sandbox="allow-scripts">`（`allow-same-origin` なし）です。
  ページ起源の `Function` では決して動きません。サンプルはホストページの DOM や
  ストレージを読めません。
- **Vue / React / Svelte / Solid** プレビューは同じ iframe フラグと
  `srcdoc` を使います。プレビューランタイムは `esm.sh` から読みます。
- `sh` は docs ホスト上でローカルシェルを起動しません。
- **Rust / Go** はソースを `play.rust-lang.org` / `play.golang.org`
  （または `endpoints` の上書き）へ POST します。それらのホストはサンプルを見ます。
  プライバシーポリシーが適用されます。
- Piston 互換の `languages.<id>.endpoint` はその言語のソースを受け取ります。
  信頼できる HTTPS エンドポイントだけを、埋め込み認証情報なしで設定してください。

## 初回公開

`@ox-content/code-play` は npm では新しいです。Trusted publishing はパッケージを
作れないので、メンテナーがノート PC から **一度** 公開し、そのあと trusted publisher を足します。
コマンドと npmjs.com の正確な欄は
[リリース作業](/release.md#first-time-npm-publishing) にあります。

後続 PR は [Code Play ロードマップ](/code-play-roadmap.md) を見てください。
