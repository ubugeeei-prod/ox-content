---
title: 埋め込み
description: Markdown 中の HTML 風タグで書く GitHub / OG カード、パッケージマネージャタブ、メディア、SNS。
---

# 埋め込み

埋め込みは Markdown 中の HTML 風タグで、変換時に静的 HTML へ展開されます。静的マークアップだけを出す 2 つは既定でオン、それ以外はオプトインです。

| 埋め込み                 | オプション            | 既定    | 書き方                             |
| ------------------------ | --------------------- | ------- | ---------------------------------- |
| GitHub カード            | `embeds.github`       | `true`  | `<GitHub repo="owner/name" />`     |
| OG リンクカード          | `embeds.openGraph`    | `true`  | `<OgCard url="https://..." />`     |
| パッケージマネージャタブ | `embeds.pm`           | `false` | `<pm>npm install pkg</pm>`         |
| Twitter / X              | `embeds.twitter`      | `false` | `<Tweet />` または `<XPost />`     |
| Bluesky                  | `embeds.bluesky`      | `false` | `<Bluesky />`                      |
| Spotify                  | `embeds.spotify`      | `false` | `<Spotify url="https://..." />`    |
| StackBlitz               | `embeds.stackBlitz`   | `false` | `<StackBlitz url="https://..." />` |
| WebContainer             | `embeds.webContainer` | `false` | `<WebContainer />`                 |

タブと YouTube 埋め込みは `embeds` オプションの外です。SSG ビルドと dev preview では常に処理され、設定は不要です。同じ執筆モデルなので [下](#タブ) で扱います。

すべての組み込み埋め込みを切るときは `embeds: false`、個別に設定するときはオブジェクトです。

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        github: { maxSourceLines: 120 },
        openGraph: { timeout: 5000 },
        pm: { sync: true },
        twitter: true,
        bluesky: true,
      },
    }),
  ],
};
```

## GitHub カード

`embeds.github` はビルド時に GitHub API からリポジトリカードとソーススニペットを描画します。出力は静的 HTML です。クライアント側 JavaScript も、第三者ウィジェットのスクリプトも使いません。

リポジトリカード:

```md
<GitHub repo="ubugeeei-prod/ox-content" />
```

<GitHub repo="ubugeeei-prod/ox-content" />

ref と行範囲を固定したソーススニペット:

```md
<GitHub repo="ubugeeei-prod/ox-content" path="README.md" ref="main" loc="1-10" />
```

<GitHub repo="ubugeeei-prod/ox-content" path="README.md" ref="main" loc="1-10" />

パーマリンク形式も使えます。`#L10-L18` 行アンカー付きの GitHub blob URL を貼ります。

```md
<GitHub permalink="https://github.com/ubugeeei-prod/ox-content/blob/278098b/npm/vite-plugin-ox-content/src/plugins/github.ts#L10-L18" />
```

<GitHub permalink="https://github.com/ubugeeei-prod/ox-content/blob/278098b/npm/vite-plugin-ox-content/src/plugins/github.ts#L10-L18" />

ソースカードのヘッダは blob へリンクし、GitHub API が返すときは、その ref でそのパスを最後に触ったコミットも表示します。

| オプション       | 既定      | 目的                                                         |
| ---------------- | --------- | ------------------------------------------------------------ |
| `token`          | `""`      | レート制限とプライベートリポジトリ向け GitHub API トークン。 |
| `cache`          | `true`    | API 応答をメモリにキャッシュする。                           |
| `cacheTTL`       | `3600000` | キャッシュ寿命（ミリ秒）。                                   |
| `maxSourceBytes` | `200000`  | これより大きいファイルは飛ばす。                             |
| `maxSourceLines` | `120`     | 範囲指定がないときのインライン行数上限。                     |

明示的な `token` がなければ `process.env.GITHUB_TOKEN` を自動で拾います。ビルド中にリポジトリやファイルが取れないとき — オフライン CI、レート制限、不正なパス — 埋め込みはビルドを落とさず、フォールバックのリンクカードを描画します。

## Open Graph カード

`embeds.openGraph` はビルド時にページの Open Graph メタデータを取り、静的リンクカードを描画します。

```md
<OgCard url="https://vite.dev" />
```

<OgCard url="https://vite.dev" />

| オプション  | 既定                         | 目的                             |
| ----------- | ---------------------------- | -------------------------------- |
| `timeout`   | `10000`                      | 取得タイムアウト（ミリ秒）。     |
| `cache`     | `true`                       | 取得したメタデータをキャッシュ。 |
| `cacheTTL`  | `3600000`                    | キャッシュ寿命（ミリ秒）。       |
| `userAgent` | `ox-content-ogp-bot/1.0 ...` | 対象へ送る User-Agent。          |

届かないページはプレーンなリンクカードに落ちます。localhost、プライベート IP 範囲、HTTP(S) 以外のスキームへのリクエストは拒否するので、Markdown 本文がビルド環境のネットワークを探れません。

## パッケージマネージャタブ

`embeds.pm` は 1 つの npm 風コマンドを、npm、pnpm、yarn、bun、vp（Vite+）向けのアクセシブルなタブグループへ展開します。

```ts
oxContent({
  embeds: {
    pm: true,
  },
});
```

```md
<pm>npm install -D @ox-content/vite-plugin</pm>
```

