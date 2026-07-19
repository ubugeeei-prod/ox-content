# @ox-content/vite-plugin

Base Vite plugin for Ox Content with Environment API support.

## Installation

```bash
vp install @ox-content/vite-plugin
```

`@ox-content/vite-plugin` already depends on `@ox-content/napi`, so a separate `vp install @ox-content/napi` is not required when you are using the Vite plugin.

## Basic Usage

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
    }),
  ],
});
```

## VitePress Migration

If you already have a VitePress site, generate an editable ox-content options object:

```bash
ox-content-migrate-vitepress .vitepress/config.ts \
  --src-dir docs \
  --out-dir dist \
  --out ox-content.config.ts
```

The CLI can run on Node.js, Deno, or Bun:

```bash
# Node.js, after installing @ox-content/vite-plugin
ox-content-migrate-vitepress .vitepress/config.ts --out ox-content.config.ts

# Deno
deno run -A npm:@ox-content/vite-plugin/vitepress-migrate .vitepress/config.ts \
  --out ox-content.config.ts

# Bun
bunx --bun @ox-content/vite-plugin .vitepress/config.ts --out ox-content.config.ts
```

The generated `ox-content.config.ts` maps these settings into ox-content:

- `title` / `themeConfig.siteTitle` -> `ssg.siteName`
- `base` -> `base`
- `themeConfig.sidebar` -> `ssg.navigation`
- `themeConfig.socialLinks` / `themeConfig.footer` / `themeConfig.logo` -> `ssg.theme`
- `themeConfig.search.placeholder` -> `search.placeholder`

For landing pages, VitePress-style `layout: home` frontmatter is treated the same as
ox-content's `layout: entry`.

## Options

For a consolidated default table, including which non-standard features are
opt-in, see [Built-in Features](../built-in-features.md).

### srcDir

- Type: `string`
- Default: `'docs'`

Source directory for Markdown files.

### extensions

- Type: `string[]`
- Default: `['.md', '.markdown', '.mdx']`

Markdown-like file extensions processed by the Vite plugin, SSG, dev server, search index, and OG viewer.

### outDir

- Type: `string`
- Default: `'dist'`

Output directory for built files.

### ssg

- Type: `SsgOptions | boolean`
- Default: `{ enabled: true }`

SSG (Static Site Generation) options. By default, ox-content generates static HTML files for each Markdown file during build.

```ts
oxContent({
  ssg: {
    enabled: true,
    extension: ".html",
    clean: false,
  },
});
```

#### SsgOptions

| Option      | Type      | Default   | Description                                 |
| ----------- | --------- | --------- | ------------------------------------------- |
| `enabled`   | `boolean` | `true`    | Enable/disable SSG mode                     |
| `extension` | `string`  | `'.html'` | Output file extension                       |
| `clean`     | `boolean` | `false`   | Clean output directory before build         |
| `bare`      | `boolean` | `false`   | Bare HTML output (no navigation, no styles) |

### Bare Mode (for benchmarking)

```ts
oxContent({
  ssg: {
    bare: true, // Output minimal HTML without navigation/styles
  },
});
```

### Disabling SSG

```ts
oxContent({
  ssg: false, // Disable SSG, use as module transformer only
});
```

### gfm

- Type: `boolean`
- Default: `true`

Enable GitHub Flavored Markdown extensions.

### codeAnnotations

- Type: `boolean | CodeAnnotationsOptions`
- Default: `false`

Enables opt-in code block annotations for fenced code blocks.

By default, Ox Content uses the configurable attribute syntax. You can also opt into VitePress-compatible fence metadata and inline notation, or enable both at the same time.

```ts
oxContent({
  highlight: true,
  codeAnnotations: {
    notation: "both",
  },
});
```

Attribute syntax with the default `metaKey`:

````md
```ts annotate="highlight:1,6;warning:2;error:3"
export function loadUser(input: string) {
  if (!input) console.warn("missing payload");
  throw new Error("missing id");
}

const user = loadUser(payload);
console.log(user);
```
````

VitePress-compatible syntax:

````md
```ts:line-numbers=10 {1,4} [config.ts]
const user = loadUser(payload);
console.warn("Deprecated") // [!code warning]
throw new Error("boom") // [!code error]
```
````

Rendered example:

```ts annotate="highlight:1,6;warning:2;error:3"
export function loadUser(input: string) {
  if (!input) console.warn("missing payload");
  throw new Error("missing id");
}

