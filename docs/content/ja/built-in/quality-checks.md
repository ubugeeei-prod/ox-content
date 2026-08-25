---
title: 品質チェック
description: コードフェンスの lint、TypeScript スニペットの型チェック、docs をテストとして実行、ビルド時の HTML サニタイズ。
---

# 品質チェック

コードサンプルが腐ると、ドキュメントも腐ります。これらのオプトインチェックは Markdown 変換中に走るので、壊れたスニペットは読者に届く前にビルドを落とします。

| オプション           | 既定    | 検査対象                                             |
| -------------------- | ------- | ---------------------------------------------------- |
| `codeBlockLint`      | `false` | フェンス衛生: 言語欠落、末尾スペース。               |
| `codeBlockTypecheck` | `false` | TypeScript フェンスがコンパイルするか。`tsgo` 経由。 |
| `docsTests`          | `false` | 実行可能なフェンスが Vitest で通るか。               |
| `sanitize`           | `false` | 描画 HTML を許可リストに照らす。                     |

チェックは静的です。変換中にドキュメントのコードは実行しません。`docsTests` はコードを実行しますが、CI から呼ぶ別の Vitest ハーネスの中だけです。

## コードブロック lint

ネイティブのスキャナはソース位置付きの診断を報告し、ファイルに実際にフェンスがあるときだけ走ります。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      codeBlockLint: {
        requireLanguage: true,
        trailingSpaces: true,
        mode: "error",
      },
    }),
  ],
};
```

| オプション        | 既定     | 目的                                            |
| ----------------- | -------- | ----------------------------------------------- |
| `languages`       | すべて   | lint するフェンス言語を制限する。               |
| `requireLanguage` | `false`  | 言語識別子のないフェンスを報告する。            |
| `trailingSpaces`  | `true`   | フェンス内の末尾空白を報告する。                |
| `mode`            | `"warn"` | `"warn"` はログ。`"error"` は変換を失敗させる。 |

ラベルなしフェンスと末尾スペースがあるページでは、変換は次を報告します。

```
code-block-language at 3:1
  Code block is missing a language identifier.
code-block-trailing-spaces at 8:13
  Code block line has trailing whitespace.
```

`mode: "warn"`（既定）では診断をログしてビルドは続きます。`mode: "error"` では変換が throw し、ビルドは失敗します。

## コードブロックの型チェック

[tsgo](https://github.com/microsoft/typescript-go)（ネイティブ TypeScript コンパイラ）で TypeScript フェンスを型チェックします。

```ts
oxContent({
  codeBlockTypecheck: {
    languages: ["ts", "tsx"],
    requireMeta: true,
    tsgoCommand: "tsgo",
    mode: "error",
  },
});
```

| オプション    | 既定            | 目的                                              |
| ------------- | --------------- | ------------------------------------------------- |
| `languages`   | `["ts", "tsx"]` | コンパイラへ送るフェンス言語。                    |
| `requireMeta` | `true`          | `typecheck` / `twoslash` 付きフェンスだけ検査。   |
| `tsgoCommand` | `"tsgo"`        | 起動するコンパイラバイナリ。                      |
| `mode`        | `"warn"`        | `"warn"` はログ。`"error"` はビルドを失敗させる。 |

既定の `requireMeta: true` では、著者はフェンスごとにオプトインするので、意図して不完全なスニペットも残せます。

````md
```ts typecheck
const value: string = "ok";
```
````

`requireMeta: false` にすると、すべての TypeScript フェンスを検査します。スニペットは一時ディレクトリに書き、`tsgo --noEmit` でコンパイルします。コンパイラエラーはソースフェンスを指すビルド診断になります。

`tsgo` は `@typescript/native-preview` に付きます。

<pm>npm install -D @typescript/native-preview</pm>

## Docs テスト

実行可能なフェンスを取り出して Vitest で走らせます。Rust の doctest と同じ「ドキュメントがテストされる」流れです。取り出しはネイティブで、文書全体を JavaScript でパースしません。

```ts
import { runDocsTests } from "@ox-content/vite-plugin";

await runDocsTests({
  include: ["docs/content/**/*.md"],
  vitestCommand: "vitest",
  vitestArgs: ["run"],
});
```

フェンスがテストになるのは、meta に `test`、`runnable`、`vitest`、または `docs-test` があるときです。

````md
```ts docs-test
import { expect } from "vitest";

const result = 1 + 1;
expect(result).toBe(2);
```
````

各フェンスは生成された Vitest の `test(...)` で包みます。`source: "jsdoc"` に切り替えると、Markdown ではなくソースコードの `@example` フェンスを走らせます。[生成 API ドキュメント](../jsdoc.md) を動かすのと同じ抽出器です。ハーネスオプション（独自テストを宣言するフェンス向けの `executionMode: "module"` を含む）は [Vitest Docs Tests](/examples/vitest-docs-test.md) を見てください。

## HTML サニタイザ

信頼できない、または混在したコンテンツを描画するとき、最終 HTML をサニタイズします。

```ts
oxContent({
  sanitize: {
    allowedUrlSchemes: ["http", "https", "mailto"],
  },
});
```

次の入力があるとき、

```html
<p onclick="alert(1)">
  Hello
  <script>
    alert(2);
  </script>
  <a href="javascript:alert(3)">link</a>
</p>
```

サニタイザは次を出します。

```html
<p>Hello <a>link</a></p>
```

script 要素、イベントハンドラ属性、安全でない URL スキームは、ネイティブの 1 パスで除きます。`sanitize: true` は、メディア要素（`video`、`audio`、`source`、`track`、`picture`）を含む、よくあるドキュメント HTML を残す安全な既定を使います。`allowedTags`、`allowedAttributes`、`allowedUrlSchemes` を渡すと、対応する組み込みリストを **置き換え** ます。足し算ではありません。

パスはパイプラインのいちばん最後 — [埋め込み](./embeds.md) の展開のあと — に走るので、許可リストはページが実際に出すものすべてに効きます。

## 関連

- [コードブロック](./code-blocks.md) — これらの検査が守るフェンス向けのハイライト、注釈、取り込み。
- [型ホバー](./typed-hover.md) — `twoslash` フェンスのビルド時型オーバーレイ
- [HTML Sanitizer の例](/examples/html-sanitizer.md)
