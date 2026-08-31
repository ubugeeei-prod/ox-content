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

## API

| Function                  | Role                                                                                    |
| ------------------------- | --------------------------------------------------------------------------------------- |
| `planSsgOutputs`          | Build writer inputs from host pages and the same option objects `buildSsg()` reads.     |
| `writeResourceFiles`      | Fingerprint page-bundle assets and rewrite host HTML URLs.                              |
| `writeSelfHostedAssets`   | Write self-hosted `__ox_icons__` and `__ox_fonts__` files for a custom host.            |
| `writeMarkdownCompanions` | Write original Markdown beside host-rendered pages. Reuses the copy-as-markdown writer. |
| `renderFeedFiles`         | Render RSS / Atom / JSON feed files without filesystem writes.                          |
| `writeFeedFiles`          | Write RSS / Atom / JSON feeds, including [named feeds](./feeds.md).                     |
| `writeSiteMapFiles`       | Write `sitemap.xml`, `robots.txt`, and `llms.txt`.                                      |
| `resolveGitLastmod`       | Return a file's latest git commit time in milliseconds, or `undefined`.                 |

`lastUpdated` on a page is used as-is. When it is omitted and `ssg.lastUpdated`
or `siteMaps` is on, the planner calls `resolveGitLastmod(inputPath, root)`.

Hosts can skip the planner and call a writer with the same resolved option
objects `buildSsg()` already uses (`resolveResourcesOptions`,
`resolveFeedsOptions`, `resolveSiteMapsOptions`,
`resolveMarkdownSourceOptions`). Use `resolveSelfHostedAssetManifest()` when a
custom renderer needs the matching stylesheet and preload tags for `<head>`.

## Related

- [Page resources](./resources.md)
- [Markdown source companions](./markdown-source.md)
- [RSS / Atom / JSON feeds](./feeds.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Page head](./page-head.md)
- [Site Generation](./site-generation.md)
- Tracking: [#878](https://github.com/ubugeeei-prod/ox-content/issues/878)
