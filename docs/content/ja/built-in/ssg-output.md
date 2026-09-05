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
  rewriteCollectionAssetUrls,
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

const rewritten = rewriteCollectionAssetUrls({
  html: hostRenderedHtml,
  pagePath: "/works/showcase/",
  manifest: collectionAssets,
});
```

`writeCollectionAssets()` は各 content hash を `contentDir`（既定値は
`"/assets/content"`）配下に一度だけ書き、alias は hard-link、非対応の環境では
copy で生成します。`sourcePath` は `root` 配下でなければならず、壊れた URL
encoding、path traversal、出力先外への alias は拒否されます。

`rewriteCollectionAssetUrls()` は独自 renderer 向けの純粋な HTML 処理です。
既定では HTML fragment を parse し、`href`、`src`、`poster` を指定した
page path から解決して、既知の alias を manifest の content-addressed target
へ書き換えます。query string と fragment は残します。未知の alias、外部
origin、fragment-only link、`data:`、`mailto:`、`javascript:` のような
非 HTTP scheme、壊れた属性値はそのままです。`origin` を渡すと same-origin の
absolute URL は root-relative path と同じように扱います。書き換え後の値は URL
path として出力されます。入力が HTML document 全体なら `document: true` を渡します。

## 独自ホストで外部フィードを読む

組み込み blog renderer と独自 blog index は同じ RSS / Atom loader を共有できます。
`loadBlogFeedEntries()` は公開 blog feed 設定を受け取り、既存の timeout、
redirect、size、安全な network policy を保ったまま、正規化した外部 entry と
warning/fatal diagnostics を返します。

```ts
import { loadBlogFeedEntries, mergeBlogFeedEntries } from "@ox-content/vite-plugin";

const external = await loadBlogFeedEntries({
  sources: [
    { url: "https://example.com/feed.xml", language: "en", author: "Ada" },
    { url: "https://example.jp/atom.xml", language: "ja", onError: "warn" },
  ],
});

if (external.fatals.length) {
  throw new Error(external.fatals.join("\n"));
}

const entries = mergeBlogFeedEntries(localEntries, external.entries);
```

entry は `title`、`url`、安定した `id`、任意の `canonical`、`date`、
`language`、`author`、`summary`、`external`、`sourceUrl` を持ちます。source が
空なら network request は発生しません。同じ source URL は 1 call あたり 1 回だけ
fetch し、warn の失敗は他の成功 source を捨てません。merge は組み込み blog と
同じで、canonical URL または安定 id が重複したときは local entry が勝ち、
そのあと新しい順に並びます。

## 組み込みページなしのリダイレクト出力

独自ホストは、組み込み SSG と同じ設定で redirect output を計画し、明示的に
書き出せます。計画は filesystem に触らず、書き出しは選択された redirect file
だけを出し、既存の host-rendered HTML page は置き換えません。

```ts
import { planRedirectOutputs, writeRedirectOutputs } from "@ox-content/vite-plugin";

const redirectInput = {
  redirects: {
    provider: "cloudflare",
    html: false,
    map: { "/old-guide": "/guide" },
  },
  routes: [{ path: "/guide", aliases: ["/old"], redirect: "/retired" }],
  occupiedPaths: hostPagePaths,
  base: "/docs/",
} as const;

const redirectPlan = planRedirectOutputs(redirectInput);
await writeRedirectOutputs({ ...redirectInput, outDir });
```

`planRedirectOutputs()` は公開 `redirects` option、独自 host route、host が占有する
path、`base`、任意の CI env を受け取ります。出力は `html`、`provider`、
`headers`、`json` の discriminated union で、provider 出力は内部 file-plan field
ではなく `cloudflare` または `netlify` を示します。`_redirects`、`_headers`、
`redirects.json` は host root のファイルです。Ox Content はこの API または
組み込み SSG で求められたときだけ書くので、独自ホストが同じファイルを既に
持つ場合は、書く前に merge するか owner を 1 つにしてください。

## API

| 関数                               | 役割                                                                                    |
| ---------------------------------- | --------------------------------------------------------------------------------------- |
| `planSsgOutputs`                   | ホストのページと `buildSsg()` と同じオプションから writer 入力を作る。                  |
| `writeResourceFiles`               | ページバンドル資産に指紋を付け、ホスト HTML の URL を書き換える。                       |
| `writeSelfHostedAssets`            | 独自ホスト向けに `__ox_icons__` と `__ox_fonts__` のファイルを書く。                    |
| `planCollectionAssets`             | 明示的な collection source-to-public mapping から content-addressed target を計画する。 |
| `writeCollectionAssets`            | 重複排除した collection target と public hard-link/copy alias を書く。                  |
| `createCollectionAssetsMiddleware` | development host で計画済み alias と content target を配信する。                        |
| `rewriteCollectionAssetUrls`       | ホスト描画 HTML の alias を collection content target へ書き換える。                    |
| `loadBlogFeedEntries`              | page rendering なしで、独自 blog index 向け RSS / Atom entry を読む。                   |
| `mergeBlogFeedEntries`             | host entry と外部 entry を組み込み blog と同じ優先順位で merge する。                   |
| `planRedirectOutputs`              | redirect HTML、provider、headers、JSON 出力を filesystem に触らず preview する。        |
| `writeRedirectOutputs`             | host-rendered page の横に選択した redirect output を書く。                              |
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

- [独自ホスト lifecycle](./custom-host.md)
- [Document assets](./document-assets.md)
- [ページリソース](./resources.md)
- [Markdown ソースの併記](./markdown-source.md)
- [RSS / Atom / JSON フィード](./feeds.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [ページ head](./page-head.md)
- [サイト生成](./site-generation.md)
- 追跡: [#878](https://github.com/ubugeeei-prod/ox-content/issues/878)
