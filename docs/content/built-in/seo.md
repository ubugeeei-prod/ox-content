---
title: SEO
description: Built-in canonical, robots, hreflang, Open Graph, and head validation on the page-head API.
---

# SEO

The [page-head](./page-head.md) resolver can emit documented SEO tags. Nothing
is invented: no author, publisher, URL, or indexability unless you set it.

## What turns a tag on

| Tag                          | When                                                                                           |
| ---------------------------- | ---------------------------------------------------------------------------------------------- |
| `<title>`, OG/Twitter title  | Always on themed pages. Bare pages always have `<title>`.                                      |
| `description` + OG/Twitter   | Page `description` frontmatter.                                                                |
| `og:image` / `twitter:image` | `ssg.ogImage` or a generated OG image.                                                         |
| `canonical` / `og:url`       | `ssg.siteUrl` (themed) or the computed bare `canonicalUrl`. Frontmatter `canonical` overrides. |
| `og:site_name`               | Bare pages when `siteName` is set. Themed pages do not add it unless you pass a custom meta.   |
| `robots`                     | Frontmatter `robots` only.                                                                     |
| `hreflang` alternates        | `locale_paths` from i18n, when an absolute URL can be built.                                   |

Themed pages without `ssg.siteUrl` stay as they were: no canonical, no
`og:url`, no `hreflang`. Setting `siteUrl` for sitemaps or JSON-LD also
enables those tags. Relative locale hrefs need `siteUrl` to become absolute.

```yaml
---
title: Guide
description: How it works
robots: noindex, nofollow
canonical: https://example.com/guide/
---
```

```ts
oxContent({
  ssg: {
    siteName: "Docs",
    siteUrl: "https://example.com",
    headValidation: "warn",
  },
  i18n: {
    locales: [
      { code: "en", name: "English" },
      { code: "ja", name: "日本語" },
    ],
  },
});
```

## Validation

`ssg.headValidation`:

| Value             | Effect                                               |
| ----------------- | ---------------------------------------------------- |
| omitted / `false` | Drop invalid values silently. Default.               |
| `warn`            | Keep valid tags and log findings.                    |
| `strict`          | Fail the build on unsafe URLs or invalid `hreflang`. |

Invalid custom descriptors are dropped in every mode. `strict` is for CI.

`renderHead({ validation: "strict", ... })` returns the same findings in
`diagnostics` for custom hosts.

## Deterministic conflicts

Built-ins and custom descriptors are deduped by their effective SEO identity:
`description`, `robots`, `og:*`, `twitter:*`, canonical links, and matching
`hreflang` alternates produce one tag. If a later custom descriptor conflicts
with an earlier SEO tag, `warn` logs a non-fatal diagnostic and `strict` returns
the same diagnostic without failing the build. `strict` still fails only unsafe
URLs, invalid `hreflang`, and invalid JSON-LD.

`ssg.siteUrl` must be a safe absolute `http(s)` URL for feeds and crawl
manifests. Named feeds are preflighted before writing; duplicate feed output
paths or paths that escape the output directory skip feed generation with a
warning instead of silently overwriting files.

## JSON-LD variants

`ssg.jsonLd.type` can be `TechArticle` (default), `BlogPosting`, or `WebPage`.
`ssg.jsonLd.graph` appends extra `@graph` nodes. Unknown types fall back to
`TechArticle`. See [JSON-LD](./json-ld.md).

## Related

- [Page head](./page-head.md)
- [JSON-LD](./json-ld.md)
- [Locale Switcher](./locale-switcher.md)
- [Site Generation](./site-generation.md)
- Tracking: [#820](https://github.com/ubugeeei-prod/ox-content/issues/820)
