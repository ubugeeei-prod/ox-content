---
title: SSG output primitives
description: Plan and emit resources, Markdown companions, feeds, sitemaps, and git lastmod without the default theme.
---

# SSG output primitives

Custom hosts that set `ssg: false` keep their own page templates. They can
still ask Ox Content to plan and emit:

- content-addressed resource fingerprinting and URL rewriting
- self-hosted font and Iconify asset files
- Markdown companion files for host-rendered HTML pages
- RSS / Atom / JSON feeds and sitemap metadata
- git-derived `lastmod`

None of this requires the default theme or `buildSsg()`. The same option
objects used by `oxContent()` / `buildSsg()` configure the composable path.

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

`ssg: false` (the boolean) turns SSG off and also clears `markdownSource`,
`lastUpdated`, and `siteUrl`. Use `ssg: { enabled: false, ... }` when those
fields should still resolve.

## Collection assets with custom aliases

`planCollectionAssets()` handles files that do not belong to a Markdown page's
resource flow. Give it explicit collection source paths and the public aliases
your host owns. It hashes and deduplicates the content target, URL-encodes
aliases safely, and keeps the same manifest for production and development.

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

`writeCollectionAssets()` writes each distinct content hash below `contentDir`
(`"/assets/content"` by default) once, then hard-links aliases with a copy
fallback. `sourcePath` must stay beneath `root`; malformed URL encoding, path
traversal, and aliases outside the output directory are rejected.

`rewriteCollectionAssetUrls()` is the pure HTML step for custom renderers. It
parses an HTML fragment by default, resolves `href`, `src`, and `poster`
against the supplied page path, and rewrites known aliases to the manifest's
content-addressed target. Query strings and fragments are kept. Unknown aliases,
external origins, fragment-only links, non-HTTP schemes such as `data:`,
`mailto:`, and `javascript:`, and malformed attributes are left unchanged. Pass
`origin` to treat same-origin absolute URLs like root-relative paths; rewritten
values are still emitted as URL paths. Pass `document: true` when the input is a
full HTML document.

## External feeds in a custom host

The built-in blog renderer and a custom blog index can share the same RSS/Atom
loader without building pages. `loadBlogFeedEntries()` accepts the public blog
feed configuration, keeps the existing timeout, redirect, size, and safe-network
checks, and returns normalized external entries plus warning/fatal diagnostics.

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

Entries include `title`, `url`, stable `id`, optional `canonical`, `date`,
`language`, `author`, `summary`, `external`, and `sourceUrl`. Empty sources make
no network request, repeated source URLs are fetched once per call, and warning
sources do not discard unrelated successful sources. Merge order matches the
built-in blog: local entries win on duplicate canonical URL or stable id, then
items sort newest first.

## Redirect outputs without built-in pages

Custom hosts can plan and explicitly write redirect outputs with the same
configuration the built-in SSG uses. Planning is side-effect free; writing emits
only the selected redirect files and will not replace an existing host-rendered
HTML page.

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

`planRedirectOutputs()` accepts the public `redirects` option, custom host
routes, occupied host paths, `base`, and optional CI env. Outputs are
discriminated as `html`, `provider`, `headers`, or `json`; provider outputs name
`cloudflare` or `netlify` instead of exposing the internal file-plan field.
`_redirects`, `_headers`, and `redirects.json` are root host files. Ox Content
writes them only when requested by this API or the built-in SSG, so a custom host
that already owns those files should merge or choose one owner before writing.

## API

| Function                           | Role                                                                                    |
| ---------------------------------- | --------------------------------------------------------------------------------------- |
| `planSsgOutputs`                   | Build writer inputs from host pages and the same option objects `buildSsg()` reads.     |
| `writeResourceFiles`               | Fingerprint page-bundle assets and rewrite host HTML URLs.                              |
| `writeSelfHostedAssets`            | Write self-hosted `__ox_icons__` and `__ox_fonts__` files for a custom host.            |
| `planCollectionAssets`             | Plan content-addressed targets from explicit collection source-to-public mappings.      |
| `writeCollectionAssets`            | Write deduplicated collection targets and public hard-link/copy aliases.                |
| `createCollectionAssetsMiddleware` | Serve the planned aliases and content targets in a development host.                    |
| `rewriteCollectionAssetUrls`       | Rewrite host-rendered HTML aliases to collection content targets.                       |
| `loadBlogFeedEntries`              | Load public RSS/Atom entries for a custom blog index without page rendering.            |
| `mergeBlogFeedEntries`             | Merge host entries and external entries with built-in blog precedence.                  |
| `planRedirectOutputs`              | Preview redirect HTML, provider, headers, and JSON outputs without writing files.       |
| `writeRedirectOutputs`             | Emit selected redirect outputs beside host-rendered pages.                              |
| `writeMarkdownCompanions`          | Write original Markdown beside host-rendered pages. Reuses the copy-as-markdown writer. |
| `renderFeedFiles`                  | Render RSS / Atom / JSON feed files without filesystem writes.                          |
| `writeFeedFiles`                   | Write RSS / Atom / JSON feeds, including [named feeds](./feeds.md).                     |
| `writeSiteMapFiles`                | Write `sitemap.xml`, `robots.txt`, and `llms.txt`.                                      |
| `resolveGitLastmod`                | Return a file's latest git commit time in milliseconds, or `undefined`.                 |

`lastUpdated` on a page is used as-is. When it is omitted and `ssg.lastUpdated`
or `siteMaps` is on, the planner calls `resolveGitLastmod(inputPath, root)`.

Hosts can skip the planner and call a writer with the same resolved option
objects `buildSsg()` already uses (`resolveResourcesOptions`,
`resolveFeedsOptions`, `resolveSiteMapsOptions`,
`resolveMarkdownSourceOptions`). Use `resolveSelfHostedAssetManifest()` when a
custom renderer needs the matching stylesheet and preload tags for `<head>`.

## Related

- [Custom host lifecycle](./custom-host.md)
- [Document assets](./document-assets.md)
- [Page resources](./resources.md)
- [Markdown source companions](./markdown-source.md)
- [RSS / Atom / JSON feeds](./feeds.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Page head](./page-head.md)
- [Site Generation](./site-generation.md)
- Tracking: [#878](https://github.com/ubugeeei-prod/ox-content/issues/878)