const user = loadUser(payload);
console.log(user);
```

You can also customize the attribute name:

```ts
oxContent({
  codeAnnotations: {
    metaKey: "markers",
  },
});
```

See the [Code Annotations example](../examples/code-annotations.md) for a rendered example.

### toc

- Type: `boolean`
- Default: `true`

Generate table of contents.

### embeds

- Type: `BuiltinEmbedOptions | false`
- Default: `{ github: true, openGraph: true, pm: false, spotify: false, stackBlitz: false, twitter: false, bluesky: false, webContainer: false }`

Built-in static embeds are rendered at transform time, with no client-side JavaScript.
Non-standard embeds are opt-in.
See [Embeds](../built-in/embeds.md) for the full
default table and rendered examples.

```md
<GitHub repo="ubugeeei-prod/ox-content" />

<GitHub permalink="https://github.com/ubugeeei-prod/ox-content/blob/278098b/README.md#L1-L12" />

<GitHub repo="ubugeeei-prod/ox-content" path="README.md" ref="main" loc="1-12" />

<OgCard url="https://github.com/ubugeeei-prod/ox-content" />
```

`permalink`, `url`, and `href` accept GitHub `blob` URLs. The `#L1-L12` fragment is used as the source line range. You can also use `repo`, `path`, `ref`, and `loc` when you do not want to paste the full permalink. Source embeds fetch the GitHub contents API and render code directly instead of using an Open Graph preview.

Disable all embeds or configure each fetcher:

```ts
oxContent({
  embeds: {
    github: {
      token: process.env.GITHUB_TOKEN,
      maxSourceBytes: 200000,
      maxSourceLines: 120,
    },
    openGraph: {
      timeout: 5000,
    },
    pm: true,
  },
});
```

```ts
oxContent({
  embeds: false,
});
```

#### Styling built-in embeds

Built-in embed markup uses stable CSS classes so the generated HTML can be themed without client-side JavaScript.

Repository card classes:

- `.ox-github-card`
- `.ox-github-header`
- `.ox-github-icon`
- `.ox-github-repo`
- `.ox-github-description`
- `.ox-github-stats`
- `.ox-github-stat`
- `.ox-github-language`

Source code card classes:

- `.ox-github-code`
- `.ox-github-code-header`
- `.ox-github-code-title`
- `.ox-github-code-loc`
- `.ox-github-code-block`
- `.ox-github-code-line`
- `.ox-github-code-line-number`
- `.ox-github-code-line-content`

Open Graph card classes:

- `.ox-ogp-card`
- `.ox-ogp-simple`
- `.ox-ogp-content`
- `.ox-ogp-title`
- `.ox-ogp-description`
- `.ox-ogp-image`
- `.ox-ogp-meta`
- `.ox-ogp-domain`
- `.ox-ogp-favicon`

```css
.ox-github-card,
.ox-github-code,
.ox-ogp-card {
  border-color: var(--my-border-color);
}

.ox-github-code-line-number,
.ox-ogp-domain {
  color: var(--my-muted-color);
}
```

### docs

- Type: `DocsOptions | false`
- Default: `{ enabled: true }`

Source documentation generation options. Set to `false` to disable.

Generated API pages now include summary stats, signature badges, one-line symbol overviews, expandable detail sections, and labeled examples. A machine-readable `docs.json` payload with aggregate counts is also emitted next to the Markdown files so custom viewers can build richer experiences without re-parsing source.

```ts
oxContent({
  docs: {
    enabled: true,
    src: ["./src"],
    out: "docs/api",
    include: ["**/*.ts"],
    exclude: ["**/*.test.*"],
    format: "markdown",
    toc: true,
    groupBy: "file",
  },
});
```

#### DocsOptions

| Option    | Type                             | Default                          | Description                    |
| --------- | -------------------------------- | -------------------------------- | ------------------------------ |
| `enabled` | `boolean`                        | `true`                           | Enable/disable docs generation |
| `src`     | `string[]`                       | `['./src']`                      | Source directories to scan     |
| `out`     | `string`                         | `'docs/api'`                     | Output directory               |
| `include` | `string[]`                       | JS/TS source globs               | Files to include               |
| `exclude` | `string[]`                       | `['**/*.test.*', '**/*.spec.*']` | Files to exclude               |
| `format`  | `'markdown' \| 'json' \| 'html'` | `'markdown'`                     | Output format                  |
| `private` | `boolean`                        | `false`                          | Include @private members       |
| `toc`     | `boolean`                        | `true`                           | Generate table of contents     |
| `groupBy` | `'file' \| 'category'`           | `'file'`                         | Group docs by file or category |

## Disabling Docs Generation

```ts
oxContent({
  docs: false, // Opt-out of builtin docs generation
});
```

### search

- Type: `SearchOptions | boolean`
- Default: `{ enabled: true }`

Full-text search options. Ox Content includes a built-in search engine powered by Rust with BM25 scoring.

