---
title: Built-in Features
description: Default and opt-in features available in @ox-content/vite-plugin.
---

# Built-in Features

Ox Content keeps common documentation behavior on by default and keeps
non-standard Markdown or HTML extensions opt-in.

The defaults below match `@ox-content/vite-plugin`. They are designed for a
fast static baseline: parsing, static embeds, source docs, and search indexing
run during transform or build, while extra syntax and runtime behavior must be
enabled explicitly.

This documentation site is built with Ox Content, so the feature guides below
do not just describe each feature — they enable it and **render live examples
inline**.

## Feature Guides

| Guide                                                  | Covers                                                                    |
| ------------------------------------------------------ | ------------------------------------------------------------------------- |
| [Markdown Baseline](./built-in/markdown.md)            | GFM, tables, task lists, footnotes, autolinks, frontmatter, TOC           |
| [Syntax Extensions](./built-in/syntax-extensions.md)   | Emoji shortcodes, wiki links, attribute syntax, CJK emphasis              |
| [Custom Containers](./built-in/containers.md)          | Opt-in `::: tip` / `::: details` callout blocks                           |
| [Cards](./built-in/cards.md)                           | Opt-in `::: card` / `::: link-card` / `::: card-grid` blocks              |
| [Step Lists](./built-in/steps.md)                      | Opt-in `::: steps` tutorial lists                                         |
| [File Includes](./built-in/includes.md)                | Opt-in `<!-- @include -->` Markdown fragments                             |
| [File Tree](./built-in/file-tree.md)                   | Opt-in static `file-tree` directory diagrams                              |
| [Inline Badges](./built-in/badges.md)                  | Opt-in `{badge:tip}` status labels in headings or prose                   |
| [Images](./built-in/images.md)                         | Opt-in figures, captions, lazy loading, and safe dimensions               |
| [Code Blocks](./built-in/code-blocks.md)               | Syntax highlighting, code annotations, code imports                       |
| [Embeds](./built-in/embeds.md)                         | GitHub cards, OG cards, package-manager tabs, tabs, YouTube, social cards |
| [Mermaid Diagrams](./built-in/mermaid.md)              | Diagram fences rendered to static SVG                                     |
| [Math](./built-in/math.md)                             | Opt-in `$…$` inline and `$$…$$` block math                                |
| [Search](./built-in/search.md)                         | The static BM25 index and client search API                               |
| [Collections](./built-in/collections.md)               | Query Markdown files with a SQL-like builder                              |
| [Quality Checks](./built-in/quality-checks.md)         | Code block lint, type checking, docs tests, HTML sanitizer                |
| [Site Generation](./built-in/site-generation.md)       | SSG, OG images, edit links, collections, API docs, transformers           |
| [Previous / Next](./built-in/pagination.md)            | Opt-in previous and next page links                                       |
| [Breadcrumbs](./built-in/breadcrumbs.md)               | Opt-in trail from the site root through sidebar ancestors                 |
| [Reader Chrome](./built-in/reader-chrome.md)           | Opt-in copy, outbound-link icons, and back-to-top                         |
| [Sitemap / robots / llms.txt](./built-in/site-maps.md) | Opt-in crawl manifests written next to generated HTML                     |
| [Draft / unlisted / scheduled](./built-in/drafts.md)   | Opt-in frontmatter publish states for production output                   |
| [Permalinks and Cascade](./built-in/permalinks.md)     | Opt-in frontmatter URLs and directory-level default frontmatter           |
| [Redirects and aliases](./built-in/redirects.md)       | Opt-in static HTML redirects from aliases and a rewrite map               |
| [Custom 404](./built-in/not-found.md)                  | Opt-in themed 404 page with nav and search                                |
| [RSS / Atom / JSON feeds](./built-in/feeds.md)         | Opt-in collection feeds written next to generated HTML                    |

## Default vs Opt-in

