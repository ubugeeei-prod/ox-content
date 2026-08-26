---
title: Data Tables
description: Opt-in static tables from csv-table and json-table fences.
---

# Data Tables

`csv-table` and `json-table` fences are opt-in. When enabled, those fences
become a static HTML table. Cell text is escaped. No client JavaScript is
required. Disabled fences stay ordinary code blocks.

| Option               | Type                           | Default   |
| -------------------- | ------------------------------ | --------- |
| `dataTables`         | `boolean` / `DataTableOptions` | `false`   |
| `dataTables.rootDir` | `string`                       | project   |
| `dataTables.missing` | `"error"` / `"warn"`           | `"error"` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      dataTables: true,
    }),
  ],
};
```

## Authoring

This site enables `dataTables`, so the next block is a live table.

```csv-table title="Options"
Option,Type,Default
highlight,boolean,false
```

````md
```csv-table title="Options"
Option,Type,Default
highlight,boolean,false
```
````

- Fence languages are `csv-table` and `json-table`.
- The first CSV row is the header. Quoted fields may contain commas.
- JSON may be an array of objects, an array of arrays, or
  `{ "headers": [...], "rows": [[...]] }`.
- `title="..."` becomes a `<caption>`.
- Fences inside other fences, indented code, or inline code are left alone.

## Import a file

Once inline data works, a fence can load a file. `@/` and `/` resolve from
`rootDir` (the Vite project root unless you set it). Relative paths resolve
from the current Markdown file. `..` cannot leave that root.

````md
```csv-table src="@/content/data/options.csv" title="From CSV"

```
````

```csv-table src="@/content/data/options.csv" title="From CSV"

```

A single path in the fence body is also an import:

````md
```json-table
./options.json
```
````

```json-table
./options.json
```

Missing files use `missing`. The default is `"error"` (reported as a transform
diagnostic). `"warn"` leaves the fence as a code block and does not record an
error. Malformed CSV/JSON always reports an actionable diagnostic.

```ts
oxContent({
  dataTables: {
    rootDir: "docs",
    missing: "warn",
  },
});
```

## Related

- [File Tree](./file-tree.md)
- [File Includes](./includes.md)
- [Markdown Baseline](./markdown.md)
- [Built-in Features overview](../built-in-features.md)
