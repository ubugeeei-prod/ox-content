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
import { oxContentCustomHost, planCollectionAssets } from "@ox-content/vite-plugin";

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
      collectionAssets: {
        manifest: (ctx) =>
          planCollectionAssets({
            root: ctx.root,
            assets: [
              {
                sourcePath: "content/projects/cover.jpg",
                publicPath: "/projects/cover.jpg",
              },
            ],
          }),
        watch: [{ path: "content/projects", kind: "directory" }],
        ownedPrefixes: ["/assets/content"],
      },
      dev: {
        routeDependencies: [{ path: "content/projects", kind: "directory" }],
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
  async outputs(ctx) {
    const content = await ctx.memo("content", () => ctx.loadModule("/src/content.ts"));
    return {
      siteDescription: content.siteDescription,
      collectionNames: ["blog", "media"],
      collections: {
        blog: content.blogFeedItems,
        media: content.mediaFeedItems,
      },
    };
  },
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
in-flight render cannot delete a newer cache entry. Requests carrying
`Cookie` or `Authorization` are rendered without storing their response, so
request identities are not shared through the development cache.
`dependencies` accepts file paths or `{ path, kind }` descriptors where `kind`
is `"file"`, `"directory"`, or `"glob"`. `dev.dependencies` invalidates every
response, while
`dev.routeDependencies` also clears and reloads the route catalogue.
Modules loaded through `ctx.loadModule()` and CSS returned by
`ctx.assets.stylesheets()` are tracked automatically in development.
Set `dev.feedOutputs: true` to serve configured RSS, Atom, and JSON feed files
from `outputs(ctx)` in development. Explicit host routes win when they use the
same path. Missing feed paths fall through to the host `notFound()` handler or
Vite, and failed feed renders are not cached so the next matching request can
retry after the source or module error is fixed.

In production, the plugin runs once from `closeBundle`, after Vite has emitted
client assets and `.vite/manifest.json`. It opens a temporary middleware-mode
Vite server only to SSR-load the host and site modules, passes `ctx.loadModule`
instead of the raw server, and closes the temporary server in `finally`.
`outputs(ctx)` is called only when `feeds` is enabled. It is build-only unless
the development server opts into `dev.feedOutputs`.

`ctx.memo(key, load)` shares one expensive route-catalogue load inside a build
or development route-catalogue pass. Use the same key from `routes(ctx)` and
`outputs(ctx)` to reuse a host-owned content artifact without module-global
mutable state. Rejected loaders are evicted so the next call can retry.

## Markdown renderer

Every custom-host context exposes `ctx.markdown.render()` for hosts that read
Markdown or MDX source themselves but want Ox Content to own the configured
rendering pipeline. Pass the raw `source` and the real `documentPath`.

```ts
const page = await ctx.markdown.render<{
  clientModules: Array<{ moduleId: string }>;
}>({
  source,
  documentPath,
  async renderHtml(markdown) {
    const solid = await markdown.loadModule("/src/solid-renderer.ts");
    return solid.render(markdown.html, {
      documentPath: markdown.documentPath,
      imports: markdown.transform.imports,
      root: markdown.root,
      srcDir: markdown.srcDir,
      contentRoot: markdown.contentRoot,
    });
  },
});

const styles = ctx.assets.stylesheets({
  modules: page.metadata?.clientModules.map((module) => module.moduleId) ?? [],
});
```

The supported order is:

1. Markdown and MDX syntax transforms from the resolved Ox Content options.
2. Built-in HTML/embed transforms, with project-relative provider cache and
   media output paths resolved from the Vite root.
3. Legacy `<Island>` HTML transform.
4. Optional `renderHtml(markdown)` framework integration.
5. Reader-chrome HTML transformation from `oxContent.ssg.readerChrome`, unless
   the call overrides `readerChrome`.

The result includes final `html`, the pre-framework `markdownHtml`, the full
`TransformResult`, flattened `frontmatter`, `toc`, MDX `imports`, MDX `exports`,
`components`, optional framework `metadata`, and `dependencies`. The input
document path is included in `dependencies`; merge the returned dependencies
into the route result so development responses invalidate when the source
changes. Modules loaded with `markdown.loadModule()` and stylesheets resolved
inside the renderer callback are tracked by the same development lifecycle as
`ctx.loadModule()` and `ctx.assets.stylesheets()`, so a failed or changed
renderer can recover without a process-global cache.

Inside `renderHtml(markdown)`, `markdown.parseHtml()` returns the callback's
input HTML as a HAST fragment. Parsing happens on the first call, and repeated
calls reuse the same mutable tree for that render. The tree omits source
positions, which avoids tracking offsets in generated HTML when an adapter only
needs its elements, attributes, and text. Return serialized HTML from the
callback to apply tree edits to the output. Each render receives its own tree.

The framework integration remains host-owned. The framework-neutral island
contract is available from `@ox-content/vite-plugin/html-host` for adapters that
own server rendering, head output, slots, browser hydration, mounting, and
disposal. Solid and Svelte custom hosts can keep using
`createSolidHtmlHostRenderer()` or `createSvelteHtmlHostRenderer()`; those
helpers wrap the shared contract and return the rendered `html`, `headHtml`,
`clientModules`, and diagnostics metadata the host needs for assets and
hydration. `renderHtmlHostMarkdown()` from
`@ox-content/vite-plugin/html-host` can be used inside `renderHtml(markdown)`
when the host wants the shared Markdown HTML, island metadata, and document
asset composition in one step. The host still owns route selection, publication
rules, layout, URL policy, and global styles.

Island head output is appended in document island order before client entries
when the returned `documentAssets.headHtml` is placed in the document `<head>`.
Exact serialized head fragments are emitted once, so two islands that return the
same scoped `<style>` do not duplicate it; semantic conflicts such as two
different `<title>` values remain host/framework policy.

## Coordinated outputs

Host-rendered HTML routes are connected to the same public output writers as
the default SSG:

- `writeResourceFiles()` for resource fingerprinting and rewritten HTML.
- `writeSelfHostedAssets()` for fonts and Iconify CSS.
- `writeCollectionAssets()` from `collectionAssets.manifest`.
- `writeMarkdownCompanions()` from route `source`.
- `writeRedirectOutputs()` from route `aliases` / `redirect`.
- `writeFeedFiles()` and `writeSiteMapFiles()` from selected route metadata.

Route metadata belongs on the route or render result: `title`, `description`,
`source`, `aliases`, `redirect`, `lastUpdatedPaths`, and publish-state fields
describe that HTML page. Site-output data that is not itself a page belongs in
`outputs(ctx)`: programmatic default `items`, named feed `collections`,
`collectionNames`, and `siteDescription`. Ox Content passes that data through
`planSsgOutputs()` and the existing feed writers, so publish-state filtering,
validation, output paths, and diagnostics stay shared with the built-in SSG.
With `dev.feedOutputs: true`, the same feed channels and `outputs(ctx)` data
are rendered on demand without writing files. The development server still does
not render every page to serve one feed; hosts should return feed collections or
programmatic `items` from `outputs(ctx)` for feed-only data.

Set `ssg.minifyHtml: true` for built-in SSG pages or
`build.minifyHtml: true` on the custom-host plugin to minify production HTML.
When the custom-host build option is omitted, it follows `oxContent.ssg.minifyHtml`.
Minification runs after `transformIndexHtml()` and resource URL rewriting,
immediately before HTML files are written. It does not transform XML feeds,
JSON feeds, Markdown companions, text outputs, or binary assets.

Use `lastUpdatedPaths` on a route or render result when sitemap freshness should
consider shared metadata files or source directories in addition to `inputPath`.
It is only used for git lastmod; dev invalidation still uses explicit
`dependencies`.

Duplicate route output paths fail the build with the conflicting owners. The
host still owns publication selection; Ox Content only writes the routes the
host returns.

## Collection assets

Use `collectionAssets` when the host owns files attached to collections, such
as project images or downloadable artifacts, and wants Ox Content to serve and
write them with the same manifest.

```ts
oxContentCustomHost({
  host: "./src/site-host.ts",
  collectionAssets: {
    manifest: async (ctx) =>
      planCollectionAssets({
        root: ctx.root,
        assets: [
          {
            sourcePath: "content/projects/cover.jpg",
            publicPath: "/projects/cover.jpg",
          },
        ],
      }),
    watch: [{ path: "content/projects", kind: "directory" }],
    ownedPrefixes: ["/assets/content"],
  },
});
```

In development, the middleware serves aliases and content-addressed targets
before route rendering. A missing URL under an owned prefix returns 404 instead
of falling through to the host router. When watched files or manifest source
files change, Ox Content re-plans the manifest, clears cached route responses,
and reloads after a successful plan. A failed re-plan is retryable and does not
replace the last successful manifest. In production, collection assets are
written with the coordinated output files unless `collectionAssets.write` is
`false`.

Route, render, and output hooks can read the same configured snapshot through
`ctx.assets.collectionManifest()`. Use it when the host needs the manifest for
HTML rewriting or structured page-data aliases; the writer and development
middleware reuse that resolved manifest instead of calling
`collectionAssets.manifest` again.

```ts
const manifest = await ctx.assets.collectionManifest();
const body = manifest
  ? rewriteCollectionAssetUrls({
      html,
      pagePath: ctx.url.pathname,
      manifest,
    }).html
  : html;
```

When the host already knows the public Markdown or MDX documents, use
`planCollectionAssetsFromDocuments()` instead of globbing media extensions. It
discovers relative Markdown image/link URLs plus literal HTML and MDX JSX
`href`, `src`, and `poster` attributes from only those selected documents,
resolves files against the real document path, and returns the same
`CollectionAssetManifest`.

```ts
collectionAssets: {
  manifest: async (ctx) => {
    const result = await planCollectionAssetsFromDocuments({
      root: ctx.root,
      contentRoot: "content",
      documents: [{ documentPath: "content/posts/hello.mdx", pagePath: "/posts/hello" }],
      extraAssets: [{ sourcePath: "content/posts/og.png", publicPath: "/og/hello.png" }],
      publicPath: (reference) => [reference.publicPath, `/legacy${reference.publicPath}`],
    });
    if (result.diagnostics.length > 0) {
      throw new Error(result.diagnostics.map((diagnostic) => diagnostic.message).join("\n"));
    }
    return result.manifest;
  },
}
```

External, root-absolute, data/blob, fragment-only, and other non-local URLs are
ignored. Missing files and references outside `contentRoot` are reported as
document-scoped diagnostics, while query strings and fragments stay on the
reported references and are preserved later by `rewriteCollectionAssetUrls()`.

HTML-string hosts can generate their browser island registry from that same
selected route/document set. Use `createHtmlHostIslandRegistry()` from
`@ox-content/vite-plugin/html-host` for a framework-neutral virtual module, or
the Solid and Svelte wrappers when the host wants their stable framework
virtual ids. The generated module contains only the selected island
dynamic-import roots; Vite still keeps their transitive dependencies.

## SSR stylesheets

When route pages or layouts import blocking CSS from server-rendered modules,
configure the small set of SSR stylesheet roots up front and resolve the
route-selected roots during render:

```ts
// vite.config.ts
oxContentCustomHost({
  host: "./src/host.ts",
  ssrStylesheets: {
    modules: ["src/layout.tsx", "src/pages/**/page.tsx"],
  },
});

// host.ts
const routes = [{ path: "/about", pageModule: "/src/pages/about/page.tsx" }];

export default {
  routes: routes.map((route) => ({
    path: route.path,
    async render(ctx) {
      const layout = await ctx.loadModule("/src/layout.tsx");
      const page = await ctx.loadModule(route.pageModule);
      const ssrStyles = ctx.assets.ssrStylesheets({
        modules: ["/src/layout.tsx", route.pageModule],
      });
      const assets = ctx.assets.document({
        pageStyles: ssrStyles.stylesheets,
        clientEntries: ["src/main.ts"],
      });

      return {
        html: `<!doctype html><html><head>${assets.headHtml}</head><body>${layout.render(page)}</body></html>`,
        dependencies: ssrStyles.dependencies,
      };
    },
  })),
};
```

The host supplies module identity explicitly because rendered HTML strings do
not carry their source module ids. Route descriptors can keep page module ids
next to `path`, `inputPath`, and other route metadata; pass only the selected
layout/page roots for the current route. Do not derive a page's styles from a
shared route catalogue import, because that would include sibling routes loaded
only to build the catalogue.

In development, call `ctx.loadModule()` for each selected SSR root before
`ctx.assets.ssrStylesheets()`. The resolver then uses Vite's module graph to
return blocking stylesheet hrefs plus dependency files that invalidate the
cached route. `ssrStylesheets.modules` is also watched, so new or deleted page
module files matching a configured glob can refresh the route catalogue.

In production builds, the plugin statically walks configured local JavaScript
and TypeScript SSR roots, follows local static imports, and writes hashed
custom-host CSS artifacts for directly imported plain CSS files. External
package JavaScript is not traversed. Local dynamic imports, missing roots,
unresolved local imports, and CSS resolved outside the Vite root are reported
as diagnostics instead of being silently dropped. Ordering follows the source
import order and then the route's `modules` order; pass the returned styles
into `ctx.assets.document()` so overlap with other returned stylesheet
descriptors is deduplicated by the document asset API. Preprocessed stylesheets
that need Vite plugin transforms should stay on explicit browser/CSS entries
and use `ctx.assets.stylesheets()`.

## Island stylesheets

When a rendered route knows the browser module ids used by SSR-visible islands,
resolve their blocking CSS through `ctx.assets.stylesheets()`.

```ts
const islandStyles = ctx.assets.stylesheets({
  modules: rendered.clientModules.map((module) => module.moduleId),
});

const assets = ctx.assets.document({
  islandStyles: islandStyles.stylesheets,
  clientEntries: ["src/main.ts"],
});

return {
  html: `<!doctype html><html><head>${assets.headHtml}</head><body>${rendered.html}</body></html>`,
  dependencies: islandStyles.dependencies,
};
```

In development, after the host renders islands through `ctx.loadModule()`, the
resolver walks the internal Vite module graph for each rendered browser module
id, includes direct and transitive CSS in dependency order, preserves CSS query
strings, and returns source file dependencies that can invalidate the cached
route. In build, it reads the Vite manifest and returns the same module
identities with emitted hashed stylesheet hrefs. Missing entries are reported
as diagnostics instead of silently dropping styles. Pass the returned styles
into `ctx.assets.document()` so document-level dedupe, nonce, base, shared CSS,
and page CSS composition all stay in one place.

For Markdown-first HTML-string hosts, the shared helper wires the same pieces
through the custom-host Markdown lifecycle:

```ts
const page = await ctx.markdown.render({
  source,
  documentPath: "content/posts/hello.mdx",
  renderHtml(markdown) {
    return renderHtmlHostMarkdown({
      context: markdown,
      renderIslands,
      documentAssets: { clientEntries: ["src/client.ts"] },
    });
  },
});

return {
  html: `<!doctype html><html><head>${page.metadata?.documentAssets?.headHtml ?? ""}</head><body>${page.html}</body></html>`,
  dependencies: page.dependencies,
};
```

Build hosts that deliberately inline critical CSS can read the emitted bytes for
the same resolved descriptors without deriving filesystem paths from public
hrefs:

```ts
const islandStyles = ctx.assets.stylesheets({
  modules: rendered.clientModules.map((module) => module.moduleId),
});
const critical = await ctx.assets.stylesheetContent({
  stylesheets: islandStyles.stylesheets,
});

const assets = ctx.assets.document({
  inlineStyles: critical.stylesheets.map((style) => ({
    key: `critical:${style.href}`,
    content: style.content,
  })),
});
```

`stylesheetContent()` is build-only for Vite-emitted stylesheets. In
development it returns `unavailable` diagnostics, and in build it reports
`missing-artifact` diagnostics when a descriptor has no recorded artifact or the
file cannot be read. Ordering follows the input descriptors, so hosts can keep
their own policy for home-only inlining, linked stylesheets, and dedupe.

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
