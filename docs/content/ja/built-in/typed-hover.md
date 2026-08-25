---
title: 型ホバー
description: twoslash フェンス向けの、ビルド時 TypeScript 型ホバーオーバーレイ。コンパイラはブラウザに出しません。
---

# 型ホバー

TypeScript のサンプルは、すでに [`codeBlockTypecheck`](./quality-checks.md)
でビルド時に型検査できます。ただし、ここでオプトインしない限り、描画された
フェンス上で読者がその型を見ることはできません。

`typedHover` の既定はオフです。有効にすると、**`twoslash` が付いた
TypeScript / TSX フェンスだけ** がホバー用ペイロードを受け取ります。型は
Markdown 変換中に計算します。ページが送るのは JSON と小さなオーバーレイ
スクリプトだけです。**TypeScript コンパイラはブラウザでは動きません。**

| オプション   | 型                              | 既定    |
| ------------ | ------------------------------- | ------- |
| `typedHover` | `boolean` / `TypedHoverOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      typedHover: true,
    }),
  ],
};
```

`true` と `{}` は同じデフォルトで有効になります。省略または `false` では
どのフェンスも変わりません。オブジェクト形式では機能を有効にしたうえで
`languages`（既定 `["ts", "tsx"]`）だけ上書きできます。

このサイトでは `typedHover` を有効にしているので、次のブロックは実際の
オーバーレイです。`value` にホバーするか Tab でフォーカスしてください。

```ts twoslash
const value = 1;
```

## フェンスのメタ

著者はフェンスごとに `twoslash` メタでオプトインします。サイトオプションが
オンでも、そのトークンが無いフェンスはスキップされます。

````md
```ts twoslash
const value = 1;
```

```ts
const skipped = 1;
```
````

| フェンス                           | オーバーレイ |
| ---------------------------------- | ------------ |
| ` ```ts twoslash `                 | あり         |
| ` ```tsx twoslash `                | あり         |
| ` ```ts `（メタなし）              | なし         |
| ` ```js twoslash `                 | なし         |
| インライン `` `const value = 1` `` | なし         |

`twoslash` は [`codeBlockTypecheck`](./quality-checks.md) がすでに認識する
メタと同じです。2 つ目のマーカーを付けなくても、同じフェンスを型検査しつつ
ホバーを付けられます。検査したくない未完成スニペットは `typecheck` を省略し、
オーバーレイだけ欲しい場合は `twoslash` を使えます。

## ビルド時であり、ブラウザではない

ペイロードはページ変換中に作る `{ start, end, type }` の範囲です。プラグインは
既存の TypeScript フェンス経路を再利用します。`extractCodeBlocks` がスニペットを
書き出し、識別子オフセットの型を `tsgo`（`@typescript/native-preview`）に尋ねます。
ブラウザは `typescript` も `tsgo` も Language Service もダウンロードしません。

[`codeBlockTypecheck`](./quality-checks.md) と同じコンパイラを入れてください。

<pm>npm install -D @typescript/native-preview</pm>

オプトインした各フェンスには `class="ox-typed-hover"` が付きます。ペイロードは
隣の `<script type="application/json">` に入り、`<` / `>` は `\u003c` /
`\u003e` にエスケープされるので、型文字列がスクリプトを破ったりマークアップを
注入したりできません。

## キーボードとポインタ

型がある識別子は次の要素になります。

`<span class="ox-typed-hover-token" tabindex="0">`

- **ポインタ:** トークンにホバーすると小さなオーバーレイが開きます。
- **キーボード:** トークンへ Tab 移動します。フォーカスで同じオーバーレイが
  開き、`Escape` で閉じます。
- オーバーレイは `role="tooltip"` で、中身は `innerHTML` ではなく
  `textContent` で入れます。

トークンには点線の下線が付くので、マウスが無くても見つけられます。

## リテラルのまま残るもの

次は書き換えません。

- `twoslash` が無いフェンス
- JavaScript、JSON、その他 `ts` / `tsx` 以外のフェンス
- インラインコード
- インデントコード
- 閉じられていないフェンス（ファイルの残りをホバー対象として飲み込みません）

`<img onerror>` や `</script>` のような敵対的な型文字列は、JSON ペイロード内で
エスケープされ、オーバーレイではテキストとして描画されます。

## 関連

- [品質チェック](./quality-checks.md) — `tsgo` による `codeBlockTypecheck`
- [コードブロック](./code-blocks.md) — ハイライトと注釈
- [組み込み機能の概要](../built-in-features.md)
