---
title: Graphviz DOT
description: dot / graphviz フェンスをビルド時にサニタイズ済みの静的 SVG へ描画します。
---

# Graphviz DOT

Graphviz の描画はオプトインです。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      graphviz: true,
    }),
  ],
};
```

有効にすると、` ```dot ` と ` ```graphviz ` フェンスはビルド中にインライン
SVG へ描画されます。出力は静的 HTML と SVG だけなので、ページに図描画用の
runtime やクライアント JavaScript は載りません。

## 例

````md
```dot
digraph pipeline {
  rankdir=LR
  Markdown -> Parser -> Renderer -> HTML
}
```
````

Graphviz 出力は安定した markup で包まれます。

```html
<figure class="ox-graphviz" role="img" aria-label="Graphviz diagram">
  <svg><!-- sanitized Graphviz output --></svg>
</figure>
```

生成 SVG は埋め込む前に制限されます。script 系の内容、event handler 属性、
fragment 以外の link は取り除かれます。SVG の ID と参照は図ごとに prefix
されるため、同じ図を複数回置いても衝突しません。

## Renderer command

既定では Ox Content が `dot -Tsvg` を起動し、DOT source を stdin に渡します。
別の互換 command や固定引数も指定できます。

```ts
oxContent({
  graphviz: {
    command: "dot",
    args: ["-Gbgcolor=transparent"],
  },
});
```

renderer が見つからない場合、既定では build を失敗させます。Graphviz のない
CI image でも一時的に進めたい場合は、`missingRenderer: "warn"` で元の code
block を残せます。

```ts
oxContent({
  graphviz: {
    missingRenderer: "warn",
  },
});
```

不正な DOT source も既定では失敗です。元の fence を残して best-effort の docs
build にしたい場合だけ `renderErrors: "warn"` を使ってください。

## Options

| Option            | Default   | 説明                                                  |
| ----------------- | --------- | ----------------------------------------------------- |
| `command`         | `"dot"`   | 実行する Graphviz 互換 command。                      |
| `args`            | `[]`      | `-Tsvg` の前に渡す追加引数。                          |
| `missingRenderer` | `error`   | command がないときの `error` / `warn`。               |
| `renderErrors`    | `error`   | Graphviz が graph を拒否したときの `error` / `warn`。 |
| `timeout`         | `10000`   | 図ごとの timeout。ミリ秒。                            |
| `cache`           | `true`    | 現在の process で描画済み raw SVG を cache します。   |
| `cacheTTL`        | `3600000` | cache TTL。ミリ秒。                                   |

## 関連

- [Mermaid](./mermaid.md) — もう一つの静的 diagram renderer。
- [コンポーネント CSS](./component-styles.md) — 独自 host では `styles/graphviz.css` を import。
- [組み込み機能の一覧](../built-in-features.md)
