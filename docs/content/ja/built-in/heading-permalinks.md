---
title: 見出しパーマリンク
description: 生成済み見出し id を再利用する、オプトインの可視 # リンク。
---

# 見出しパーマリンク

見出しにはすでに安定した `id` が付きます。このオプションは、URL を手で直さなくても
`#section` をコピーしたり開けたりできるコントロールを見出しの横に出します。

明示的にオンにするまでオフです。オフの出力は変わりません。

| オプション               | 型                                     | 既定      |
| ------------------------ | -------------------------------------- | --------- |
| `headingPermalinks`      | `boolean` / `HeadingPermalinksOptions` | `false`   |
| `theme.headingPermalink` | `"hover"` / `"always"`                 | `"hover"` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      headingPermalinks: true,
    }),
  ],
};
```

`false` または省略は HTML をバイト単位で据え置きます。`true` または `{}` は既定で
オンにします。オブジェクトなら `enabled: false` で形だけ残せます。

## マークアップ契約

レンダラは生成済みの見出し id をそのまま使います。もう一度 slug 化しません。
重複や Unicode の id はアウトラインと一致します。

```html
<h2 id="hello-world">
  Hello World<a class="header-anchor" href="#hello-world" aria-label='Permalink to "Hello World"'
    >#</a
  >
</h2>
```

- コントロールは HTML 内の本物の `<a href="#id">` です。CSS や JavaScript が無くても使えます。
- アクセシブル名には見出しテキストが入ります。空の見出しは
  `Permalink to this section` です。
- すでに `class="header-anchor"` がある見出し、または同じ id への `#` リンクがある見出しには、
  二個目のマーカーを付けません。
- 明示的な `{#custom-id}` 属性は、パーマリンクの `href` をその id に書き換えます。

このサイトは `headingPermalinks` をオンにしているので、このページの見出しにも
コントロールがあります。

## 見え方

`theme.headingPermalink` は CSS だけを変えます。見出し HTML は同じです。

```ts
oxContent({
  headingPermalinks: true,
  ssg: {
    theme: {
      headingPermalink: "always",
    },
  },
});
```

| 値       | 表示                                            |
| -------- | ----------------------------------------------- |
| `hover`  | hover / `:focus-visible` で表示。タッチでは常時 |
| `always` | 常に表示                                        |

hover 表示は CSS だけです。クライアント JS、hydration、レイアウト計測はありません。
`prefers-reduced-motion` は不透明度のトランジションを切ります。余白は論理プロパティなので
RTL でも揃います。

## 関連

- [Markdown の土台](./markdown.md) — 見出し id とアウトライン。
- [テーマ](../theming.md) — 既定テーマの `headingPermalink`。
- [組み込み機能の一覧](../built-in-features.md)