```ts
oxContent({
  search: {
    enabled: true,
    limit: 10,
    prefix: true,
    placeholder: "Search documentation...",
    hotkey: "/",
  },
});
```

#### SearchOptions

| Option        | Type      | Default                     | Description                             |
| ------------- | --------- | --------------------------- | --------------------------------------- |
| `enabled`     | `boolean` | `true`                      | Enable/disable search functionality     |
| `limit`       | `number`  | `10`                        | Maximum number of search results        |
| `prefix`      | `boolean` | `true`                      | Enable prefix matching for autocomplete |
| `placeholder` | `string`  | `'Search documentation...'` | Placeholder text for search input       |
| `hotkey`      | `string`  | `'/'`                       | Keyboard shortcut to open search        |

#### How It Works

1. **Build Time**: The plugin scans all Markdown files and builds a search index using the Rust-based search engine
2. **Index Storage**: The index is written to `search-index.json` in the output directory
3. **Client-Side Search**: The search index is loaded on-demand and searched entirely client-side

#### Features

- **BM25 Scoring**: Industry-standard relevance ranking algorithm
- **Multi-field Search**: Title, headings, body, and code are indexed with different weights
- **Japanese/CJK Support**: Proper tokenization for CJK characters
- **Prefix Matching**: Type-ahead suggestions for autocomplete
- **Scoped Queries**: Prefix queries like `@api transform` to limit results by section
- **Zero Dependencies**: No external search service required

### Disabling Search

```ts
oxContent({
  search: false, // Disable built-in search
});
```

### Using with Custom Search UI

You can access the search index programmatically via the virtual module:

```ts
import { search, searchOptions } from "virtual:ox-content/search";

// Search the index
const results = await search("query text", { limit: 5 });

// Scope search to the API reference
const apiResults = await search("@api transform", { limit: 5 });

// Results include:
// - id: document ID
// - title: document title
// - url: document URL
// - score: relevance score
// - snippet: text snippet with context
```

### collections

- Type: `CollectionsOptions | boolean`
- Default: `{ content: { source: "**/*" } }`

Collections expose Markdown frontmatter and route metadata through `virtual:ox-content/collections`.
They are built only when that virtual module is imported. The default payload is metadata-only:
Ox Content uses a native Rust manifest builder for directory walking, source pattern filtering,
frontmatter parsing, route path generation, and title extraction, so large Markdown trees avoid
per-file JavaScript/NAPI round trips and do not render every Markdown file into HTML.

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { defineCollection, oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "content",
      collections: {
        blog: defineCollection({
          source: "blog/**/*.md",
        }),
        docs: defineCollection({
          source: "docs/**/*.md",
          include: ["body"],
        }),
      },
    }),
  ],
});
```

```ts
import { queryCollection } from "virtual:ox-content/collections";

const posts = await queryCollection("blog")
  .where("draft", "=", false)
  .order("date", "DESC")
  .select("title", "path", "description")
  .all();

const page = await queryCollection("docs").path("/docs/getting-started").first();
```

`include` is intentionally explicit for large sites:

| Field  | Cost                                                   |
| ------ | ------------------------------------------------------ |
| `body` | Embeds stripped raw Markdown into the virtual module.  |
| `html` | Runs the native Markdown transform and embeds HTML.    |
| `toc`  | Runs the native Markdown transform and embeds the TOC. |

For full page-level JavaScript post-processing such as syntax highlighting or Mermaid rendering,
import the Markdown module directly. Collection `html` is optimized for query payloads.

Disable collections entirely with `collections: false`.

## Environment API

The plugin creates a `markdown` environment using Vite's Environment API for SSG-focused rendering.

## HMR Support

Markdown files are hot-reloaded during development. The plugin sends custom HMR events:

```ts
// Client-side
if (import.meta.hot) {
  import.meta.hot.on("ox-content:update", (data) => {
    console.log("Markdown updated:", data.file);
  });
}
```

## Virtual Modules

The plugin provides virtual modules:

- `virtual:ox-content/config` - Resolved plugin configuration
- `virtual:ox-content/runtime` - Runtime utilities
- `virtual:ox-content/search` - Search functionality
- `virtual:ox-content/collections` - Collection query helpers

```ts
import config from "virtual:ox-content/config";
import { useMarkdown, withBase, withoutBase } from "virtual:ox-content/runtime";
import { search, searchOptions } from "virtual:ox-content/search";
import { queryCollection } from "virtual:ox-content/collections";

const assetUrl = withBase("/og.png");
const routePath = withoutBase("/docs/guide");

// Use the search function
const results = await search("query", { limit: 10 });

const page = await queryCollection("content").path("/guide").first();
```
