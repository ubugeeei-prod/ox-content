---
title: Site Generation
description: Static site generation, OG images, edit links, content collections, API docs, and custom transformers.
---

# Site Generation

Beyond per-page Markdown transforms, the plugin ships the build-level features
a documentation site needs: static HTML generation, per-page Open Graph
images, content collections, and generated API docs.

| Option         | Default              | Purpose                                  |
| -------------- | -------------------- | ---------------------------------------- |
| `ssg`          | `{ enabled: true }`  | Generate static HTML pages during build. |
| `ogImage`      | `false`              | Generate per-page Open Graph images.     |
| `editThisPage` | `false`              | Append "Edit this page" links.           |
| `collections`  | `content` collection | Query Markdown files from client code.   |
| `docs`         | `{ enabled: true }`  | Generate API docs from JSDoc/TSDoc.      |
| `transformers` | `[]`                 | Custom Markdown AST transforms.          |

## Static Site Generation

SSG is on by default: every Markdown file under `srcDir` becomes a static
HTML page with the default theme, navigation, and search UI. The site you are
reading is generated exactly this way.

```ts
import { defineConfig } from "vite-plus";
import { oxContent, defineTheme, defaultTheme } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "content",
      outDir: "dist/docs",
      ssg: {
        siteName: "Ox Content",
        siteUrl: "https://example.com",
        lastUpdated: true,
        theme: defineTheme({
          extends: defaultTheme,
          sidebar: [
            {
              text: "Guide",
              items: [{ text: "Getting Started", link: "/getting-started.md" }],
            },
          ],
        }),
      },
    }),
  ],
});
```

| Option            | Default        | Purpose                                              |
| ----------------- | -------------- | ---------------------------------------------------- |
| `enabled`         | `true`         | Set `ssg: false` to keep only `.md` modules.         |
| `extension`       | `".html"`      | Generated page extension.                            |
| `clean`           | `false`        | Remove generated output before writing.              |
| `bare`            | `false`        | Emit unthemed HTML without navigation.               |
| `siteName`        | —              | Suffix for `<title>` and OG site name.               |
| `siteUrl`         | —              | Origin used for absolute OG URLs.                    |
| `ogImage`         | —              | Static fallback OG image URL.                        |
| `generateOgImage` | `false`        | Per-page OG images (see below).                      |
| `lastUpdated`     | `false`        | Show the git last-commit time per page.              |
| `theme`           | `defaultTheme` | Theme configuration via `defineTheme()`.             |
| `navigation`      | derived        | Explicit navigation groups instead of the file tree. |

Theming — colors, fonts, header, footer, sidebar, custom CSS — is a topic of
its own: see [Theming](../theming.md).

## OG Images

Generate a social preview image per page at build time:

```ts
oxContent({
  ogImage: true,
  ssg: {
    generateOgImage: true,
    siteUrl: "https://example.com",
  },
});
```

Each page gets an image rendered from its title and description. This page's
generated image looks like this:

![Generated Open Graph image for this page](/screenshots/og-image-example.png)

| `ogImageOptions` | Default  | Purpose                                               |
| ---------------- | -------- | ----------------------------------------------------- |
| `template`       | built-in | Custom template: `.ts`, `.vue`, `.svelte`, or `.tsx`. |
| `width`          | `1200`   | Image width in pixels.                                |
| `height`         | `630`    | Image height in pixels.                               |
| `cache`          | `true`   | Skip re-rendering unchanged pages.                    |
| `concurrency`    | `1`      | Parallel image renders.                               |

During dev, `/__og-viewer` previews every page's Open Graph metadata and
image (the `ogViewer` option, on by default):

![The OG viewer during development](/screenshots/og-viewer.png)

Custom templates receive the page frontmatter as props — see
[Custom OG Image Templates](../examples/og-image-custom.md).

## Edit This Page

Append a "suggest an edit" link to every page. The option is enabled by
providing `repoUrl` — a bare `editThisPage: true` stays disabled because
there is nothing to link to:

```ts
oxContent({
  editThisPage: {
    repoUrl: "https://github.com/ubugeeei-prod/ox-content",
    branch: "main",
    label: "Edit this page",
  },
});
```

The rendered link points at the file that produced the page:

```html
<p class="ox-edit-this-page">
  <a
    href="https://github.com/ubugeeei-prod/ox-content/edit/main/docs/content/example.md"
    target="_blank"
    rel="noopener noreferrer"
    >Edit this page</a
  >
</p>
```

Set `rootDir` when source paths need a prefix stripped before being joined to
the edit URL.

## Collections

Collections expose Markdown files as a lazily-loaded, queryable manifest —
useful for blog indexes, changelogs, or "related pages" lists. A default
`content` collection covering every Markdown file exists out of the box:

```ts
import { queryCollection } from "virtual:ox-content/collections";

const guides = await queryCollection("content")
  .path("/guide")
  .order("date", "DESC")
  .limit(10)
  .all();
```

Define explicit collections when different content types need different
shapes:

```ts
import { oxContent, defineCollections } from "@ox-content/vite-plugin";

oxContent({
  collections: defineCollections({
    blog: {
      source: "blog/**/*.md",
      // Opt into heavier fields per collection:
      // "body" = raw Markdown, "html" = rendered HTML, "toc" = parsed TOC.
      include: ["html", "toc"],
    },
    changelog: "changelog/*.md",
  }),
});
```

By default only metadata (path, title, frontmatter) is in the manifest;
`include` adds `body`, `html`, or `toc` per collection. Queries run against a
Rust-generated manifest, so filtering and ordering do not load page bodies.

## API Docs

`docs` generates Markdown API references from JSDoc/TSDoc comments — the
`cargo doc` workflow for TypeScript. It is on by default (`docs: false` opts
out) and writes into `srcDir` so the generated pages join the site:

```ts
oxContent({
  docs: {
    src: ["./src"],
    out: "content/api",
    include: ["**/*.ts"],
    exclude: ["**/*.test.*"],
    githubUrl: "https://github.com/owner/repo",
    generateNav: true,
  },
});
```

The [API Reference](../api/index.md) on this site is generated by this
pipeline from the plugin's own sources. The full option set — entry points,
grouping, sorting, link styles, per-kind rendering formats — is documented in
[API Docs from JSDoc](../jsdoc.md).

## Custom Transformers

`transformers` run against the Markdown AST between parsing and rendering, for
project-specific rewrites that should stay out of page content:

```ts
import type { MarkdownTransformer } from "@ox-content/vite-plugin";

const stampDrafts: MarkdownTransformer = {
  name: "stamp-drafts",
  transform(ast, context) {
    if (context.frontmatter.draft) {
      ast.children.unshift({
        type: "paragraph",
        children: [{ type: "text", value: "🚧 Draft — not published yet." }],
      });
    }
    return ast;
  },
};

oxContent({
  transformers: [stampDrafts],
});
```

Each transformer receives the parsed AST plus `{ filePath, frontmatter,
options }` and returns the (possibly replaced) AST. Transformers compose in
array order.

## Related

- [Theming](../theming.md) — the theme system used by SSG.
- [API Docs from JSDoc](../jsdoc.md) — the full `docs` option reference.
- [Internationalization](../i18n.md) — locale-aware sites on top of SSG.
