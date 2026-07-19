---
title: Markdown Baseline
description: GitHub-flavored Markdown, frontmatter, and table-of-contents defaults that are on out of the box.
---

# Markdown Baseline

Common GitHub-flavored Markdown behavior is enabled by default. You do not need
any configuration for the features on this page — every rendered example below
is produced by this documentation site itself with the default settings.

| Option          | Type      | Default          | Purpose                                     |
| --------------- | --------- | ---------------- | ------------------------------------------- |
| `gfm`           | `boolean` | `true`           | GitHub Flavored Markdown extensions.        |
| `tables`        | `boolean` | `true`           | GFM tables.                                 |
| `taskLists`     | `boolean` | `true`           | `- [ ]` / `- [x]` checkboxes.               |
| `strikethrough` | `boolean` | `true`           | `~~text~~`.                                 |
| `autolinks`     | `boolean` | inherits `gfm`   | Turn bare URLs into links.                  |
| `footnotes`     | `boolean` | `true`           | `[^1]` references and definitions.          |
| `frontmatter`   | `boolean` | `true`           | Parse YAML frontmatter before rendering.    |
| `toc`           | `boolean` | `true`           | Build a table of contents from headings.    |
| `tocMaxDepth`   | `number`  | `3`              | Deepest heading level included in the TOC.  |

Turn any of them off explicitly when a site needs stricter CommonMark behavior:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      strikethrough: false,
      taskLists: false,
    }),
  ],
};
```

## Tables

```md
| Feature    | Status  |
| ---------- | ------- |
| Tables     | Default |
| Task lists | Default |
```

Rendered:

| Feature    | Status  |
| ---------- | ------- |
| Tables     | Default |
| Task lists | Default |

## Task Lists

```md
- [x] Parse Markdown in Rust
- [x] Render HTML
- [ ] Take over the world
```

Rendered:

- [x] Parse Markdown in Rust
- [x] Render HTML
- [ ] Take over the world

## Strikethrough

```md
Ox Content is ~~slow~~ fast.
```

Rendered:

Ox Content is ~~slow~~ fast.

## Autolinks

Bare URLs become links. The default follows `gfm`, so `autolinks: false` opts
out without giving up the rest of GFM.

```md
Docs live at https://ubugeeei-prod.github.io/ox-content/
```

Rendered:

Docs live at https://ubugeeei-prod.github.io/ox-content/

Auto-linked URLs open in a new tab with `rel="noopener noreferrer"`.

## Footnotes

```md
Ox Content renders footnotes natively.[^1]

[^1]: This is the footnote body.
```

Rendered:

Ox Content renders footnotes natively.[^1]

[^1]: This is the footnote body.

The reference becomes a superscript link, and the definition renders where it
is written in the source with a back-link — put definitions at the bottom of a
page to collect them there.

## Frontmatter

YAML frontmatter is parsed before rendering and never appears in the output
HTML. This page starts with:

```yaml
---
title: Markdown Baseline
description: GitHub-flavored Markdown, frontmatter, and table-of-contents defaults that are on out of the box.
---
```

The SSG theme uses `title` for the document title and navigation, and
`description` for `<meta name="description">` and Open Graph tags. Any other
keys are passed through: `.md` modules expose them as the `frontmatter` export,
[collections](./site-generation.md#collections) expose them to queries, and
[custom transformers](./site-generation.md#custom-transformers) receive them as
`context.frontmatter`.

```ts
import { frontmatter, html } from "./guide.md";

console.log(frontmatter.title); // "Markdown Baseline"
```

## Table of Contents

A TOC is built from headings during transform. The sidebar navigation for this
very page is driven by it. `tocMaxDepth: 3` means `#` through `###` are
included by default; deeper headings are rendered but not indexed.

```ts
oxContent({
  toc: true,
  tocMaxDepth: 3,
});
```

The TOC is exposed to `.md` modules as a tree of `{ depth, text, slug,
children }` entries:

```json
[
  {
    "depth": 1,
    "text": "Install Guide",
    "slug": "install-guide",
    "children": [
      { "depth": 2, "text": "Prerequisites", "slug": "prerequisites", "children": [] },
      { "depth": 2, "text": "Run Vite", "slug": "run-vite", "children": [] }
    ]
  }
]
```

Every heading also gets a stable `id` attribute (the `slug` above), so deep
links like [#task-lists](#task-lists) work on every page.

## Related

- [Syntax Extensions](./syntax-extensions.md) — opt-in authoring syntax on top
  of this baseline.
- [Built-in Features overview](../built-in-features.md)