<pm>npm install -D @ox-content/vite-plugin</pm>

コマンド変換は Rust ネイティブです。`npm install -D` は `pnpm add -D`、`yarn add -D`、`bun add -D`、`vp install -D` になり、`npx <bin>` は `vp exec -- <bin>` になります。タブはクライアント側 JavaScript なしで動きます。選択は CSS `:has()` です。`pm: { sync: true }` をオプトインすると、ページ上のすべてのブロックで選んだパッケージマネージャを `localStorage` 経由で同期します。変換表の全体は [Package Manager Tabs](/examples/package-manager-tabs.md) を見てください。

## タブ

汎用タブグループはパッケージマネージャタブと同じウィジェットで、SSG ビルドと dev preview では常に使えます。

```md
<tabs>
  <tab label="macOS">brew install oxc</tab>
  <tab label="Linux">apt install oxc</tab>
  <tab label="Windows">winget install oxc</tab>
</tabs>
```

<tabs>
  <tab label="macOS">brew install oxc</tab>
  <tab label="Linux">apt install oxc</tab>
  <tab label="Windows">winget install oxc</tab>
</tabs>

`label` 属性のない `<tab>` は `Tab 1`、`Tab 2` のように落ちます。

## YouTube

YouTube 埋め込みは SSG ビルドと dev preview で常に処理されます。iframe はプライバシー強化モード（`youtube-nocookie.com`）と遅延読み込みが既定です。

```md
<youtube id="aqz-KE-bpKQ" title="Big Buck Bunny" />
```

<youtube id="aqz-KE-bpKQ" title="Big Buck Bunny" />

`id`、`url`、`href` 属性を受け付けます。`youtu.be`、`watch?v=`、`shorts`、`embed` の URL 形はどれも認識します。

## Twitter / X

`embeds.twitter` は投稿を静的カードとして描画し、第三者ウィジェットのスクリプトは決して読みません。`twitter: true` のとき、埋め込みはプライバシーを意識したリンクカードです。

```md
<XPost url="https://x.com/jack/status/20" />
```

<XPost url="https://x.com/jack/status/20" />

オブジェクト形式を使うと、ビルド時に本文、著者、アバター、写真を取り、自分のオリジンから配信します。

```ts
oxContent({
  embeds: {
    twitter: {
      fetch: true,
      lang: "en",
      mediaOutputDir: "public/ox-content/twitter",
      mediaPublicPath: "/ox-content/twitter",
    },
  },
});
```

| オプション        | 既定                        | 目的                                                |
| ----------------- | --------------------------- | --------------------------------------------------- |
| `fetch`           | `false`                     | ビルド時に投稿本文を取る。                          |
| `lang`            | `"en"`                      | syndication 言語と表示日付。                        |
| `timeout`         | `10000`                     | メタデータ要求のタイムアウト（ミリ秒）。            |
| `cache`           | `true`                      | メモリと永続 JSON キャッシュ。                      |
| `cacheDir`        | `.cache/ox-content/twitter` | 永続メタデータキャッシュディレクトリ。              |
| `mediaOutputDir`  | `public/ox-content/twitter` | アバターと写真のローカルディレクトリ。              |
| `mediaPublicPath` | `/ox-content/twitter`       | ダウンロードしたメディアに出す URL プレフィックス。 |

ダウンロードしたメディアは自分のサイトから出すので、厳しい `img-src 'self'` CSP も動き続けます。削除済みや非公開の投稿は、ビルドを落とさずリンクのみのカードに落ちます。詳細は [Twitter/X Embed](/examples/twitter-embed.md) を見てください。

## Bluesky

`embeds.bluesky` は静的カードを描画します。カードに出すテキストは要素本文なので、ネットワーク要求は一切要りません。

```md
<Bluesky url="https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l">
  👋 Bluesky is an open social network
</Bluesky>
```

<Bluesky url="https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l">
  👋 Bluesky is an open social network
</Bluesky>

## Spotify

`embeds.spotify` はトラック、アルバム、プレイリスト、エピソード、番組、アーティスト向けの公式 iframe プレーヤーを描画します。

```md
<Spotify url="https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC" />
```

<Spotify url="https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC" />

出力は遅延読み込み付きで `open.spotify.com/embed/...` を指す `<iframe>` です。上の静的カードと違い、本物の第三者プレーヤーなのでオプトインのままです。

## StackBlitz

`embeds.stackBlitz` は StackBlitz プロジェクト URL を、`embed=1` を付けたサンドボックス iframe にします。

```md
<StackBlitz url="https://stackblitz.com/edit/vitejs-vite-abc123"></StackBlitz>
```

## WebContainer

`embeds.webContainer` は、操作時に [WebContainers](https://webcontainers.io/) を起動するサイト向けに、プロジェクトソースとクロスオリジン分離メタデータを持つ遅延プレースホルダを出します。プレースホルダ自体は完全に静的です。

```md
<WebContainer entry="index.html" title="Demo">
  npm install
  npm run dev
</WebContainer>
```

<WebContainer entry="index.html" title="Demo">
  npm install
  npm run dev
</WebContainer>

分離要件は [WebContainer Embed](/examples/webcontainer-embed.md) を見てください。

## 関連

- [Mermaid](./mermaid.md) — 静的 SVG に描画する図フェンス。
- [組み込み機能の一覧](../built-in-features.md)
