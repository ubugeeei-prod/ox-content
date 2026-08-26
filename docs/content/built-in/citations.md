---
title: Citations
description: Opt-in bibliography-backed citation syntax for static documents.
---

# Citations

`citations` turns compact citation references into accessible links backed by
local CSL JSON files. It is separate from footnotes, so repeated source
references share one bibliography entry instead of duplicating footnote bodies.

The feature is static: bibliography files are read during transform, output is
HTML, and no client JavaScript is shipped. This page cites the HTTP Semantics
RFC with the live renderer [@rfc9110].

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      citations: {
        bibliography: "content/references.json",
        rootDir: process.cwd(),
      },
    }),
  ],
};
```

## Authoring

Use `[@key]` for one citation and separate grouped citations with semicolons:

```md
HTTP semantics are defined by the core RFC [@rfc9110].

CommonMark and HTTP are both external references [@commonmark; @rfc9110].
```

Use `-@key` inside a group when prose already names the author or standard:

```md
RFC 9110 defines HTTP semantics [-@rfc9110].
```

Generated citations are ordinary links:

```html
<span class="ox-cite" role="group" aria-label="Citations 1">
  <a class="ox-cite__ref" href="#ref-rfc9110">[1]</a>
</span>
```

When `appendBibliography` is enabled, each cited source appears once:

```html
<section class="ox-bibliography" aria-labelledby="ox-bibliography-title">
  <h2 class="ox-bibliography__title" id="ox-bibliography-title">References</h2>
  <ol class="ox-bibliography__list">
    <li class="ox-bibliography__item" id="ref-rfc9110">...</li>
  </ol>
</section>
```

## Bibliography Files

Start with CSL JSON. Files must be local paths under `rootDir`; URLs and paths
that escape `rootDir` fail before rendering.

```json
[
  {
    "id": "rfc9110",
    "title": "HTTP Semantics",
    "author": [{ "given": "Roy T.", "family": "Fielding" }],
    "issued": { "date-parts": [[2022]] },
    "URL": "https://www.rfc-editor.org/rfc/rfc9110"
  }
]
```

## Options

| Option               | Type                  | Default         |
| -------------------- | --------------------- | --------------- |
| `enabled`            | `boolean`             | `true`          |
| `bibliography`       | `string` / `string[]` | `[]`            |
| `rootDir`            | `string`              | `process.cwd()` |
| `appendBibliography` | `boolean`             | `true`          |
| `missing`            | `"error"` / `"warn"`  | `"error"`       |
| `duplicates`         | `"error"` / `"warn"`  | `"error"`       |
| `malformed`          | `"error"` / `"warn"`  | `"error"`       |
| `bibliographyTitle`  | `string`              | `"References"`  |

Set a diagnostic policy to `"warn"` when migrating existing documents. Missing
citations remain literal in warn mode, and the transform still reports them.

## Metadata

Markdown modules export citation metadata for custom renderers:

```ts
import page from "./guide.md";

for (const cite of page.citations) {
  console.log(cite.key, cite.href, cite.label);
}

for (const entry of page.bibliography) {
  console.log(entry.key, entry.title);
}
```

`renderMarkdown()` returns the same `citations` and `bibliography` arrays. Search
index text includes bibliography titles only when `citations` is enabled.

## Styling

The built-in SSG includes citation styles on pages that render `.ox-cite` or
`.ox-bibliography`. Custom hosts import the matching sheet:

```css
@import "@ox-content/vite-plugin/styles/core.css";
@import "@ox-content/vite-plugin/styles/citations.css";
```

## Related

- [Markdown Baseline](./markdown.md) - footnotes remain independent.
- [Cross References](./cross-references.md) - generated labels for page-local targets.
- [Component styles](./component-styles.md) - official CSS entry points.
