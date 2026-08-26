---
title: マジックリンク
description: GitHub ユーザー、名前付きエイリアス、明示 URL 向けのオプトインなリッチリンク。
---

# マジックリンク

本文では人、プロジェクト、繰り返し出すサイトの名前がよく出ます。普通の
Markdown リンクは URL を繰り返し、安定したアバターやファビコンを付けられません。
`{link:...}` はオプトインで、既定はオフです。着想は
[markdown-it-magic-link](https://github.com/antfu/markdown-it-magic-link) にあります。

| オプション   | 型                             | 既定    |
| ------------ | ------------------------------ | ------- |
| `magicLinks` | `boolean` / `MagicLinkOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      magicLinks: {
        aliases: {
          Oxc: {
            href: "https://oxc.rs",
            image: "https://github.com/oxc-project.png",
          },
        },
      },
    }),
  ],
};
```

`false` または省略はソースを変えません。`true` またはオブジェクトは変換を
オンにします。`{link:...}` が書き換えられるまで、既存文書はバイト単位で同じです。

## 書き方

形は `{link:BODY}` です。`BODY` は次のいずれかです。

- GitHub ユーザー: `{link:@ryoppippi}` — プロフィール URL と
  `https://github.com/ryoppippi.png`
- GitHub ユーザー + ラベル: `{link:@ubugeeei|ox-content}` — 同じアバター、カスタムラベル
- GitHub ユーザー + ラベル + URL:
  `{link:@ubugeeei|ox-content|https://github.com/ubugeeei?tab=repositories}`
- 名前付きエイリアス: `{link:Oxc}` — 設定した `{ href, label?, image? }`
- 明示ラベル + URL: `{link:Example|https://example.com}` — `favicon` がオンでない限り画像なし

未知のエイリアス、危険なスキーム（`javascript:`、`data:`、`file:`）、壊れた
URL、閉じていないタグはリテラルのままです。ラベルと URL は HTML エスケープされます。

```md
See {link:@ryoppippi} and {link:Oxc}.
```

See {link:@ryoppippi} and {link:Oxc}.

## 画像

GitHub ユーザー形は組み立てたアバター URL を使います。エイリアスは
`image` があればそれを使います。明示 URL は `favicon` をオンにしない限り画像なしです。

```ts
oxContent({
  magicLinks: {
    favicon: {
      template: "https://icons.duckduckgo.com/ip3/{host}.ico",
    },
  },
});
```

`favicon: true` は `https://{host}/favicon.ico` を使います。変換時に
fetch はしません。URL を書くだけです。`imageOverrides` は exact な `href`
または `prefix` に対して解決済み画像を置き換えます。

出力のクラスは固定です。`ox-magic-link`、`ox-magic-link--github` /
`--alias` / `--url`、`ox-magic-link__image`、`ox-magic-link__label`。
画像は装飾（`alt=""`）で、ラベルがアクセシブルな名前です。

独自の `ssg: false` ホストは
`@ox-content/vite-plugin/styles/magic-links.css` を import してください
（`--octc-*` トークン用に `core.css` も大抵必要です）。
[コンポーネント CSS](./component-styles.md) を見てください。

## 対象外

フェンス、インデントコード、インラインコード、生の `<code>` / `<pre>` /
`<script>` / `<style>`、HTML 属性、すでにリンク済みの `[text](url)` は
書き換えません。

```md
`{link:@ryoppippi}`
```

`{link:@ryoppippi}`

走査はソースに対する 1 パスです。エイリアスは名前ごとの再パースではなく
マップ参照です。クライアント JavaScript はありません。

## 関連

- [構文拡張](./syntax-extensions.md)
- [インラインバッジ](./badges.md)
- [キーボードキー](./keyboard-keys.md)
- [コンポーネント CSS](./component-styles.md)
- [組み込み機能の一覧](../built-in-features.md)
- [markdown-it-magic-link](https://github.com/antfu/markdown-it-magic-link)
