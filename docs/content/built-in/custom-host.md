---
title: Custom host lifecycle
description: Let Ox Content own Vite loading, development routing, cache invalidation, and coordinated output writing for a custom HTML host.
---

# Custom host lifecycle

Use `oxContentCustomHost()` when the site owns layout and publication policy,
but Ox Content should own the Vite lifecycle.

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContentCustomHost } from "@ox-content/vite-plugin";

export default defineConfig({
  appType: "custom",
  plugins: [
    oxContentCustomHost({
      host: "./src/site-host.ts",
      oxContent: {
        srcDir: "content",
        redirects: { provider: "netlify" },
        ssg: {
          markdownSource: true,
          siteUrl: "https://example.com",
          siteName: "Example",
        },
      },
      themeTokens: {
        theme: colorScheme,
        include: (name) => name.startsWith("syntax-"),
      },
    }),
  ],
});
```

The factory registers `oxContent({ ssg: { enabled: false } })` and the custom
host plugin together. A lower-level `createOxContentCustomHostPlugin()` is also
available for hosts that already install `oxContent()` themselves.

## Host module

The host module exports routes. Routes render ordinary `Response` objects or a
plain object with `html`, `text`, `contentType`, metadata, and dependencies.

```ts
// src/site-host.ts
export default {
  routes: [
    {
      path: "/",
      inputPath: "content/index.md",
      source: "# Home\n",
      aliases: ["/old-home"],
      dependencies: ["src/data.ts"],
      async render(ctx) {
        const data = await ctx.loadModule("/src/data.ts");
        const assets = ctx.assets.document({
          head: "<title>" + data.title + "</title>",
          sharedStyles: [ctx.assets.themeTokens?.href].filter(Boolean),
          clientEntries: ["src/main.ts"],
          crossorigin: true,
        });

        return {
          html: `<!doctype html><html><head>${assets.headHtml}</head><body>${data.html}</body></html>`,
          title: data.title,
        };
      },
    },
    {
      path: "/feed.xml",
      render: () =>
        new Response("<feed />", {
          headers: { "content-type": "application/xml; charset=utf-8" },
        }),
    },
  ],
  notFound() {
    return { text: "Not Found", status: 404, contentType: "text/plain" };
  },
};
```

In development, Ox Content SSR-loads the host through Vite, dispatches matching
routes, preserves status and content type, applies `transformIndexHtml()` only
to HTML, and falls through when no route or custom 404 handles the request.
Route responses are cached as promises. Declared dependencies invalidate only
the affected responses, reloads are debounced, failed renders retry, and an old
in-flight render cannot delete a newer cache entry.

In production, the plugin runs once from `closeBundle`, after Vite has emitted
client assets and `.vite/manifest.json`. It opens a temporary middleware-mode
Vite server only to SSR-load the host and site modules, passes `ctx.loadModule`
instead of the raw server, and closes the temporary server in `finally`.

## Coordinated outputs

Host-rendered HTML routes are connected to the same public output writers as
the default SSG:

- `writeResourceFiles()` for resource fingerprinting and rewritten HTML.
- `writeSelfHostedAssets()` for fonts and Iconify CSS.
- `writeMarkdownCompanions()` from route `source`.
- `writeRedirectOutputs()` from route `aliases` / `redirect`.
- `writeFeedFiles()` and `writeSiteMapFiles()` from selected route metadata.

Duplicate route output paths fail the build with the conflicting owners. The
host still owns publication selection; Ox Content only writes the routes the
host returns.

## Theme token stylesheet

`themeTokens` writes and serves a small stylesheet, defaulting to
`/__ox_theme_tokens__/theme-tokens.css`. Include `ctx.assets.themeTokens.href`
in `ctx.assets.document()` to reuse syntax token CSS without a local Vite
transform.

## Related

- [Document assets](./document-assets.md)
- [SSG output primitives](./ssg-output.md)
- [Theme](../theming.md)
- Tracking: [#1281](https://github.com/ubugeeei-prod/ox-content/issues/1281)
