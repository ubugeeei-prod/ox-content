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

| Guide                                                | Covers                                                                    |
| ---------------------------------------------------- | ------------------------------------------------------------------------- |
| [Markdown Baseline](./built-in/markdown.md)          | GFM, tables, task lists, footnotes, autolinks, frontmatter, TOC           |
| [Syntax Extensions](./built-in/syntax-extensions.md) | Emoji shortcodes, wiki links, attribute syntax, CJK emphasis              |
| [Code Blocks](./built-in/code-blocks.md)             | Syntax highlighting, code annotations, code imports                       |
| [Embeds](./built-in/embeds.md)                       | GitHub cards, OG cards, package-manager tabs, tabs, YouTube, social cards |
| [Mermaid Diagrams](./built-in/mermaid.md)            | Diagram fences rendered to static SVG                                     |
| [Search](./built-in/search.md)                       | The static BM25 index and client search API                               |
| [Quality Checks](./built-in/quality-checks.md)       | Code block lint, type checking, docs tests, HTML sanitizer                |
| [Site Generation](./built-in/site-generation.md)     | SSG, OG images, edit links, collections, API docs, transformers           |

## Default vs Opt-in

| Area             | Option                                                                                                        | Default              | Guide                                                |
| ---------------- | ------------------------------------------------------------------------------------------------------------- | -------------------- | ---------------------------------------------------- |
| Markdown base    | `gfm`, `footnotes`, `tables`, `taskLists`, `strikethrough`, `autolinks`                                       | `true`               | [Markdown Baseline](./built-in/markdown.md)          |
| Page metadata    | `frontmatter`                                                                                                 | `true`               | [Markdown Baseline](./built-in/markdown.md)          |
| Navigation       | `toc`, `tocMaxDepth`                                                                                          | `true`, `3`          | [Markdown Baseline](./built-in/markdown.md)          |
| Static site      | `ssg`                                                                                                         | `{ enabled }`        | [Site Generation](./built-in/site-generation.md)     |
| API docs         | `docs`                                                                                                        | `{ enabled }`        | [Site Generation](./built-in/site-generation.md)     |
| Search           | `search`                                                                                                      | `{ enabled }`        | [Search](./built-in/search.md)                       |
| Collections      | `collections`                                                                                                 | `content` collection | [Site Generation](./built-in/site-generation.md)     |
| Static embeds    | `embeds.github`, `embeds.openGraph`                                                                           | `true`               | [Embeds](./built-in/embeds.md)                       |
| Opt-in embeds    | `embeds.pm`, `embeds.twitter`, `embeds.bluesky`, `embeds.spotify`, `embeds.stackBlitz`, `embeds.webContainer` | `false`              | [Embeds](./built-in/embeds.md)                       |
| Syntax highlight | `highlight`                                                                                                   | `false`              | [Code Blocks](./built-in/code-blocks.md)             |
| Code authoring   | `codeAnnotations`, `codeImports`                                                                              | `false`              | [Code Blocks](./built-in/code-blocks.md)             |
| Extra syntax     | `wikiLinks`, `emojiShortcodes`, `attrs`, `cjkEmphasis`                                                        | `false`              | [Syntax Extensions](./built-in/syntax-extensions.md) |
| Diagrams         | `mermaid`                                                                                                     | `false`              | [Mermaid Diagrams](./built-in/mermaid.md)            |
| OG images        | `ogImage`                                                                                                     | `false`              | [Site Generation](./built-in/site-generation.md)     |
| HTML safety      | `sanitize`                                                                                                    | `false`              | [Quality Checks](./built-in/quality-checks.md)       |
| Editing links    | `editThisPage`                                                                                                | `false`              | [Site Generation](./built-in/site-generation.md)     |
| Code checks      | `codeBlockLint`, `codeBlockTypecheck`, `docsTests`                                                            | `false`              | [Quality Checks](./built-in/quality-checks.md)       |
| Custom pipeline  | `transformers`                                                                                                | `[]`                 | [Site Generation](./built-in/site-generation.md)     |

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
