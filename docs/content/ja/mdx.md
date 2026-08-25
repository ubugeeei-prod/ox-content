---
title: MDX とコンポーネント
description: island ハイドレーションで、Markdown に Vue、React、Svelte コンポーネントを埋め込みます。
---

# MDX とコンポーネント

Ox Content では、Markdown と `.mdx` ファイルの中にフレームワークコンポーネントを埋め込めます。
動き方を理解しておく価値があります。いわゆる「クラシック」な MDX とは違います。

- **JSX 要素、モジュールレベルの `import` / `export`、本文の `{expression}` は、
  MDX が有効なときにパースされます。** `.mdx` ではそれが既定です。
  `mdx: true` / `ParserOptions.mdx` を付けると、設定したすべての拡張子で
  同じ経路を有効にできます。Rust パーサーは PascalCase とメンバー名の
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
  `.mdx` ファイル（または `mdx: true`）では、React / Vue / Svelte / Solid
  プラグインは MDX AST を歩きます。AST が使えないときは、描画済みの
  `data-ox-island` 名を使います。グローバルな `components` マップにある名前、
  またはその文書から解決した相対 `import` の名前だけ、ハイドレーション用
  モジュールを import します。入れ子の JSX、式属性、フラグメントもその走査に
  入ります。一致する import のない未登録 JSX は静的 HTML のままです。
  素の `.md` は既存ページのためグローバルマップのソース走査のままです。
  式は保存され、評価は後です。

そのため、本文は Markdown の速さのまま、必要なところだけ本物の対話コンポーネントを置けます。
コンポーネントのないページには JavaScript バンドルを出しません。

## 既定

`mdx` を省略すると、Ox Content はソースの拡張子から推論します。

| ソース              | 既定                                 | `mdx: true` | `mdx: false`     |
| ------------------- | ------------------------------------ | ----------- | ---------------- |
| `.mdx`              | MDX オン（JSX、ESM、`{expression}`） | MDX オン    | CommonMark + GFM |
| `.md` / `.markdown` | CommonMark + GFM                     | MDX オン    | CommonMark + GFM |

`.mdx` に `mdx: true` は**不要**です。`.md` でも同じ構文を使いたいときだけ
`mdx: true` / `ParserOptions.mdx` を付けます。`.mdx` をプレーンな
Markdown 経路のままにするなら `mdx: false` です。

## 静的 HTML と island

フレームワークプラグインが無いとき、HTML レンダラーは静的経路のままです。

- **小文字 / カスタム HTML タグ**（`<div>`、`<note>`）は HTML のままです。
  island にはなりません。
- **PascalCase / メンバー名のタグ**（`<NoteCard />`、`<Icons.Star />`）は
  `data-ox-island` プレースホルダーになります。props は直列化されます。
  React / Vue / Svelte / Solid プラグインが無いあいだはハイドレートしません。
- **モジュールレベルの `import` / `export`** は `MdxjsEsm` ノードになります。
  HTML には**出ません**し、**実行もされません**。
- **本文の `{expression}`** は AST ソースとして保存され、**評価されません**。
  静的 HTML レンダラーはいまのところこれらのノードには何も出しません。
  ソースはテキストとしても JavaScript としても漏れません。

本物の `.mdx` ページがある、実行可能な Vite + `@ox-content/vite-plugin`
サイトは
[`examples/mdx`](https://github.com/ubugeeei-prod/ox-content/tree/main/examples/mdx)
です。

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

## 文書ローカルな import

`.mdx`（または `mdx: true`）では、サイト全体の `components` マップに登録せず、
文書自身からの相対 import でコンポーネントを使えます。

```md
import GtvChart from './gtv-chart/GtvChart.tsx'

<GtvChart title="ok" />
```

specifier はそのファイルのディレクトリから解決されます。束縛はその文書だけに
効きます。2 つのページが同じ名前 `Chart` を別ファイルから import しても、
グローバル名は衝突しません。そのページが実際に使うコンポーネントだけが、
静的な `import` として生成モジュールに入ります。コンポーネントファイルを
変えると、Vite HMR がその Markdown モジュールを無効化します。

| 形                                           | island として解決するか            |
| -------------------------------------------- | ---------------------------------- |
| `import Name from './file.tsx'`              | `<Name />` を使っていればする      |
| `import { Chart as Plot } from './file.tsx'` | `<Plot />` を使っていればする      |
| bare / npm / `https:` specifier              | しない。報告するだけで解決しない   |
| `srcDir` を出る `../`                        | しない。診断を出し、import しない  |
| 文書 import とグローバルマップに同じ名前     | そのファイルでは文書 import が勝つ |
| `mdx: true` のない `.md`                     | しない。ESM は文書 import ではない |

グローバルな `components` マップは、ローカル import を書かないページ向けの
後方互換フォールバックのままです。フレームワークプラグインは任意で
`renderIsland(name, props, filePath)` フックを渡し、transform 時に island の
内側 HTML を差し替えられます。そのフックはアダプタ側に置きます。コア
レンダラーは `react-dom/server`、`svelte/server`、`solid-js/web` を
import しません。

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

MDX が有効なとき、生成される Vite モジュールはそれらの AST ノードから集めた
構造化メタデータも export します。transform 中にユーザーの JavaScript は
**実行されません**。`import` 文は生きた ESM としては再出力されず、JSON
データになります。

```ts
import { html, frontmatter, toc, imports, exports, components } from "./guide.mdx";

html;
// string — 描画済み HTML（island。生きた import はなし）

frontmatter;
// object — パース済み YAML

toc;
// array — 見出しツリー

imports;
// [
//   {
//     source: "./Alert",
//     specifiers: [{ imported: "default", local: "Alert", kind: "default" }],
//   },
//   {
//     source: "./Chart",
//     specifiers: [{ imported: "Chart", local: "Plot", kind: "named" }],
//   },
//   {
//     source: "./icons",
//     specifiers: [{ imported: "*", local: "Icons", kind: "namespace" }],
//   },
// ]

exports;
// ["title", "helper"]

components;
// ["Alert", "Badge", "Icons.Star"]
```

`imports` の各要素は 1 つの文です。specifier の `kind` は `default`、
`named`、または `namespace` です。`exports` は export された名前のリストです
（`export default` は `default`）。`components` は文書順の一意な
PascalCase / メンバー JSX 名です。フラグメント（`<>...</>`）は除きます。
MDX がオフのとき、または MDX ノードがないファイルでは、これら 3 つの
export は空配列になり、モジュールの形は安定したままです。

```md
import Alert from './Alert'
import { Chart as Plot } from './Chart'
import * as Icons from './icons'

export const title = 'Guide'
export function helper() {}

<Alert />

Hello <Badge /> and <Icons.Star />
```

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
フレームワークプラグインは MDX AST から登録済みの名前と文書ローカルな
import を解決し、それらの island を後でハイドレートします。未登録の名前は、
レンダラーがすでに出した静的 HTML のままです。

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
`.mdx` ではその一覧は AST と、グローバルなコンポーネントマップおよび
解決済みの文書ローカル import の交差です。入れ子やフラグメント内のタグも、
登録されているか、そのページが import していればハイドレートされます。

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

- [組み込み MDX の例](/examples/mdx.md)
- [React 連携](/packages/vite-plugin-ox-content-react.md)
- [Vue 連携](/packages/vite-plugin-ox-content-vue.md)
- [Svelte 連携](/packages/vite-plugin-ox-content-svelte.md)
- [Solid 連携](/packages/vite-plugin-ox-content-solid.md)
