---
title: はじめに
description: Ox Content を Vite プラグイン、N-API、WebAssembly、Rust crate のどこから使い始めるか。
---

# はじめに

Ox Content は 4 つの層から導入できます。

ドキュメントサイトを作るなら Vite プラグインから始めてください。
Node.js から Rust コアを呼ぶなら N-API パッケージです。
ブラウザやサンドボックスなら WebAssembly パッケージです。
Rust からパーサとレンダラを直接使うなら crate です。

コントリビュータ向けのソースビルドは [Development Setup](/development-setup.md) にあります。

## 入口を選ぶ

| やりたいこと                               | ここから                                  |
| ------------------------------------------ | ----------------------------------------- |
| ドキュメントサイトやコンテンツパイプライン | [Vite プラグイン](#1-vite-プラグインから) |
| Node.js からパーサとレンダラを呼ぶ         | [N-API](#2-n-api-経由の-node-js-api)      |
| ブラウザや Wasm ホストで動かす             | [WebAssembly](/packages/wasm.md)          |
| Rust プロジェクトに埋め込む                | [Rust crate](#4-rust-crate)               |

## 要件

| 経路            | 要件                                                              |
| --------------- | ----------------------------------------------------------------- |
| Vite プラグイン | Node.js `24+` と Vite または Vite+ プロジェクト                   |
| N-API           | Node.js `24+`                                                     |
| WebAssembly     | npm パッケージを入れ、ESM から `.wasm` を読める JS ツールチェーン |
| Rust crate      | Rust `1.95+`                                                      |

## 1. Vite プラグインから

ドキュメント用途の既定の入口です。

Vite プラグインは Markdown 変換、SSG、テーマ、検索、OG 画像、API ドキュメント生成まで一通り提供します。ネイティブランタイムは同梱されるので、Vite 経路では `@ox-content/napi` を別途入れる必要はありません。よく使う Markdown は既定で有効、絵文字ショートコードや注釈、埋め込みなど非標準の機能はオプトインです。選択肢は [組み込み機能](./built-in-features.md) を見てください。

### インストール

```bash
vp install @ox-content/vite-plugin
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

このサイトは Markdown から生成されます。
```

開発サーバー:

```bash
vp dev
```

日本語ページを同じサイトに置くときは、`i18n` を有効にして `content/ja/` に翻訳を置き、`ssg.localeSwitcher: true` を付けます。既定ロケールはプレフィックスなし、日本語は `/ja/` です。詳細は [国際化](./i18n.md) と [ロケールスイッチャー](./built-in/locale-switcher.md) を見てください。

## 2. N-API 経由の Node.js API

Node.js のツールや独自ワークフローから高速な Markdown エンジンとして使うなら `@ox-content/napi` です。英語のパッケージ案内は [N-API](/packages/napi.md) にあります。

## 3. WebAssembly

ブラウザやサンドボックスでは [WebAssembly パッケージ](./packages/wasm.md) を使います。

## 4. Rust crate

Rust から直接埋め込む場合の依存は [Getting Started (English)](/getting-started.md) の crate 節と同じバージョンを使ってください。

## 次のページ

- [組み込み機能](./built-in-features.md)
- [ドキュメントのバージョン管理](./built-in/versioning.md)
- [テーマ](./theming.md)
