---
title: SSG 出力プリミティブ
description: 既定テーマなしで、リソース・Markdown 併記・フィード・sitemap・git lastmod を計画して書き出す。
---

# SSG 出力プリミティブ

`ssg: false` の独自ホストは、ページテンプレートを自分で持ちます。その場合でも
Ox Content に次の出力の計画と書き出しを任せられます。

- コンテンツアドレスのリソース指紋と URL 書き換え
- セルフホストフォントと Iconify アセットファイル
- ホストが描画した HTML ページ向けの Markdown 併記
- RSS / Atom / JSON フィードと sitemap メタデータ
- git 由来の `lastmod`

既定テーマも `buildSsg()` も不要です。設定オブジェクトは `oxContent()` /
`buildSsg()` と同じものを使います。

```ts
import {
  planSsgOutputs,
  renderFeedFiles,
  writeResourceFiles,
  writeMarkdownCompanions,
  writeFeedFiles,
  writeSiteMapFiles,
  writeSelfHostedAssets,
} from "@ox-content/vite-plugin";

const plan = planSsgOutputs({
  outDir,
  srcDir,
  root,
  options: {
    ssg: {
      enabled: false,
      markdownSource: true,
      lastUpdated: true,
      siteUrl: "https://example.com",
      siteName: "Docs",
    },
    resources: { dedupe: true },
    feeds: true,
    siteMaps: true,
  },
  pages: [
    {
      inputPath: path.join(srcDir, "guide.md"),
      urlPath: "guide",
      outputPath: path.join(outDir, "guide", "index.html"),
      html: hostRenderedHtml,
      source: markdownSource,
      title: "Guide",
    },
  ],
});

await writeResourceFiles(plan.resources);
await writeSelfHostedAssets(plan.selfHostedAssets);
await writeMarkdownCompanions(plan.markdownCompanions);
const feedFiles = await renderFeedFiles(plan.feeds);
await writeFeedFiles(plan.feeds);
await writeSiteMapFiles(plan.siteMaps);
```

boolean の `ssg: false` は SSG を切ると同時に `markdownSource`、
`lastUpdated`、`siteUrl` も消します。これらのフィールドを解決したいときは
`ssg: { enabled: false, ... }` を使います。

## 任意 alias のコレクションアセット

`planCollectionAssets()` は Markdown ページのリソースフローに含まれない
ファイルを扱います。コレクションの source path とホストが持つ public alias を
明示し、production と development で同じ manifest を使えます。content target の
hash 化・重複排除と、alias の安全な URL encoding もこの API が行います。

```ts
import {
  createCollectionAssetsMiddleware,
  planCollectionAssets,
  writeCollectionAssets,
} from "@ox-content/vite-plugin";

const collectionAssets = await planCollectionAssets({
  root,
  assets: [
    {
      sourcePath: "src/content/showcase/project-cover.jpg",
      publicPath: ["/works/showcase/assets/project-cover.jpg", "/works/showcase/cover.jpg"],
    },
  ],
});

await writeCollectionAssets({ manifest: collectionAssets, outDir });
viteServer.middlewares.use(createCollectionAssetsMiddleware(collectionAssets));
```

`writeCollectionAssets()` は各 content hash を `contentDir`（既定値は
`"/assets/content"`）配下に一度だけ書き、alias は hard-link、非対応の環境では
copy で生成します。`sourcePath` は `root` 配下でなければならず、壊れた URL
encoding、path traversal、出力先外への alias は拒否されます。

## API

| 関数                               | 役割                                                                                    |
| ---------------------------------- | --------------------------------------------------------------------------------------- |
| `planSsgOutputs`                   | ホストのページと `buildSsg()` と同じオプションから writer 入力を作る。                  |
| `writeResourceFiles`               | ページバンドル資産に指紋を付け、ホスト HTML の URL を書き換える。                       |
| `writeSelfHostedAssets`            | 独自ホスト向けに `__ox_icons__` と `__ox_fonts__` のファイルを書く。                    |
| `planCollectionAssets`             | 明示的な collection source-to-public mapping から content-addressed target を計画する。 |
| `writeCollectionAssets`            | 重複排除した collection target と public hard-link/copy alias を書く。                  |
| `createCollectionAssetsMiddleware` | development host で計画済み alias と content target を配信する。                        |
| `writeMarkdownCompanions`          | ホスト描画ページの横に元の Markdown を書く。copy-as-markdown の writer を再利用する。   |
| `renderFeedFiles`                  | filesystem に書かずに RSS / Atom / JSON フィードファイルを描画する。                    |
| `writeFeedFiles`                   | RSS / Atom / JSON フィードを書く。[名前付きフィード](./feeds.md) も含む。               |
| `writeSiteMapFiles`                | `sitemap.xml`、`robots.txt`、`llms.txt` を書く。                                        |
| `resolveGitLastmod`                | ファイルの最新 git コミット時刻（ミリ秒）を返す。無ければ `undefined`。                 |

ページに `lastUpdated` があればそれを使います。省略されていて
`ssg.lastUpdated` か `siteMaps` がオンなら、プランナーは
`resolveGitLastmod(inputPath, root)` を呼びます。

プランナーを使わず、`buildSsg()` と同じ解決済みオプション
（`resolveResourcesOptions`、`resolveFeedsOptions`、
`resolveSiteMapsOptions`、`resolveMarkdownSourceOptions`）で writer を
直接呼ぶこともできます。独自 renderer が `<head>` 用の stylesheet / preload
タグを必要とするときは `resolveSelfHostedAssetManifest()` を使います。

## 関連

- [ページリソース](./resources.md)
- [Markdown ソースの併記](./markdown-source.md)
- [RSS / Atom / JSON フィード](./feeds.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [ページ head](./page-head.md)
- [サイト生成](./site-generation.md)
- 追跡: [#878](https://github.com/ubugeeei-prod/ox-content/issues/878)
