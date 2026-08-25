---
title: Markdown Baseline
description: GitHub-flavored Markdown, frontmatter, and table-of-contents defaults that are on out of the box.
---

# Markdown Baseline

Common GitHub-flavored Markdown behavior is enabled by default. You do not need
any configuration for the features on this page — every rendered example below
is produced by this documentation site itself with the default settings.

| Option              | Type      | Default        | Purpose                                                |
| ------------------- | --------- | -------------- | ------------------------------------------------------ |
| `gfm`               | `boolean` | `true`         | GitHub Flavored Markdown extensions.                   |
| `tables`            | `boolean` | `true`         | GFM tables.                                            |
| `taskLists`         | `boolean` | `true`         | `- [ ]` / `- [x]` checkboxes.                          |
| `strikethrough`     | `boolean` | `true`         | `~~text~~`.                                            |
| `autolinks`         | `boolean` | inherits `gfm` | Turn bare URLs into links.                             |
| `footnotes`         | `boolean` | `true`         | `[^1]` references and definitions.                     |
| `semanticFootnotes` | `boolean` | `false`        | Numeric markers and one `<section class="footnotes">`. |
| `frontmatter`       | `boolean` | `true`         | Parse YAML frontmatter before rendering.               |
| `toc`               | `boolean` | `true`         | Build a table of contents from headings.               |
| `tocMaxDepth`       | `number`  | `3`            | Deepest heading level included in the TOC.             |

Every option above is an extension on top of CommonMark, and each is opt-out.
The parser underneath targets full conformance: it renders all 652
[CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/) spec examples correctly
in its core profile, checked on every CI run. A document that uses none of the
extensions conforms to the specification under the conformance suite's HTML
normalization rule; the markup is not byte-identical, because ox-content adds
slug `id` attributes to headings. See
[CommonMark Conformance](../performance.md#commonmark-conformance) for the
per-profile numbers.

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

The reference becomes a superscript link. With the default renderer the visible
marker is the source identifier, and each definition is emitted in place as
`<div class="footnote">` — put definitions at the bottom of a page to collect
them there.

Set `semanticFootnotes: true` to assign stable numeric markers in document
order (`[^deployment-note]` → 1, 2, …) and collect definitions into one
accessible section. The source identifier is used only for lookup and slug
generation (`fn-…` / `fnref-…`). Repeated references keep unique ids
(`fnref-note`, `fnref-note-2`, …) and each occurrence gets a backlink.
Definition bodies keep their block content. No client JavaScript is required.

```html
<section class="footnotes" aria-label="Footnotes">
  <ol>
    <li id="fn-deployment-note">
      … <a href="#fnref-deployment-note" aria-label="Back to reference 1">↩</a>
    </li>
  </ol>
</section>
```

```ts
oxContent({
  footnotes: true,
  semanticFootnotes: true,
});
```

This documentation site enables `semanticFootnotes` so the live example above
uses the ordered section. The option stays off by default so current alpha
HTML does not change.

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

Visible `#` permalink controls next to the heading are opt-in. See
[Heading Permalinks](./heading-permalinks.md). Off by default so existing HTML
stays unchanged.

## Related

- [Heading Permalinks](./heading-permalinks.md) — opt-in visible `#` links on
  those ids.
- [Syntax Extensions](./syntax-extensions.md) — opt-in authoring syntax on top
  of this baseline.
- [Built-in Features overview](../built-in-features.md)
