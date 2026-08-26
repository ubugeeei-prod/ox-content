---
title: Code Play
description: stdio、stderr、config、provenance、timing ビューアー付きのオンデマンドサンプル実行です。
---

# Code Play

このページは `@ox-content/code-play` で **JavaScript** と **TypeScript** だけを有効にします。
Code Play がないページは通常の docs ページのままで、`ox-code-play.js` を読み込みません。

## プラグインを有効にする

```ts
import { oxContent } from "@ox-content/vite-plugin";
import { codePlay } from "@ox-content/code-play";

export default {
  plugins: [
    oxContent({ highlight: true }),
    codePlay({
      languages: {
        javascript: true,
        typescript: { execute: true, typecheck: true },
      },
      ui: "default",
      viewers: { config: true, stdio: true, stderr: true, provenance: true, timing: true },
    }),
  ],
};
```

## TypeScript サンプル

```ts play typecheck play-title="Strict TypeScript" play-target=ESNext
const message: string = "hello from Code Play";
console.log(message);
console.warn("this warning is a stderr chunk");
```

## サンプルごとの config

`play-<config-key>=...` は、そのサンプルだけ言語 config を上書きします。

```ts play typecheck play-title="Loose TypeScript" play-strict=false play-compact
const label = "works without an explicit type annotation";
console.log(label.toUpperCase());
```

## 実行時エラー

```js play play-title="Runtime error"
console.log("before");
throw new Error("boom from the example");
```

## Headless usage

```ts
import { createCodePlay } from "@ox-content/code-play";

const play = createCodePlay({ languages: { typescript: true } });
const session = play.createSession({
  language: "ts",
  code: "const n: number = 1;",
});

const result = await session.run();
result.stdio;
result.stdout;
result.stderr;
result.provenance.compile;
result.provenance.execute;
result.timing.phases;
```

`RunActionState` ヘルパーは idle、running、result、error、offline を表します。
transport / CORS の失敗は `status: "offline"` です。

詳しくは [@ox-content/code-play](/packages/code-play.md) と
[ロードマップ](/code-play-roadmap.md) を見てください。
