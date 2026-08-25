---
title: はじめに
description: Ox Content を Vite プラグイン、N-API、WebAssembly、Rust crate のどこから使い始めるか。
---

# はじめに

Ox Content は 4 つの層から導入できます。

ドキュメントサイトを作るなら Vite プラグインから始めてください。
Node.js から Rust コアを呼ぶなら、次に N-API パッケージです。
ブラウザやサンドボックスでの実行が必要なら、次に WebAssembly パッケージです。
パーサとレンダラを Rust から直接使うなら、最後に Rust crate です。

コントリビュータ向けのセットアップとソースビルドは別ページです。[開発環境](./development-setup.md) を見てください。

## 入口を選ぶ

| やりたいこと                                            | ここから                                     |
| ------------------------------------------------------- | -------------------------------------------- |
| ドキュメントサイトやコンテンツパイプラインを作る        | [Vite プラグイン](#1-vite-プラグインから)    |
| Node.js からパーサとレンダラを呼ぶ                      | [N-API](#2-n-api-経由の-node-js-api)         |
| ブラウザや別の WebAssembly ホストで Ox Content を動かす | [WebAssembly パッケージ](./packages/wasm.md) |
| Rust プロジェクトに Ox Content を直接埋め込む           | [Rust crate](#4-rust-crate)                  |
| Ox Content 自体を触る                                   | [開発環境](./development-setup.md)           |

## 要件

| 経路            | 要件                                                              |
| --------------- | ----------------------------------------------------------------- |
| Vite プラグイン | Node.js `24+` と Vite または Vite+ プロジェクト                   |
| N-API           | Node.js `24+`                                                     |
| WebAssembly     | npm パッケージを入れ、ESM から `.wasm` を読める JS ツールチェーン |
| Rust crate      | Rust `1.95+`                                                      |

## 1. Vite プラグインから

ほとんどの利用者向けの既定の入口です。

Vite プラグインは Ox Content パイプライン一式を渡します。Markdown 変換、静的サイト生成、テーマ、検索、OG 画像、生成 API ドキュメントです。必要なネイティブランタイムはすでに同梱されるので、Vite 経路では `@ox-content/napi` を別途入れる必要はありません。よく使う Markdown の挙動は既定でオン、絵文字ショートコード、コード注釈、パッケージマネージャタブ、SNS 埋め込みといった非標準の執筆機能はオプトインです。選ぶときは [組み込み機能](./built-in-features.md) を見てください。

3.0 はいま alpha です。npm の `latest` は 2.90.0 のままなので、`alpha` ディストタグから入れてください。この系統の他の `@ox-content/*` も `@alpha` です。

### インストール

```bash
vp install @ox-content/vite-plugin@alpha
```

### 最小設定

```ts
// vite.config.ts
import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "content",
      outDir: "dist/docs",
      highlight: true,
      ogImage: true,
      docs: {
        enabled: true,
        src: ["./src"],
        out: "content/api",
      },
    }),
  ],
});
```

Markdown のエントリページを作ります。

```md
<!-- content/index.md -->

# Hello Ox Content

This site is generated from Markdown.
```

それから docs アプリを起動します。

```bash
vp dev
```

日本語ページを同じサイトに置くときは、`i18n` を有効にして `content/ja/` に翻訳を置き、`ssg.localeSwitcher: true` を付けます。既定ロケールはプレフィックスなし、日本語は `/ja/` です。詳細は [国際化](./i18n.md) と [ロケールスイッチャー](./built-in/locale-switcher.md) を見てください。

### フレームワーク連携

Markdown の中にコンポーネント island を置きたいときは、公式連携のどれかを足します。

```bash
# Vue
vp install @ox-content/vite-plugin-vue@alpha vue @vitejs/plugin-vue

# React
vp install @ox-content/vite-plugin-react@alpha react react-dom @vitejs/plugin-react

# Svelte
vp install @ox-content/vite-plugin-svelte@alpha svelte @sveltejs/vite-plugin-svelte

# Solid
vp install @ox-content/vite-plugin-solid@alpha solid-js vite-plugin-solid
```

続き:

- [@ox-content/vite-plugin](./packages/vite-plugin-ox-content.md)
- [組み込み機能](./built-in-features.md)
- [Vue 連携](/packages/vite-plugin-ox-content-vue.md)
- [React 連携](/packages/vite-plugin-ox-content-react.md)
- [Svelte 連携](/packages/vite-plugin-ox-content-svelte.md)
- [Solid 連携](/packages/vite-plugin-ox-content-solid.md)
- [テーマ](./theming.md)
- [ソース docs の例](/examples/gen-source-docs.md)

## 2. N-API 経由の Node.js API

Node.js のツール、スクリプト、独自 docs ワークフローの中で、Ox Content を速い Markdown エンジンとして使いたいときは `@ox-content/napi` です。

### インストール

```bash
vp install @ox-content/napi@alpha
```

### パースと描画

```ts
import { parseAndRender } from "@ox-content/napi";

const markdown = `
# Welcome

- Fast parser
- Rust core
- HTML output
`;

const result = parseAndRender(markdown, {
  gfm: true,
  tables: true,
  taskLists: true,
});

console.log(result.html);
```

### AST へパース

```ts
import { parseMarkdown } from "@ox-content/napi";

const ast = parseMarkdown("# Hello\n\nThis is **bold**.", {
  gfm: true,
});

console.log(JSON.stringify(ast, null, 2));
```

続き:

- [@ox-content/napi](./packages/napi.md)
- [@ox-content/wasm](./packages/wasm.md)
- [API リファレンス](/api/index.md)

## 3. WebAssembly（@ox-content/wasm）

ブラウザ、Web Worker、または別の WebAssembly ホストで Ox Content が必要なら `@ox-content/wasm` です。

### インストール

```bash
vp install @ox-content/wasm
```

### JavaScript から使う

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

既定の `init()` はパッケージエントリからの相対で `ox_content_wasm_bg.wasm` を読むので、バンドラや、ESM から `.wasm` アセットを扱える環境でよく動きます。

### このリポジトリからローカルでビルドして公開する

Ox Content 自体を保守しているなら、リポジトリが公開用 npm パッケージを作れます。

```bash
vp run build:wasm
cd crates/ox_content_wasm/pkg
vp exec -- npm pack --dry-run
```

このスコープ付きパッケージを初めてローカル公開するときは、必要ならレジストリに認証し、public として公開します。

```bash
cd crates/ox_content_wasm/pkg
vp exec -- npm whoami || vp exec -- npm login
vp exec -- npm publish --access public
```

`crates/ox_content_wasm/pkg` から公開するほうが、ワークスペースルートから公開するより安全です。生成パッケージだけを対象にするからです。

意図して広いワークスペース公開フローにしたいとき以外は、ここでワークスペースルートから公開しないでください。

npm アカウントが公開に 2FA を求めていると、`npm publish` のあいだにワンタイムコードを聞かれます。

いまの WASM 面は [`crates/ox_content_wasm/src/lib.rs`](https://github.com/ubugeeei-prod/ox-content/blob/main/crates/ox_content_wasm/src/lib.rs) から `parseAndRender`、`transform`、`version`、`WasmParserOptions` を出します。

## 4. Rust crate

いちばん低い層の部品を直接使いたいときは Rust crate です。

### 依存を足す

```toml
[dependencies]
ox_content_allocator = "3.0.0-alpha.1"
ox_content_ast = "3.0.0-alpha.1"
ox_content_parser = "3.0.0-alpha.1"
ox_content_renderer = "3.0.0-alpha.1"
```

### Rust でパースして描画する

```rust
use ox_content_allocator::Allocator;
use ox_content_parser::{Parser, ParserOptions};
use ox_content_renderer::HtmlRenderer;

fn main() {
    let allocator = Allocator::new();
    let markdown = "# Hello from Rust\n\n- Fast\n- Reusable\n- Markdown";

    let parser = Parser::with_options(&allocator, markdown, ParserOptions::gfm());
    let document = parser.parse().expect("failed to parse markdown");

    let mut renderer = HtmlRenderer::new();
    let html = renderer.render(&document);

    println!("{}", html);
}
```

より深い内部が必要なら、crate 単位の API は Rust ワークスペースにあり、[アーキテクチャ](./architecture.md) でも説明しています。

## Ox Content 自体をビルドする必要があるとき

リポジトリをクローンする、docs テーマを触る、N-API バインディングをローカルでビルドする、テスト一式を走らせる、といったときは、このページではなく [開発環境](./development-setup.md) を使ってください。
