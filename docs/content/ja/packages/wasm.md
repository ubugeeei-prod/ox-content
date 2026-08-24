# @ox-content/wasm

Ox Content の Rust Markdown エンジン向け WebAssembly バインディングです。

ブラウザ、Web Worker、または ESM から `.wasm` を読み込める別の JavaScript 環境で Ox Content を動かしたいときに、このパッケージを使ってください。

Node.js 向けに作るなら、[`@ox-content/napi`](./napi.md) を優先してください。

## インストール

```bash
vp install @ox-content/wasm
```

## 使い方

```ts
import init, { parseAndRender, WasmParserOptions } from "@ox-content/wasm";

await init();

const options = new WasmParserOptions();
options.gfm = true;
options.tables = true;
options.taskLists = true;

const result = parseAndRender("# Hello from WASM", options);
console.log(result.html);
```

## API

現在の WebAssembly 面が公開しているのは次です。

- `parseAndRender(source, options?)`
- `transform(source, options?)`
- `version()`
- `WasmParserOptions`

## 補足

- このパッケージは ESM のみです。
- 既定の `init()` 呼び出しは、パッケージ入口からの相対パスで `ox_content_wasm_bg.wasm` を読み込みます。
- そのため、`.wasm` アセットに対応したバンドラーやブラウザ向けランタイムと相性が良いです。

## ローカルビルド

リポジトリルートから:

```bash
vp run build:wasm
cd crates/ox_content_wasm/pkg
vp exec -- npm pack --dry-run
```

生成された公開可能なパッケージは `crates/ox_content_wasm/pkg/` にあります。

## 初回のローカル公開

このパッケージを手元のマシンから初めて公開する場合:

```bash
cd crates/ox_content_wasm/pkg
vp exec -- npm whoami || vp exec -- npm login
vp exec -- npm publish --access public
```

スコープ付きパッケージなので、初回公開は `--access public` を使ってください。

このモノレポでは、生成されたパッケージだけを対象にする `crates/ox_content_wasm/pkg` からの公開がいちばん安全です。

パッケージが npm に存在したあとは、リポジトリのタグベース GitHub Actions ワークフローが、provenance を有効にして後続バージョンを公開できます。
