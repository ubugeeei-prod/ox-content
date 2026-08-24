---
title: MDX とコンポーネント
description: island ハイドレーションで、Markdown に Vue、React、Svelte コンポーネントを埋め込みます。
---

# MDX とコンポーネント

Ox Content では、Markdown と `.mdx` ファイルの中にフレームワークコンポーネントを埋め込めます。
動き方を理解しておく価値があります。いわゆる「クラシック」な MDX とは違います。

- **JSX 要素、モジュールレベルの `import` / `export`、本文の `{expression}` は、
  MDX が有効なときにパースされます。** `mdx: true` /
  `ParserOptions.mdx` があると、Rust パーサーは PascalCase とメンバー名の
  タグを `MdxJsxFlowElement` / `MdxJsxTextElement` ノードにします（自己閉じ
  または開閉、リテラル、真偽、`{expr}`、spread 属性付き）。
  ファイルレベルの `import` / `export` は `MdxjsEsm` ノードになり、
  文書レベルの `{foo}` / `Hello {name}` は `MdxFlowExpression` /
  `MdxTextExpression` になります。フラグメント（`<>...</>`）、JSX コメント、
  `{expression}` の子は AST ソースとして保存されます。評価はしません。
  そのオプションがオフなら、`.md` は CommonMark + GFM のままです。
- **コンポーネントはレンダラーではなく、フレームワークプラグインが解決します。**
  HTML レンダラーは名前付き MDX JSX を island プレースホルダーにし、
  props を直列化します（リテラルは JSON、`{expression}` / spread はソース）。
  React / Vue / Svelte プラグインは、ハイドレーション用に PascalCase タグを発見し、
  式は後で評価します。

そのため、本文は Markdown の速さのまま、必要なところだけ本物の対話コンポーネントを置けます。
コンポーネントのないページには JavaScript バンドルを出しません。

## セットアップ

公式の Vite プラグインと並べて、自分のフレームワーク用プラグインを追加し、
コンポーネントを指します。

```ts
// vite.config.ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { oxContentReact } from "@ox-content/vite-plugin-react";

export default defineConfig({
  plugins: [
    react(),
    oxContentReact({
      srcDir: "docs",
      // Auto-discover components by glob…
      components: "./src/components/*.tsx",
      // …or map names explicitly:
      // components: { Counter: "./src/components/Counter.tsx" },
    }),
  ],
});
```

Vue、Svelte、Solid も同じように `@ox-content/vite-plugin-vue`
（`oxContentVue`）、`@ox-content/vite-plugin-svelte`（`oxContentSvelte`）、
`@ox-content/vite-plugin-solid`（`oxContentSolid`）で動きます。`components` が glob のとき、
コンポーネント名は PascalCase にしたファイル名です。

