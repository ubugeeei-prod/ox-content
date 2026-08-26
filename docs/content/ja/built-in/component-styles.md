---
title: コンポーネント CSS
description: ssg: false と transformAllPlugins() 向けの、公式コンポーネント CSS エントリ。
---

# コンポーネント CSS

組み込み SSG は、生成 HTML の横に機能 CSS をインラインします。`ssg: false`、
`transformAllPlugins()`、`ssg.render` で文書を自分で持つホストは、同じ
マークアップは受け取れますが、そのスタイルは付きません。

`@ox-content/vite-plugin` は、SSG がすでに使っている crate のスタイルシートを
公開します。描画するものだけ import してください。サイト固有のテーマは
アプリ側に残します。

```css
@import "@ox-content/vite-plugin/styles/core.css";
@import "@ox-content/vite-plugin/styles/magic-links.css";
@import "@ox-content/vite-plugin/styles/social.css";
@import "@ox-content/vite-plugin/styles/twitter-full.css";
```

全部まとめて取るとき:

```css
@import "@ox-content/vite-plugin/styles/all.css";
```

`transformAllPlugins()` が返すのは今までどおり HTML だけです。CSS は明示
import なので、コンパクトな Tweet だけ載せてフルカード用シートは省略できます。

## エントリポイント

| import                    | 対象                                                                                                          |
| ------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `styles/core.css`         | ベーストークン（`--octc-*`）と、SSG スタイルシートの既定 prose / chrome                                       |
| `styles/magic-links.css`  | `{link:...}` チップ                                                                                           |
| `styles/social.css`       | コンパクトな Tweet/X、Bluesky、プロバイダカード、Spotify、Apple Music、audio、video、StackBlitz、WebContainer |
| `styles/twitter-full.css` | `appearance: "full"` の Tweet カード。react-tweet / sveltweet の MIT 告知を含む                               |
| `styles/ogp.css`          | Open Graph カード                                                                                             |
| `styles/github.css`       | GitHub リポジトリ / ソースカード                                                                              |
| `styles/youtube.css`      | YouTube 埋め込み                                                                                              |
| `styles/tabs.css`         | タブとパッケージマネージャタブ                                                                                |
| `styles/mermaid.css`      | Mermaid 図                                                                                                    |
| `styles/graphviz.css`     | Graphviz DOT 図                                                                                               |
| `styles/not-by-ai.css`    | `<NotByAI />` 執筆開示バッジ                                                                                  |
| `styles/all.css`          | 上の機能シートをこの順で全部                                                                                  |

`var(--octc-*)` を使う機能シートは、先に `core.css` を読むか、ホスト側で同じ
トークンを定義してください。フル Tweet の chrome は独自の `--ox-tweet-*` を
持つので `core.css` は不要です。

これらのファイルはパッケージビルド時に `crates/ox_content_ssg` からコピー
されます。組み込み SSG も同じソースを読むので、公式 chrome が独自ホスト向け
import とずれません。

## 独自ホスト

モジュール変換器（`ssg: false`）:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      srcDir: "content",
      ssg: false,
      embeds: { twitter: { fetch: true, appearance: "full" } },
    }),
  ],
};
```

```css
@import "@ox-content/vite-plugin/styles/core.css";
@import "@ox-content/vite-plugin/styles/social.css";
@import "@ox-content/vite-plugin/styles/twitter-full.css";
```

`transformAllPlugins()` を直接呼ぶとき:

```ts
import { transformAllPlugins } from "@ox-content/vite-plugin";

const html = await transformAllPlugins(sourceHtml, {
  twitter: { fetch: true, appearance: "full" },
});
```

`html` を描画するホストで、対応するスタイルシートを import してください。
crate の CSS をアプリにコピーしないでください。

`renderMarkdown()` と `createMarkdownProcessor()` も同じです。返すのは
マークアップで、有効にした機能の公式シートは自分で import します。

## 関連

- [サイト生成](./site-generation.md)
- [マジックリンク](./magic-links.md)
- [NotByAI バッジ](./not-by-ai.md)
- [埋め込み](./embeds.md)
- [Twitter/X 埋め込み](/examples/twitter-embed.md)
- [@ox-content/vite-plugin](../packages/vite-plugin-ox-content.md)
