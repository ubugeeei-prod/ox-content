---
title: Collections
description: Query Markdown files as typed collections with a SQL-like builder, backed by a Rust-generated manifest.
---

# Collections

Collections expose the site's Markdown files as lazily-loaded, queryable data —
for blog indexes, changelogs, "related pages" lists, or any page that needs to
enumerate other pages. The manifest is generated natively in Rust at build
time; queries run client-side against plain data, so filtering and ordering
never load page bodies you did not ask for.

A default `content` collection covering every Markdown file under `srcDir`
exists out of the box — `collections: false` disables the feature.

## Defining Collections

Values in the `collections` record can be a full options object, a glob
string, or an array of globs:

```ts
import { oxContent, defineCollections } from "@ox-content/vite-plugin";

oxContent({
  collections: defineCollections({
    blog: {
      source: "blog/**/*.md",
      include: ["html", "toc"],
    },
    changelog: "changelog/*.md",
    guides: ["guide/**/*.md", "tutorials/**/*.md"],
  }),
});
```

| Option    | Default            | Purpose                                        |
| --------- | ------------------ | ---------------------------------------------- |
| `source`  | all Markdown files | Glob pattern(s) resolved from `srcDir`.        |
| `include` | `[]`               | Extra per-entry fields: `body`, `html`, `toc`. |

By default each entry carries metadata only. `include` opts into heavier
fields per collection: `body` is the raw Markdown, `html` the natively
rendered HTML, and `toc` the parsed table of contents. Numeric route prefixes
such as `1.guide/2.install.md` are stripped from the generated `path`.

## Entry Shape

Each entry is a `CollectionEntry`:

```ts
interface CollectionEntry {
  id: string; // "content/built-in/collections.md"
  collection: string; // "content"
  path: string; // "/built-in/collections"
  stem: string; // "built-in/collections"
  source: string; // source file path relative to srcDir
  extension: string; // ".md"
  title: string; // frontmatter title or first heading
  description?: string;
  frontmatter: Record<string, unknown>;
  body?: string; // include: ["body"]
  html?: string; // include: ["html"]
  toc?: TocEntry[]; // include: ["toc"]
}
```

## Querying

The manifest is exposed through a virtual module with a SQL-flavored query
builder:

```ts
import { queryCollection } from "virtual:ox-content/collections";

const recent = await queryCollection("content")
  .where("path", "LIKE", "/built-in/%")
  .order("title", "ASC")
  .limit(5)
  .all();

const page = await queryCollection("content").path("/getting-started").first();

const total = await queryCollection("content").count();
```

The module also exports `getCollection(name)` (all entries as a plain array)
and `collectionNames`.

### Builder API

| Method                                     | Behavior                                                     |
| ------------------------------------------ | ------------------------------------------------------------ |
| `path(path)`                               | Shorthand for `where("path", "=", path)` with normalization. |
| `select(...fields)`                        | Keep only the named fields on each result.                   |
| `where(field, operator, value?)`           | Add an AND condition.                                        |
| `where(field, value)`                      | Two-argument form means equality.                            |
| `andWhere(q => ...)` / `orWhere(q => ...)` | Grouped conditions joined with AND / OR.                     |
| `order(field, "ASC" \| "DESC")`            | Sort; call repeatedly for multi-key ordering.                |
| `limit(n)` / `skip(n)`                     | Pagination.                                                  |
| `all()` / `first()` / `count()`            | Execute: array, first entry or `null`, or the match count.   |

`field` accepts dot paths into nested data, so frontmatter keys are queryable
directly:

```ts
const drafts = await queryCollection("blog")
  .where("frontmatter.draft", "=", true)
  .orWhere((q) => q.where("frontmatter.date", "IS NULL"))
  .all();
```

### Operators

`=` `==` `!=` `<>` `>` `>=` `<` `<=` `IN` `NOT IN` `BETWEEN` `NOT BETWEEN`
`IS NULL` `IS NOT NULL` `LIKE` `NOT LIKE`

`LIKE` uses SQL wildcards, case-insensitively: `%` matches any run of
characters and `_` matches exactly one. Comparisons are numeric-aware —
numbers compare as numbers, date-like values as dates, and strings with
`localeCompare(..., { numeric: true })`.

## Rendered Example

This site's default `content` collection indexes every documentation page.
Querying it for this section:

```ts
await queryCollection("content")
  .where("path", "LIKE", "/built-in/%")
  .order("path", "ASC")
  .select("path", "title")
  .all();
```

returns entries for the guides in this sidebar group — the same data that
could drive a custom index page:

```json
[
  { "path": "/built-in/code-blocks", "title": "Code Blocks" },
  { "path": "/built-in/collections", "title": "Collections" },
  { "path": "/built-in/embeds", "title": "Embeds" },
  { "path": "/built-in/markdown", "title": "Markdown Baseline" },
  { "path": "/built-in/mermaid", "title": "Mermaid Diagrams" },
  { "path": "/built-in/quality-checks", "title": "Quality Checks" },
  { "path": "/built-in/search", "title": "Search" },
  { "path": "/built-in/site-generation", "title": "Site Generation" },
  { "path": "/built-in/syntax-extensions", "title": "Syntax Extensions" }
]
```

## Related

- [Site Generation](./site-generation.md) — the build that generates the
  manifest.
- [Search](./search.md) — full-text queries instead of structured ones.
