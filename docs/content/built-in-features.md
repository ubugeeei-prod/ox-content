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

## Default vs Opt-in

| Area             | Option                                                                  | Default       | Notes                                               |
| ---------------- | ----------------------------------------------------------------------- | ------------- | --------------------------------------------------- |
| Markdown base    | `gfm`, `footnotes`, `tables`, `taskLists`, `strikethrough`, `autolinks` | `true`        | Common GitHub-flavored Markdown behavior.           |
| Page metadata    | `frontmatter`                                                           | `true`        | Parses YAML frontmatter before rendering.           |
| Navigation       | `toc`, `tocMaxDepth`                                                    | `true`, `3`   | Builds a page table of contents from headings.      |
| Static site      | `ssg`                                                                   | `{ enabled }` | Generates static HTML pages during build.           |
| API docs         | `docs`                                                                  | `{ enabled }` | Generates package API docs unless set to `false`.   |
| Search           | `search`                                                                | `{ enabled }` | Builds a static BM25 index for client-side search.  |
| Collections      | `collections`                                                           | `{ content }` | Rust-native lazy query manifest for Markdown files. |
| Syntax highlight | `highlight`                                                             | `false`       | Opt in when the site needs highlighted code blocks. |
| OG images        | `ogImage`                                                               | `false`       | Opt in because image rendering adds build work.     |
| Extra syntax     | `wikiLinks`, `emojiShortcodes`, `attrs`, `codeImports`, `cjkEmphasis`   | `false`       | Non-standard authoring features are opt-in.         |
| HTML safety      | `sanitize`                                                              | `false`       | Opt in when rendering untrusted or mixed HTML.      |
| Editing links    | `editThisPage`                                                          | `false`       | Opt in with a repository URL.                       |
| Code checks      | `codeAnnotations`, `codeBlockLint`, `codeBlockTypecheck`, `docsTests`   | `false`       | Opt in per documentation workflow.                  |
| Diagrams         | `mermaid`                                                               | `false`       | Opt in for diagram rendering.                       |
| Custom pipeline  | `transformers`                                                          | `[]`          | Add project-specific Markdown AST transforms.       |

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

## Emoji Shortcodes

Emoji shortcode expansion is opt-in:

```ts
oxContent({
  emojiShortcodes: true,
});
```

The built-in table covers hundreds of common GitHub-style aliases such as
`:rocket:`, `:white_check_mark:`, `:warning:`, `:smile:`, and `:thinking:`.
Shortcodes expand outside fenced and inline code. Unknown shortcodes are left
unchanged.

Custom values override the built-in table:

```ts
oxContent({
  emojiShortcodes: {
    custom: {
      shipit: "ship it",
    },
  },
});
```

See [Emoji Shortcodes](./examples/emoji-shortcodes.md) for a rendered example.

## Code Annotations

Code annotations are opt-in so ordinary code fences stay literal unless a site
chooses annotation syntax:

```ts
oxContent({
  highlight: true,
  codeAnnotations: {
    notation: "both",
  },
});
```

The default notation is the configurable attribute syntax:

````md
```ts annotate="highlight:1,3;warning:5;error:8"
const value = load();
console.warn(value);
throw new Error("stop");
```
````

`notation: "vitepress"` enables VitePress-compatible fence metadata and inline
comments. `notation: "both"` enables both syntaxes.

Use a standalone escape comment when the next line should render literally even
if it contains annotation-looking text:

````md
```ts
// [!code escape]
console.warn("literal"); // [!code warning]
console.warn("annotated"); // [!code warning]
```
````

The escape directive is removed from the rendered block, and only the next line
is escaped.

See [Code Annotations](./examples/code-annotations.md) for the complete syntax.

## Built-in Embeds

`embeds.github` and `embeds.openGraph` are enabled by default because they render
static HTML at transform time. Non-standard or runtime-heavy embeds are opt-in.

| Embed                         | Default | Authoring form                     | Runtime behavior                          |
| ----------------------------- | ------- | ---------------------------------- | ----------------------------------------- |
| GitHub repository/source card | `true`  | `<GitHub repo="owner/name" />`     | Static HTML generated during transform.   |
| Open Graph link card          | `true`  | `<OgCard url="https://..." />`     | Static HTML generated during transform.   |
| Package manager tabs          | `false` | `<pm>npm install package</pm>`     | Static HTML; sync mode adds small JS.     |
| Spotify                       | `false` | `<Spotify url="https://..." />`    | Iframe embed.                             |
| StackBlitz                    | `false` | `<StackBlitz url="https://..." />` | Iframe embed.                             |
| Twitter/X                     | `false` | `<Tweet />` or `<XPost />`         | Static privacy-conscious card.            |
| Bluesky                       | `false` | `<Bluesky />`                      | Static card.                              |
| WebContainer                  | `false` | `<WebContainer />`                 | Lazy placeholder with isolation metadata. |

Disable every built-in embed with `embeds: false`, or configure only the embeds
your site needs:

```ts
oxContent({
  embeds: {
    github: {
      token: process.env.GITHUB_TOKEN,
      maxSourceLines: 120,
    },
    openGraph: {
      timeout: 5000,
    },
    pm: true,
  },
});
```

Copyable source snippets for these forms live in
`examples/builtin-features/content/`.

## Mermaid Diagrams

Mermaid rendering is opt-in:

```ts
oxContent({
  mermaid: true,
});
```

See `examples/builtin-features/content/mermaid-diagram.md` for a small diagram
source.

## Search Index

Search is enabled by default for SSG builds. Configure or disable it explicitly:

```ts
oxContent({
  search: {
    limit: 8,
    hotkey: "/",
  },
});
```

See `examples/builtin-features/client/search.ts` for the virtual module import.

## Code Quality Hooks

Documentation-specific checks are also opt-in:

| Option               | Default | Use when...                                                    |
| -------------------- | ------- | -------------------------------------------------------------- |
| `codeBlockLint`      | `false` | Code fences should require languages or reject trailing space. |
| `codeBlockTypecheck` | `false` | TypeScript or TSX fences should be checked with `tsgo`.        |
| `docsTests`          | `false` | Runnable fences should be extracted for a Vitest harness.      |
| `sanitize`           | `false` | Raw or third-party HTML should be cleaned before rendering.    |
| `codeImports`        | `false` | Markdown should import checked source snippets from files.     |

The feature pages under [Examples](./examples/index.md) and the snippets under
`examples/builtin-features/` show each hook in isolation, so a site can enable
only the pieces it actually uses.