Solid 連携は加えて `vite-plugin-solid` より前に動かす必要があり、
そちらには Markdown 拡張子を渡す必要があります。
[そのリファレンスページ](/packages/vite-plugin-ox-content-solid.md#plugin-order-and-extensions) を見てください。

## Markdown でコンポーネントを書く

Markdown では PascalCase タグとしてコンポーネントを書きます。自己閉じでも子付きでも構いません。

```md
# My Page

Regular **Markdown** prose. Hello {name}.

{count + 1}

<Counter initial={5} />

<Callout type="tip">

# Title

Hello **world**.

- nested
  - list

<Badge />

</Callout>

<>
<Icons.Star />
{label}
</>

<Card {...cardProps} />

{/_ Hidden from the rendered page _/}
```

大文字で始まるタグだけが JSX / コンポーネントとして扱われるので、
普通の HTML（`<div>`、`<span>`、…）は raw HTML のままです。メンバー名
（`Foo.Bar`）、フラグメント（`<>...</>`）、spread（`{...props}`）、JSX コメント
（`{/* note */}`）、`{expression}` の子、文書レベルの
`{expression}` は MDX がオンのときにパースされます。式のソースは保存され、
実行されません。フェンス付きコードブロックやインラインコード内のタグは
**コンポーネントでも式でもありません**。

ファイル先頭（および他の ESM のあと）のモジュールレベル `import` と `export` は
`MdxjsEsm` ノードになります。複数行の文は、素朴な brace / paren / 文字列 / コメント走査で集めます。
JavaScript パーサーではないので、正規表現リテラルやテンプレート内の `${}` は文境界を混乱させることがあります。
フェンスやインラインコード内の `import` / `export` は ESM ではありません。
`import x from "<script>"` のような敵対的な文字列はソースとして保存し、panic しません。
HTML レンダラーはいまのところ `MdxjsEsm` や `{expression}` ノードには何も出しません。
フレームワークプラグインが後で import を解決し、式を評価します。

文書レベルの `{expression}` も素朴な brace / 文字列 / コメント走査を使い、
JavaScript パーサーではないので、正規表現リテラルは境界を混乱させることがあります。
閉じられていない `{` は普通のテキストのままです。フェンスとインラインコードは式になりません。
`{ "<script>" }` のような敵対的なソースは保存され、HTML としては出ません。

MDX がオンのとき、コンポーネントのタグのあいだの Markdown は Markdown としてパースされ、
island ラッパーの **内側** に HTML として描画されます（`<h1>`、`<strong>`、
リスト、フェンス）。フェンスとインラインコードはコードのままです。フェンス内の `<Alert />` は
island ではありません。入れ子の PascalCase タグは入れ子の island になります。
閉じられていないタグがファイルの残りを飲み込むことはありません。子の敵対的な raw HTML、
たとえば `<script>alert(1)</script>` は無力化されます（先頭の `<` をエスケープ）ので実行できません。
フラグメント（`<>...</>`）は island ラッパーなしで Markdown の子を描画します。

MDX がオンのとき、名前付き JSX コンポーネントは HTML の island プレースホルダー
（`data-ox-island="Name"`）になります。属性はその island に直列化され、
実行されません。

- 引用文字列、真偽属性、JSON リテラルの `{42}` / `{true}` /
  `{"a":1}` 値は JSON 安全な props になります
- それ以外の `{expression}` は **ソース文字列** として保存されます
- `{...spread}` 属性は **spread ソースのリスト** になります

ペイロードは `<`、`>`、`&` を unicode エスケープした JSON で、
`data-ox-props`（HTML エスケープ済み）と、ブラウザが実行しない
`<script type="application/json">` に置かれます。
`{"</script><script>"}` や `{alert(1)}` のような敵対的なソースはペイロードから抜けられず、
評価もされません。コンポーネントのないページは `<script>` も island ランタイムも出しません。
フレームワークプラグインは後でコンポーネントを解決してハイドレートします。

### Props

Props は JSX 風の構文です。次の形を認識します。

| 構文               | 直列化先                    |
| ------------------ | --------------------------- |
| `prop="text"`      | 文字列                      |
| `prop={42}`        | 数値 / JSON 値              |
| `prop={true}`      | 真偽値                      |
| `prop={ {"a":1} }` | オブジェクト（JSON）        |
| `prop`             | 真偽値 `true`               |
| `prop={count + 1}` | 式ソース（評価しない）      |
| `{...props}`       | spread ソース（評価しない） |

リテラル props、式ソース、spread は island 要素上でまとめて直列化されます。
ハイドレーションは後です。このスライスはペイロードを保存するだけです。

## island のハイドレーション

各コンポーネントは生成 HTML の island ラッパーになります。ブロックレベルの
コンポーネントは `<div data-ox-island="Name" …>`、インラインは
`<span data-ox-island="Name" …>` です。対応するフレームワークランタイムが、
クライアントで本物のコンポーネントをその要素へマウントします。

ハイドレーションのタイミングはロード戦略で制御します
（[`@ox-content/islands`](./packages/vite-plugin-ox-content.md) を見てください）。

| 戦略      | ハイドレートするタイミング                             |
| --------- | ------------------------------------------------------ |
| `eager`   | 読み込み直後（既定）                                   |
| `idle`    | `requestIdleCallback` 中（約 200 ms のフォールバック） |
| `visible` | 要素がビューに入ったとき（`IntersectionObserver`）     |
| `media`   | メディアクエリが一致したとき（`matchMedia`）           |

サーバー出力はプレーン HTML なので、ハイドレーションの前（またはなし）でもページは描画され読めます。
island の JavaScript は、そのページが実際に使うコンポーネント分だけ読み込まれます。

## テーマ内の静的 JSX

コンポーネント island とは別に、Ox Content は小さな **静的 JSX
ランタイム**（`jsx`、`jsxs`、`Fragment`、`renderToString`、`raw`、`when`、`each`）を同梱しています。
クライアント側 JavaScript なしで HTML 文字列に描画するテーマやレイアウトを書くために使います。
`tsconfig.json` で設定します。

```json
{
  "compilerOptions": {
    "jsx": "react-jsx",
    "jsxImportSource": "@ox-content/vite-plugin"
  }
}
```

これは `@ox-content/vite-plugin/jsx-runtime` を解決し、`jsx` が `react-jsxdev` のときは
`@ox-content/vite-plugin/jsx-dev-runtime` も解決します。どちらも HTML 文字列へ描画します。
React はなく、オプトインする開発専用の振る舞いもありません。

カスタムレイアウトの作り方は [テーマ](./theming.md) を見てください。

## 関連

- [React 連携](/packages/vite-plugin-ox-content-react.md)
- [Vue 連携](/packages/vite-plugin-ox-content-vue.md)
- [Svelte 連携](/packages/vite-plugin-ox-content-svelte.md)
- [Solid 連携](/packages/vite-plugin-ox-content-solid.md)
