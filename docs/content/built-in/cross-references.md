---
title: Cross References
description: Opt-in labels and generated links for figures, tables, and sections.
---

# Cross References

`crossReferences` turns stable labels into generated links such as `Section 1.2`,
`Figure 1`, and `Table 1`. It is opt-in and uses normal `id` attributes, so the
same labels work with heading permalinks, search indexing, and custom renderers.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      attrs: true,
      images: true,
      crossReferences: true,
    }),
  ],
};
```

## Authoring

Use prefixed labels for the target kind, then refer to the label with `@id`:

```md
## Install {#sec-install}

See @sec-install and @fig-pipeline.

![Pipeline](./pipeline.png "Build pipeline"){#fig-pipeline}

| Option         | Value  |
| -------------- | ------ |
| mode           | static |
| {#tbl-options} |

See @tbl-options.
```

The generated references are links:

```html
<a class="ox-xref ox-xref-section" href="#sec-install">Section 1.1</a>
<a class="ox-xref ox-xref-figure" href="#fig-pipeline">Figure 1</a>
<a class="ox-xref ox-xref-table" href="#tbl-options">Table 1</a>
```

Target elements receive stable metadata attributes:

```html
<h2
  id="sec-install"
  data-ox-xref-kind="section"
  data-ox-xref-number="1.1"
  data-ox-xref-label="Section 1.1"
>
  Install
</h2>
```

## Label Prefixes

| Prefix              | Target kind     | Generated text |
| ------------------- | --------------- | -------------- |
| `sec-` / `section-` | Heading         | `Section N`    |
| `fig-` / `figure-`  | Figure or image | `Figure N`     |
| `tbl-` / `table-`   | Table           | `Table N`      |

Section numbers follow heading depth, while figure and table numbers are
per-page counters in document order. Reordering sections, figures, or tables
updates every generated reference text.

## Diagnostics

Missing labels, duplicate labels, and label-prefix mismatches fail by default:

```ts
oxContent({
  crossReferences: {
    missing: "error",
    duplicates: "error",
    mismatches: "error",
  },
});
```

Set a policy to `"warn"` when a migration needs unresolved references to remain
literal while still reporting them during builds.

The transform skips fenced code, indented code, inline code, raw
`<pre>`/`<code>`/`<script>`/`<style>` blocks, HTML comments, and existing links.

## Metadata

Markdown modules export the collected targets as `crossReferences`, and
`transformMarkdown()` returns the same array:

```ts
import page from "./guide.md";

for (const reference of page.crossReferences) {
  console.log(reference.id, reference.kind, reference.text);
}
```

Each entry includes `id`, `kind`, `number`, `label`, `text`, `href`, and an
optional `title` extracted from heading text, image alt text, or figcaption
content.

## Related

- [Syntax Extensions](./syntax-extensions.md) - `attrs` label syntax.
- [Images](./images.md) - figure and figcaption output.
- [Markdown Baseline](./markdown.md) - headings and GFM tables.
