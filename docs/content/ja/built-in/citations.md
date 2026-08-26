---
title: 引用
description: 静的ドキュメント向けの、bibliography に基づくオプトイン引用構文。
---

# 引用

`citations` は短い引用参照を、ローカル CSL JSON に基づくアクセシブルなリンクへ変換します。脚注とは別機能なので、同じ出典を何度引用しても bibliography entry は 1 件にまとまります。

この機能は静的です。bibliography file は変換時に読み、出力は HTML で、クライアント JavaScript は追加しません。このページでは HTTP Semantics RFC を実際の renderer で引用しています [@rfc9110]。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      citations: {
        bibliography: "content/references.json",
        rootDir: process.cwd(),
      },
    }),
  ],
};
```

## 書き方

1 件の引用は `[@key]`、複数引用はセミコロンで区切ります。

```md
HTTP semantics are defined by the core RFC [@rfc9110].

CommonMark and HTTP are both external references [@commonmark; @rfc9110].
```

本文側で著者や標準名をすでに書いている場合は、group 内で `-@key` を使えます。

```md
RFC 9110 defines HTTP semantics [-@rfc9110].
```

生成される引用は通常のリンクです。

```html
<span class="ox-cite" role="group" aria-label="Citations 1">
  <a class="ox-cite__ref" href="#ref-rfc9110">[1]</a>
</span>
```

`appendBibliography` が有効なら、使われた出典だけが 1 回ずつ出ます。

```html
<section class="ox-bibliography" aria-labelledby="ox-bibliography-title">
  <h2 class="ox-bibliography__title" id="ox-bibliography-title">References</h2>
  <ol class="ox-bibliography__list">
    <li class="ox-bibliography__item" id="ref-rfc9110">...</li>
  </ol>
</section>
```

## Bibliography File

最初の形式は CSL JSON です。file は `rootDir` の下にあるローカルパスだけです。URL と、`rootDir` の外へ出るパスは描画前に失敗します。

```json
[
  {
    "id": "rfc9110",
    "title": "HTTP Semantics",
    "author": [{ "given": "Roy T.", "family": "Fielding" }],
    "issued": { "date-parts": [[2022]] },
    "URL": "https://www.rfc-editor.org/rfc/rfc9110"
  }
]
```

## Options

| Option               | 型                    | 既定            |
| -------------------- | --------------------- | --------------- |
| `enabled`            | `boolean`             | `true`          |
| `bibliography`       | `string` / `string[]` | `[]`            |
| `rootDir`            | `string`              | `process.cwd()` |
| `appendBibliography` | `boolean`             | `true`          |
| `missing`            | `"error"` / `"warn"`  | `"error"`       |
| `duplicates`         | `"error"` / `"warn"`  | `"error"`       |
| `malformed`          | `"error"` / `"warn"`  | `"error"`       |
| `bibliographyTitle`  | `string`              | `"References"`  |

既存ドキュメントを移行するときは、診断 policy を `"warn"` にできます。warn mode では未解決の引用はリテラルのまま残り、変換は警告を出します。

## Metadata

Markdown module は custom renderer 向けに引用 metadata を export します。

```ts
import page from "./guide.md";

for (const cite of page.citations) {
  console.log(cite.key, cite.href, cite.label);
}

for (const entry of page.bibliography) {
  console.log(entry.key, entry.title);
}
```

`renderMarkdown()` も同じ `citations` / `bibliography` 配列を返します。search index の本文には、`citations` が有効なときだけ bibliography title が入ります。

## Styling

組み込み SSG は `.ox-cite` または `.ox-bibliography` を描画するページにだけ citation CSS を追加します。独自ホストでは対応するシートを import します。

```css
@import "@ox-content/vite-plugin/styles/core.css";
@import "@ox-content/vite-plugin/styles/citations.css";
```

## 関連

- [Markdown の土台](./markdown.md) - 脚注は独立したままです。
- [相互参照](./cross-references.md) - ページ内 target への生成 label。
- [コンポーネント CSS](./component-styles.md) - 公式 CSS エントリポイント。
