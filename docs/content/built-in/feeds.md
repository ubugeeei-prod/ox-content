---
title: RSS, Atom, and JSON feeds
description: Opt-in collection feeds written next to generated HTML.
---

# RSS, Atom, and JSON feeds

When `feeds` is enabled and `ssg.siteUrl` is set, the SSG build writes
machine-readable feeds from a named collection:

- `feed.xml` — RSS 2.0
- `atom.xml` — Atom 1.0
- `feed.json` — JSON Feed 1.1

The feature is off unless you turn it on. Existing sites stay unchanged.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      feeds: true,
      ssg: {
        siteUrl: "https://example.com",
      },
    }),
  ],
};
```

`false` or omitted keeps the files off. `true` enables the defaults: all three
formats, the `content` collection (or the first configured collection), and a
20-item limit. A single object is one default feed and overrides only the
fields you set:

```ts
oxContent({
  feeds: {
    formats: ["rss", "json"],
    collection: "blog",
    limit: 10,
    path: "/feeds",
  },
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

A named record or array writes multiple feeds. Each channel can set its own
collection, path, formats, and metadata:

```ts
oxContent({
  feeds: {
    blog: {
      formats: ["rss"],
      collection: "blog",
      path: "/",
      title: "blog | example.com",
      description: "Technical articles",
      language: "en",
      image: "https://example.com/icon.png",
      favicon: "https://example.com/icon.png",
      copyright: "© 2026 example.com",
    },
    media: {
      formats: ["rss"],
      collection: "media",
      path: "/works/media",
      title: "Media | example.com",
      language: "ja",
    },
  },
  ssg: {
    siteUrl: "https://example.com",
  },
});
```

## Programmatic items

A channel can set `items` instead of `collection` when the feed source is a JSON
file, database result, or another curated build-time source. The resolver runs
during SSG and may return a promise:

```ts
import media from "./src/contents/external-rss/media.json";

oxContent({
  feeds: {
    media: {
      formats: ["rss"],
      path: "/works/media",
      title: "Media | ryoppippi.com",
      items: async () =>
        media
          .filter((item) => !item.playlist)
          .map((item) => ({
            title: item.title,
            url: item.link,
            id: `media:${item.link}`,
            date: item.pubDate,
            description: `${item.kind === "podcast" ? "Podcast" : "YouTube"} | ${item.title}`,
            author: { name: "ryoppippi", url: "https://ryoppippi.com" },
            language: item.lang,
          })),
    },
  },
  ssg: {
    siteUrl: "https://ryoppippi.com",
  },
});
```

Set either `collection` or `items` on a channel. Supplying both is rejected.
Programmatic items support `title`, `url` or `loc`, optional `id`, `date`,
`description`, `content`, `author` / `authors`, `image`, `attachments`, and
`language`. The RSS, Atom, and JSON Feed renderers emit the fields that each
format supports.

| Option        | Type                                        | Default                              |
| ------------- | ------------------------------------------- | ------------------------------------ |
| `feeds`       | `boolean` / one feed / named record / array | `false`                              |
| `formats`     | `("rss" \| "atom" \| "json")[]`             | `["rss", "atom", "json"]`            |
| `collection`  | `string`                                    | `content`, else the first collection |
| `items`       | `FeedItemInput[]` / async resolver          | omitted                              |
| `limit`       | `number`                                    | `20`                                 |
| `path`        | `string`                                    | `/` (site root)                      |
| `title`       | `string`                                    | SSG site name                        |
| `description` | `string`                                    | SSG site description                 |
| `language`    | `string`                                    | omitted                              |
| `image`       | `string`                                    | omitted                              |
| `favicon`     | `string`                                    | omitted                              |
| `copyright`   | `string`                                    | omitted                              |

`path` is the site-relative directory for the generated files. `/feeds` writes
`feeds/feed.xml`, `feeds/atom.xml`, and `feeds/feed.json`. Channel `title`,
`description`, `language`, `image`, `favicon`, and `copyright` override the
site defaults where the format has a matching field (JSON Feed has no
copyright).

Items are sorted newest first. The sort key is frontmatter `date`, then
`lastUpdated` when `date` is missing. Entries with `draft: true` are omitted.

If `feeds` is enabled without `ssg.siteUrl`, no files are written. The build
continues and emits a warning.

Titles and descriptions are escaped so they cannot break out of XML or JSON.

## Blog index items

External posts aggregated by [Blog](./blog.md) `blog.feeds` stay on the blog
index only. They are omitted from these generated files. There is no include
switch in this release.

## Related

- [SSG output primitives](./ssg-output.md)
- [Collections](./collections.md)
- [Blog](./blog.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Site Generation](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
