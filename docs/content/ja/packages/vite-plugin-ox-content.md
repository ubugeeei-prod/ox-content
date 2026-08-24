---
title: "@ox-content/vite-plugin"
description: Environment API 対応の Ox Content Vite プラグイン。
---

# @ox-content/vite-plugin

ドキュメントサイトとコンテンツパイプラインの既定の入口です。

```bash
vp install @ox-content/vite-plugin
```

`@ox-content/napi` は依存に含まれるので、Vite 経路では別途入れなくて構いません。

```ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
    }),
  ],
});
```

オプションの契約と一覧は [組み込み機能](../built-in-features.md) です。導入手順は [はじめに](../getting-started.md)、API と移行手順の全文は英語の [Vite Plugin](/packages/vite-plugin-ox-content.md) です。
