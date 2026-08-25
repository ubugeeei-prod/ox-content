---
title: Blog
description: Opt-in paginated index, authors, tags, reading time, and archive.
---

# Blog

When `blog` is enabled (top-level or `ssg.blog`), the SSG build adds a blog
layout on top of collections:

- A paginated index at `/blog/` (`/blog/page/2/` when a second page is needed)
- Author(s) and reading time on each post
- Tag pages at `/blog/tags/{tag}/`
- Yearly and monthly archive at `/blog/archive/`, `/blog/archive/{yyyy}/`, and
  `/blog/archive/{yyyy}/{mm}/`
- Optional external RSS / Atom sources merged into that same index

The feature is off unless you turn it on. Existing sites stay unchanged.
Tags and archive are implemented here; they do not wait on taxonomies.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      blog: true,
    }),
  ],
};
```

`ssg.blog` is accepted as well. When both are set, the top-level `blog`
option wins.

```ts
oxContent({
  ssg: {
    blog: true,
  },
});
```

`false` or omitted keeps the extra pages and post chrome off. `true` enables
the defaults. An object enables the feature and overrides only the fields you
set:

```ts
oxContent({
  blog: {
    collection: "posts",
    pageSize: 5,
    authors: {
      ada: {
        name: "Ada Lovelace",
        bio: "Mathematician",
        url: "https://example.com/ada",
      },
    },
  },
});
```

| Option       | Type                           | Default                                                      |
| ------------ | ------------------------------ | ------------------------------------------------------------ |
| `blog`       | `boolean` / `BlogOptions`      | `false`                                                      |
| `ssg.blog`   | `boolean` / `BlogOptions`      | `false`                                                      |
| `collection` | `string`                       | collection named `blog`, else the only configured collection |
| `authors`    | `Record<string, BlogAuthor>`   | `{}`                                                         |
| `pageSize`   | `number`                       | `10`                                                         |
| `feeds`      | `(string \| BlogFeedSource)[]` | `[]` (no fetch)                                              |

## Collection

Posts come from a named collection.

1. An explicit `collection` always wins.
2. Else a collection named `blog` is used.
3. Else the only configured collection is used.
4. If several collections exist and none is named `blog`, set `collection` or
   nothing extra is written (the build continues with a warning).

When collections are disabled, every listed page is treated as a post.

## Pagination

The index lists posts newest first. The sort key is frontmatter `date`, then
href when dates tie. `pageSize` posts appear on `/blog/`. Further pages use
`/blog/page/2/`, `/blog/page/3/`, and so on. Page 1 never uses `/blog/page/1/`.
Pager links are labeled Newer and Older.

`pageSize` values below 1 fall back to 10.

## Authors

Authors come from a config map plus frontmatter `author` and/or `authors`.
A string or a string array is accepted. Each value is looked up in
`blog.authors`. A missing key becomes the display name.

```md
---
title: Notes
date: 2024-03-01
author: ada
authors:
  - grace
---
```

Names and bios are HTML-escaped. `url` must be `https:` or a site-relative
path that starts with `/` and is not `//`. Rejected URLs (`javascript:`,
`data:`, `http:`, protocol-relative `//`) are omitted; the name still
renders as plain text.

## Reading time

Reading time is deterministic: the same markdown always yields the same
integer number of minutes. The formula is:

1. Drop YAML frontmatter (`---` … `---`).
2. Drop fenced code blocks (` ``` ` … ` ``` `; an unclosed fence runs to
   end of file) and inline code spans (`` `…` ``).
3. Count Latin words: `[A-Za-z0-9]+` sequences. An apostrophe may join two
   parts into one word (`don't` is one word).
4. Count CJK characters: Hiragana, Katakana, CJK Unified Ideographs
   (including Extension A and Compatibility Ideographs), and Hangul.
5. `minutes = ceil(latin_words / 200 + cjk_chars / 500)`
6. Empty input after stripping is `0`. Any remaining text is at least 1
   minute.

Mentions of tags or authors inside fences and code spans do not affect
reading time. The value is prepended as `N min read` in `.ox-blog-meta`.

## Tags

Terms come from frontmatter `tags` only — a string or a string array.
Mentions of `tags` inside fenced or inline code do not create pages.

```md
---
title: Install
date: 2024-01-15
tags:
  - rust
  - napi
---
```

Each term becomes `/blog/tags/{slug}/`. Slugs are stable and restricted to
`[a-z0-9-]`. Hostile values such as `javascript:`, `../`, or `//evil.com`
are dropped from hrefs. Every label, title, and href is HTML-escaped.

## Archive

Archive pages use frontmatter `date` (`YYYY-MM-DD` or ISO-8601). The year
and month are taken from the parsed UTC civil date so the same `date` always
maps to the same path:

- `/blog/archive/` — years that have at least one dated post
- `/blog/archive/{yyyy}/` — months and posts in that year
- `/blog/archive/{yyyy}/{mm}/` — posts in that month (`mm` is zero-padded)

Posts without a parseable `date` stay on the index and tag pages only.

## External feeds

`feeds` is off unless you set a non-empty array. The build fetches only those
configured URLs. Links inside Markdown or HTML are never requested.

```ts
oxContent({
  blog: {
    feeds: [
      "https://example.com/rss.xml",
      {
        url: "https://example.com/atom.xml",
        language: "ja",
        author: "ada",
        onError: "warn",
      },
    ],
  },
});
```

| Field      | Type               | Default | Role                                                          |
| ---------- | ------------------ | ------- | ------------------------------------------------------------- |
| `url`      | `string`           | —       | Absolute `https:` RSS or Atom URL                             |
| `language` | `string`           | —       | Default language when an item omits one                       |
| `author`   | `string`           | —       | Default author when an item omits one                         |
| `onError`  | `"warn"`/`"error"` | `warn`  | Skip the source, or fail the build after other sources finish |

A string entry is `{ url, onError: "warn" }`. Each unique URL is fetched once
per build, not per page. The request uses a timeout, a redirect hop limit, a
response size cap, and `https:`-only public hosts. Loopback, private, and
link-local targets are rejected after DNS. HTML pages are not parsed.

A failed source in `warn` mode is skipped; successful sources still merge. One
bad source does not drop the rest of the blog. `onError: "error"` fails the
build after the remaining sources finish.

Items keep title, canonical `https:` link, publication date, stable id,
language, and summary when present. They merge with local posts, newest first,
then href. Duplicates match a canonical URL or an explicit stable id. The
local post wins.

External items carry an `external` marker (`class="ox-blog-external"`,
`rel="external"`). Themes must keep the remote URL and must not rewrite the
item to a local route.

External items are **not** written to generated RSS, Atom, or JSON feeds.
There is no include switch in this release.

## Drafts and unlisted posts

Frontmatter `draft: true` and `unlisted: true` are omitted from the index,
tag pages, archive, and post chrome, even when `publishState` is off. The
source HTML may still be written when publish-state filtering is off.

When `publishState` is on, draft, unlisted, and scheduled pages follow that
feature's listed-page rules as well.

## Related

- [Collections](./collections.md)
- [Draft / unlisted / scheduled](./drafts.md)
- [RSS / Atom / JSON feeds](./feeds.md)
- [Site Generation](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
