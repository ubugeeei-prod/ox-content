---
title: 相互参照
description: 図、表、セクション向けのオプトイン label と生成リンク。
---

# 相互参照

`crossReferences` は安定した label を `Section 1.2`、`Figure 1`、`Table 1`
のような生成リンクに変換します。オプトインで、通常の `id` 属性を使うため、見出しパーマリンク、検索インデックス、独自 renderer と同じ label を共有できます。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      attrs: true,
      images: true,
      crossReferences: true,
    }),
  ],
};
```

## 書き方

対象種別の prefix を持つ label を置き、`@id` で参照します。

```md
## Install {#sec-install}

See @sec-install and @fig-pipeline.

![Pipeline](./pipeline.png "Build pipeline"){#fig-pipeline}

| Option         | Value  |
| -------------- | ------ |
| mode           | static |
| {#tbl-options} |

See @tbl-options.
```

生成される参照はリンクです。

```html
<a class="ox-xref ox-xref-section" href="#sec-install">Section 1.1</a>
<a class="ox-xref ox-xref-figure" href="#fig-pipeline">Figure 1</a>
<a class="ox-xref ox-xref-table" href="#tbl-options">Table 1</a>
```

対象要素には安定した metadata 属性が付きます。

```html
<h2
  id="sec-install"
  data-ox-xref-kind="section"
  data-ox-xref-number="1.1"
  data-ox-xref-label="Section 1.1"
>
  Install
</h2>
```

## Label Prefix

| Prefix              | 対象種別          | 生成テキスト |
| ------------------- | ----------------- | ------------ |
| `sec-` / `section-` | 見出し            | `Section N`  |
| `fig-` / `figure-`  | figure または画像 | `Figure N`   |
| `tbl-` / `table-`   | 表                | `Table N`    |

セクション番号は見出し階層に従います。図と表の番号はページ内の出現順です。セクション、図、表を並べ替えると、生成される参照テキストも更新されます。

## 診断

見つからない label、重複 label、prefix と対象種別の不一致は既定で error です。

```ts
oxContent({
  crossReferences: {
    missing: "error",
    duplicates: "error",
    mismatches: "error",
  },
});
```

移行中に未解決の参照を literal のまま残したい場合は、policy を `"warn"` にできます。

変換は fence、インデント code、inline code、raw `<pre>` / `<code>` /
`<script>` / `<style>`、HTML コメント、既存リンクの中をスキップします。

## Metadata

Markdown module は収集した対象を `crossReferences` として export します。
`transformMarkdown()` も同じ配列を返します。

```ts
import page from "./guide.md";

for (const reference of page.crossReferences) {
  console.log(reference.id, reference.kind, reference.text);
}
```

各 entry は `id`、`kind`、`number`、`label`、`text`、`href` と、見出し本文、画像 alt、figcaption から取った任意の `title` を持ちます。

## 関連

- [構文拡張](./syntax-extensions.md) - `attrs` label 構文。
- [画像](./images.md) - figure と figcaption の出力。
- [Markdown の土台](./markdown.md) - 見出しと GFM table。
