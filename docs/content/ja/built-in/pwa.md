---
title: PWA マニフェストとサービスワーカー
description: 生成 HTML の横に書くオプトインの Web アプリマニフェストと保守的なオフラインキャッシュ。
---

# PWA マニフェストとサービスワーカー

`pwa` を有効にし、`ssg.siteUrl` があると、SSG ビルドは生成 HTML の横に Web
アプリマニフェストを書き、既定では保守的なサービスワーカーも書きます。

- `manifest.webmanifest` — 名前、開始 URL、テーマ色、standalone 表示
- `sw.js` — ハッシュ付き `assets/` と HTML ページをキャッシュ

テーマ付きページには `<link rel="manifest">` も付きます。オフラインキャッシュが
オンのときは、`sw.js` を登録する小さなスクリプトも入ります。

**これはクライアント JavaScript を追加します。** 登録スクリプトはブラウザで
動きます。サービスワーカーは同一オリジンの `GET` を横取りします。機能を
オンにするまで既存サイトは変わりません。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      pwa: true,
      ssg: {
        siteUrl: "https://example.com",
      },
    }),
  ],
};
```

`false` または省略ではファイルもクライアントスクリプトもオフです。`true` は
既定を有効にします。マニフェストとオフラインキャッシュです。オブジェクトは
機能をオンにしたうえで、指定したフィールドだけ上書きします。

```ts
oxContent({
  pwa: {
    name: "Docs",
    shortName: "Docs",
    themeColor: "#0f172a",
    backgroundColor: "#ffffff",
    startUrl: "/docs/",
  },
  ssg: {
    siteUrl: "https://example.com",
    siteName: "Docs",
  },
});
```

## マニフェストのみ（オフラインキャッシュなし）

`offline: false` にすると、インストール用メタデータは残し、サービスワーカー
と登録スクリプトは出しません。`manifest.webmanifest` と
`<link rel="manifest">` は書きます。`sw.js` は書きません。

```ts
oxContent({
  pwa: {
    offline: false,
    name: "Docs",
  },
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

| オプション        | 型                       | 既定                                  |
| ----------------- | ------------------------ | ------------------------------------- |
| `pwa`             | `boolean` / `PwaOptions` | `false`                               |
| `offline`         | `boolean`                | `true`                                |
| `name`            | `string`                 | `ssg.siteName`                        |
| `shortName`       | `string`                 | `name`                                |
| `themeColor`      | `string`                 | `#000000`                             |
| `backgroundColor` | `string`                 | `#ffffff`                             |
| `startUrl`        | `string`                 | Vite の `base`（`/` または `/docs/`） |

`start_url` と `scope` はオリジン付き絶対 URL ではなく、サイト相対パスです。
`javascript:`、プロトコル相対の `//`、パス以外の値は `base` に戻します。
テーマ色と背景色は `#rgb` / `#rrggbb` / `#rrggbbaa` または CSS 色名です。
不正な値は既定に戻します。

## オフラインキャッシュの挙動

サービスワーカーは保守的です。

- **HTML ページ** は **network-first** です。キャッシュはネットワーク失敗時だけ使います。
- `{base}assets/` 配下の **ハッシュ付きアセット**（`ox-content-*-{hash}.css` / `.js`）は
  **cache-first** です。
- クロスオリジンと `GET` 以外は無視します。

インストール時にサイト全体を事前キャッシュしません。ページは成功した
ナビゲーションのあとでキャッシュに入ります。

## `ssg.siteUrl`

`pwa` を有効にしても `ssg.siteUrl` が無いと、ファイルは書かず、テーマ付き
ページにもマニフェストリンクや登録スクリプトは付きません。ビルドは続き、
警告が出ます。

## クライアント JavaScript

`offline` を有効にすると（`pwa` オン時の既定）、テーマ付きページへ登録
スクリプトを注入します。

```html
<script>
  if ("serviceWorker" in navigator) navigator.serviceWorker.register("/sw.js");
</script>
```

ベア出力（`ssg.bare`）にはリンクもスクリプトも付きません。`siteUrl` があれば
ファイルは書くので、独自シェル側でワーカーを登録できます。

名前、色、URL はエスケープされるので、マニフェスト JSON や注入 HTML 属性から
抜け出せません。

## 関連

- [サイト生成](./site-generation.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [組み込み機能の概要](../built-in-features.md)
- [英語版ガイド](/built-in/pwa.md)