| Area             | Option                                                                                                        | Default              | Guide                                                  |
| ---------------- | ------------------------------------------------------------------------------------------------------------- | -------------------- | ------------------------------------------------------ |
| Markdown base    | `gfm`, `footnotes`, `tables`, `taskLists`, `strikethrough`, `autolinks`                                       | `true`               | [Markdown Baseline](./built-in/markdown.md)            |
| Page metadata    | `frontmatter`                                                                                                 | `true`               | [Markdown Baseline](./built-in/markdown.md)            |
| Navigation       | `toc`, `tocMaxDepth`                                                                                          | `true`, `3`          | [Markdown Baseline](./built-in/markdown.md)            |
| Static site      | `ssg`                                                                                                         | `{ enabled }`        | [Site Generation](./built-in/site-generation.md)       |
| API docs         | `docs`                                                                                                        | `{ enabled }`        | [Site Generation](./built-in/site-generation.md)       |
| Search           | `search`                                                                                                      | `{ enabled }`        | [Search](./built-in/search.md)                         |
| Collections      | `collections`                                                                                                 | `content` collection | [Collections](./built-in/collections.md)               |
| Static embeds    | `embeds.github`, `embeds.openGraph`                                                                           | `true`               | [Embeds](./built-in/embeds.md)                         |
| Opt-in embeds    | `embeds.pm`, `embeds.twitter`, `embeds.bluesky`, `embeds.spotify`, `embeds.stackBlitz`, `embeds.webContainer` | `false`              | [Embeds](./built-in/embeds.md)                         |
| Syntax highlight | `highlight`                                                                                                   | `false`              | [Code Blocks](./built-in/code-blocks.md)               |
| Code authoring   | `codeAnnotations`, `codeImports`                                                                              | `false`              | [Code Blocks](./built-in/code-blocks.md)               |
| Extra syntax     | `wikiLinks`, `emojiShortcodes`, `attrs`, `cjkEmphasis`, `containers`, `badges`                                | `false`              | [Syntax Extensions](./built-in/syntax-extensions.md)   |
| File includes    | `includes`                                                                                                    | `false`              | [File Includes](./built-in/includes.md)                |
| Cards            | `cards`                                                                                                       | `false`              | [Cards](./built-in/cards.md)                           |
| Step lists       | `steps`                                                                                                       | `false`              | [Step Lists](./built-in/steps.md)                      |
| File tree        | `fileTree`                                                                                                    | `false`              | [File Tree](./built-in/file-tree.md)                   |
| Images           | `images`                                                                                                      | `false`              | [Images](./built-in/images.md)                         |
| Diagrams         | `mermaid`                                                                                                     | `false`              | [Mermaid Diagrams](./built-in/mermaid.md)              |
| Math             | `math`                                                                                                        | `false`              | [Math](./built-in/math.md)                             |
| OG images        | `ogImage`                                                                                                     | `false`              | [Site Generation](./built-in/site-generation.md)       |
| HTML safety      | `sanitize`                                                                                                    | `false`              | [Quality Checks](./built-in/quality-checks.md)         |
| Editing links    | `editThisPage`                                                                                                | `false`              | [Site Generation](./built-in/site-generation.md)       |
| Page pager       | `ssg.pagination`                                                                                              | `false`              | [Previous / Next](./built-in/pagination.md)            |
| Breadcrumbs      | `ssg.breadcrumbs` / `theme.breadcrumbs`                                                                       | `false`              | [Breadcrumbs](./built-in/breadcrumbs.md)               |
| Reader chrome    | `ssg.readerChrome`                                                                                            | `false`              | [Reader Chrome](./built-in/reader-chrome.md)           |
| Crawl manifests  | `siteMaps`                                                                                                    | `false`              | [Sitemap / robots / llms.txt](./built-in/site-maps.md) |
| Publish states   | `publishState`                                                                                                | `false`              | [Draft / unlisted / scheduled](./built-in/drafts.md)   |
| Permalinks       | `permalinks`                                                                                                  | `false`              | [Permalinks and Cascade](./built-in/permalinks.md)     |
| Frontmatter tree | `cascade`                                                                                                     | `false`              | [Permalinks and Cascade](./built-in/permalinks.md)     |
| Redirects        | `redirects`                                                                                                   | `false`              | [Redirects and aliases](./built-in/redirects.md)       |
| Custom 404       | `ssg.notFound`                                                                                                | `false`              | [Custom 404](./built-in/not-found.md)                  |
| Collection feeds | `feeds`                                                                                                       | `false`              | [RSS / Atom / JSON feeds](./built-in/feeds.md)         |
| Code checks      | `codeBlockLint`, `codeBlockTypecheck`, `docsTests`                                                            | `false`              | [Quality Checks](./built-in/quality-checks.md)         |
| Custom pipeline  | `transformers`                                                                                                | `[]`                 | [Site Generation](./built-in/site-generation.md)       |

Tab groups and YouTube embeds have no option: they are always processed for
SSG output and dev preview. See [Embeds](./built-in/embeds.md#tabs).

## Example Configuration

Use explicit options when a site needs non-standard behavior:

```ts
import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      highlight: true,
      emojiShortcodes: true,
      codeAnnotations: {
        notation: "both",
      },
      embeds: {
        pm: { sync: true },
        twitter: { fetch: true },
        bluesky: true,
      },
    }),
  ],
});
```

Every option follows the same convention: `false` disables the feature, `true`
enables it with defaults, and an object enables it while overriding only the
fields you set.

Copyable source snippets for the authoring forms live in
`examples/builtin-features/content/`, and the pages under
[Examples](./examples/index.md) show several features in runnable projects.
