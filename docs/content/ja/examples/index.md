---
title: 事例
description: 実行可能な例と小さなソーススニペット。
---

# 事例

Ox Content は、さまざまな使い方を示す実行可能な例と小さなソーススニペットを提供します。個別の事例ページは英語です。ヘッダーの locale switcher で戻れます。

組み込み機能の小さなスニペットは、リポジトリの `examples/builtin-features/` にあります。

## 連携の例

### [Vue 連携](/examples/integ-vue.md)

`@ox-content/vite-plugin-vue` を使って、Markdown に Vue 3 コンポーネントを埋め込みます。

```ts
import { oxContentVue } from "@ox-content/vite-plugin-vue";

export default defineConfig({
  plugins: [
    vue(),
    oxContentVue({
      components: "./src/components/*.vue",
    }),
  ],
});
```

### [React 連携](/examples/integ-react.md)

`@ox-content/vite-plugin-react` を使って、Markdown に React コンポーネントを埋め込みます。

```ts
import { oxContentReact } from "@ox-content/vite-plugin-react";

export default defineConfig({
  plugins: [
    react(),
    oxContentReact({
      components: "./src/components/*.tsx",
    }),
  ],
});
```

### [Svelte 連携](/examples/integ-svelte.md)

`@ox-content/vite-plugin-svelte` を使って、Markdown に Svelte 5 コンポーネントを埋め込みます。

```ts
import { oxContentSvelte } from "@ox-content/vite-plugin-svelte";

export default defineConfig({
  plugins: [
    svelte(),
    oxContentSvelte({
      components: "./src/components/*.svelte",
    }),
  ],
});
```

### [Solid 連携](/examples/integ-solid.md)

`@ox-content/vite-plugin-solid` を使って、Markdown に Solid コンポーネントを埋め込みます。

```ts
import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import { oxContentSolid } from "@ox-content/vite-plugin-solid";

export default defineConfig({
  plugins: [
    oxContentSolid({
      components: "./src/components/*.tsx",
    }),
    // Solid's JSX is compile-time only, so this plugin runs after
    // oxContentSolid() and needs the Markdown extensions.
    solid({ extensions: [".md", ".markdown", ".mdx"] }),
  ],
});
```

## プラグインの例

### [コード注釈](/examples/code-annotations.md)

カスタム属性と互換記法の両方を使える、オプトインのコードブロック注釈です。

```ts
oxContent({
  highlight: true,
  codeAnnotations: {
    notation: "both",
  },
});
```

### [パッケージマネージャータブ](/examples/package-manager-tabs.md)

パッケージマネージャータブをオプトインし、ひとつの npm コマンドを書いて npm/pnpm/yarn/bun のインストールタブとして描画します。

```md
<pm>npm install -D vite</pm>
```

### [markdown-it プラグイン](/examples/plugin-markdown-it.md)

既存の markdown-it プロジェクトで、Ox Content を markdown-it プラグインとして使います。

### [rehype プラグイン](/examples/plugin-rehype.md)

unified エコシステムで、Ox Content を rehype プラグインとして使います。

### [unplugin mdast ブリッジ](/examples/unplugin-mdast-bridge.md)

Ox Content のネイティブパーサーの上で、カスタム mdast プラグインと既存の remark/unified プラグインを動かします。互換境界とブリッジの性能メモも文書化しています。

### [unplugin markdown-it トークンブリッジ](/examples/unplugin-markdown-it-token-bridge.md)

まず `markdown-it` プラグインを動かし、その結果のトークンストリームを下流の unified プラグインから読みます。

## ジェネレーターの例

### [ソースドキュメント生成](/examples/gen-source-docs.md)

JSDoc/TSDoc コメントから API ドキュメントを自動生成します。

```ts
oxContent({
  docs: {
    src: ["./src"],
    out: "docs/api",
    include: ["**/*.ts"],
  },
});
```

## OG 画像の例

### [OG Viewer](/examples/og-viewer.md)

すべてのページの Open Graph メタデータをプレビューする開発ツールです。開発中は `/__og-viewer` で使えます。

### [カスタム OG 画像テンプレート](/examples/og-image-custom.md)

カスタムテンプレートでページごとの Open Graph 画像を生成します。任意の frontmatter データを props として渡せます。

```ts
oxContent({
  ogImage: true,
  ogImageOptions: {
    template: "./og-template.ts",
  },
});
```

## その他の例

### [Code Play](/examples/code-play.md)

`@ox-content/code-play` によるオンデマンドのサンプル実行です。stdio、stderr、config、provenance、timing のビューアー付きです。ドキュメントページ上のライブフェンスに加え、スタンドアロンの Vite アプリは [`examples/code-play`](https://github.com/ubugeeei-prod/ox-content/tree/main/examples/code-play) です。

### [プレイグラウンド](/examples/playground.md)

Markdown パースを試す対話的な Web プレイグラウンドです。

### [Vite SSG](/examples/ssg-vite.md)

Vite を使った静的サイト生成の例です。

## 事例を動かす

```bash
# Clone the repository
git clone https://github.com/ubugeeei-prod/ox-content.git
cd ox-content

# Install dependencies
npm install

# Run an example
cd examples/integ-vue
npm run dev

# Code Play (JS / TS widgets)
corepack pnpm --filter ./examples/code-play dev
```
