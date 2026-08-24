---
title: 埋め込み
description: GitHub / OG カード、パッケージマネージャタブ、メディア、SNS。
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

タブと YouTube は `embeds` オプションの外です。SSG と dev preview では常に処理されます。

すべて切るときは `embeds: false`、個別に足すときはオブジェクトです。

```ts
oxContent({
  embeds: {
    github: { maxSourceLines: 120 },
    openGraph: { timeout: 5000 },
    pm: { sync: true },
    twitter: true,
    bluesky: true,
  },
});
```

## 関連

- [英語版ガイド](/built-in/embeds.md)
