---
title: "@ox-content/napi"
description: Ox Content の Rust コア向け Node.js バインディング。
---

# @ox-content/napi

Rust コアを Node.js から呼ぶバインディングです。

```bash
vp install @ox-content/napi
```

リリースは macOS arm64/x64、Linux arm64/x64 GNU、Windows x64 MSVC のネイティブバイナリを配ります。他の Node.js プラットフォームはソースビルドできることがありますが、事前ビルドパッケージは出しません。

```ts
import { parseMarkdown } from "@ox-content/napi";

const ast = parseMarkdown("# Hello\n\n**bold**", { gfm: true });
```

Vite プラグイン経由ならこのパッケージを直接入れなくて構いません。API の全文は英語の [N-API](/packages/napi.md) です。
