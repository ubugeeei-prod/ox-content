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
- Default: `{ github: true }`

Built-in static embeds are rendered at transform time, with no client-side JavaScript.

```md
<GitHub repo="ubugeeei/ox-content" />

<GitHub permalink="https://github.com/ubugeeei/ox-content/blob/278098b/README.md#L1-L12" />

<GitHub repo="ubugeeei/ox-content" path="README.md" ref="main" loc="1-12" />
```

Disable all embeds or configure each fetcher:

```ts
oxContent({
  embeds: {
    github: {
      token: process.env.GITHUB_TOKEN,
      maxSourceBytes: 200000,
      maxSourceLines: 120,
    },
  },
});
```

```ts
oxContent({
  embeds: false,
});
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

```ts
import config from "virtual:ox-content/config";
import { useMarkdown, withBase, withoutBase } from "virtual:ox-content/runtime";
import { search, searchOptions } from "virtual:ox-content/search";

const assetUrl = withBase("/og.png");
const routePath = withoutBase("/docs/guide");

// Use the search function
const results = await search("query", { limit: 10 });
```
