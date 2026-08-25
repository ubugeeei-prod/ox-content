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

| オプション     | 既定                         | 目的                                                     |
| -------------- | ---------------------------- | -------------------------------------------------------- |
| `timeout`      | `10000`                      | 取得タイムアウト（ミリ秒）。                             |
| `cache`        | `true`                       | 取得したメタデータを現在のプロセスのメモリにキャッシュ。 |
| `cacheTTL`     | `3600000`                    | 鮮度の窓（ミリ秒）。                                     |
| `persistCache` | `false`                      | 成功・失敗エントリをビルド間でディスクに残す。           |
| `cacheDir`     | `.cache/ox-content/ogp`      | 永続メタデータキャッシュディレクトリ。                   |
| `refresh`      | `false`                      | 新しいキャッシュがあっても再取得する。                   |
| `userAgent`    | `ox-content-ogp-bot/1.0 ...` | 対象へ送る User-Agent。                                  |

`persistCache: true` にすると、クリーンビルドや CI ワーカーでもメタデータを再利用できます。成功した取得と届かなかった URL は、正規化した URL ごとに 1 つの JSON として `cacheDir` へ保存します。新しいエントリはネットワークを飛ばし、期限切れと `refresh: true` は再取得してファイルをアトミックに置き換えます。壊れたファイルは無視するので、後続ビルドを汚染しません。変換に渡したメタデータは、キャッシュや取得より優先されます。

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
<pm>npm install -D @ox-content/vite-plugin @ox-content/theme-swiss</pm>
```

<pm>npm install -D @ox-content/vite-plugin @ox-content/theme-swiss</pm>

コマンド変換は Rust ネイティブです。`npm install -D` は `pnpm add -D`、`yarn add -D`、`bun add -D`、`vp install -D` になり、`npx <bin>` は `vp exec -- <bin>` になります。タブはクライアント側 JavaScript なしで動きます。選択は CSS `:has()` です。`pm: { sync: true }` をオプトインすると、ページ上のすべてのブロックで選んだパッケージマネージャを `localStorage` 経由で同期します。変換表の全体は [Package Manager Tabs](/examples/package-manager-tabs.md) を見てください。

## タブ

汎用タブグループはパッケージマネージャタブと同じウィジェットで、SSG ビルドと dev preview では常に使えます。

```md
<tabs>
<tab label="Install">
<pre><code>pnpm add -D @ox-content/vite-plugin
pnpm add -D @ox-content/theme-swiss</code></pre>
</tab>
<tab label="Config">
<pre><code>oxContent({ srcDir: "content", embeds: { pm: true } })</code></pre>
</tab>
<tab label="Markdown">
<pre><code>---
title: Install
---

好きなパッケージマネージャで Ox Content を入れます。

&lt;pm&gt;npm install -D @ox-content/vite-plugin&lt;/pm&gt;</code></pre>
</tab>
<tab label="Build">
<pre><code>pnpm vite build
pnpm vite preview</code></pre>
</tab>
</tabs>
```

<tabs>
<tab label="Install">
<pre><code>pnpm add -D @ox-content/vite-plugin
pnpm add -D @ox-content/theme-swiss</code></pre>
</tab>
<tab label="Config">
<pre><code>oxContent({ srcDir: "content", embeds: { pm: true } })</code></pre>
</tab>
<tab label="Markdown">
<pre><code>---
title: Install
---

好きなパッケージマネージャで Ox Content を入れます。

&lt;pm&gt;npm install -D @ox-content/vite-plugin&lt;/pm&gt;</code></pre>
</tab>
<tab label="Build">
<pre><code>pnpm vite build
pnpm vite preview</code></pre>
</tab>
</tabs>

`label` 属性のない `<tab>` は `Tab 1`、`Tab 2` のように落ちます。

## YouTube

YouTube 埋め込みは SSG ビルドと dev preview で常に処理されます。iframe はプライバシー強化モード（`youtube-nocookie.com`）と遅延読み込みが既定です。

```md
<youtube id="aqz-KE-bpKQ" title="Big Buck Bunny" />
```

<youtube id="aqz-KE-bpKQ" title="Big Buck Bunny" />

`id`、`url`、`href` 属性を受け付けます。`youtu.be`、`watch?v=`、`shorts`、`embed` の URL 形はどれも認識します。`start` は非負整数の秒で、iframe URL に `?start=` を付けます。不正、負、小数、オーバーフロー、重複した値は無視します。`start` を省略すると、これまでの URL のままです。

```md
<youtube id="aqz-KE-bpKQ" title="Big Buck Bunny" start="4190" />
```

## Twitter / X

`embeds.twitter` は投稿を静的カードとして描画し、第三者ウィジェットのスクリプトは決して読みません。`twitter: true` のとき、埋め込みはプライバシーを意識したリンクカードです。

```md
<XPost url="https://x.com/jack/status/20" />
```

<XPost url="https://x.com/jack/status/20" />

オブジェクト形式を使うと、ビルド時に本文、著者、アバター、写真、動画ポスターを取り、自分のオリジンから配信します。取ってきたカードには、syndication にそのメタデータがあるとき、引用投稿の入れ子カードと「Replying to @…」リンクも含まれます。`appearance: "full"` は sveltweet / react-tweet 形の静的カードです。既定の `"compact"` カードは変わりません。

```ts
oxContent({
  embeds: {
    twitter: {
      fetch: true,
      lang: "en",
      appearance: "compact",
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
| `mediaOutputDir`  | `public/ox-content/twitter` | アバター、写真、動画のローカルディレクトリ。        |
| `mediaPublicPath` | `/ox-content/twitter`       | ダウンロードしたメディアに出す URL プレフィックス。 |
| `downloadVideo`   | `false`                     | ビルド時に MP4 動画とアニメーション GIF を取る。    |
| `maxVideoBytes`   | `8388608`                   | これより大きい動画はスキップする（8 MiB）。         |
| `appearance`      | `"compact"`                 | `"full"` で sveltweet 形の静的クロムを出す。        |

ダウンロードしたメディアは自分のサイトから出すので、厳しい `img-src 'self'` CSP も動き続けます。動画とアニメーション GIF は、`downloadVideo` をオンにしない限り自前のポスターと Watch on X パーマリンクを使い、生成 HTML に `video.twimg.com` は出しません。削除済みや非公開の投稿は、ビルドを落とさずリンクのみのカードに落ちます。引用投稿が欠けていても、元の投稿カードは残します。フルカード用 CSS は `.ox-tweet--full` を描画するページにだけ載ります。フルカードのクロムは MIT ライセンスの [react-tweet](https://github.com/vercel/react-tweet) と [sveltweet](https://github.com/ryoppippi/sveltweet) の見た目の契約に従います。帰属は [クレジット](../credits.md) にあります。詳細は [Twitter/X Embed](/examples/twitter-embed.md) を見てください。

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
<StackBlitz url="https://stackblitz.com/edit/vitejs-vite" />
```

<StackBlitz url="https://stackblitz.com/edit/vitejs-vite" />

## WebContainer

`embeds.webContainer` は、操作時に [WebContainers](https://webcontainers.io/) を起動するサイト向けに、プロジェクトソースとクロスオリジン分離メタデータを持つ遅延プレースホルダを出します。プレースホルダ自体は完全に静的です。

<!-- prettier-ignore -->
```mdx
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
