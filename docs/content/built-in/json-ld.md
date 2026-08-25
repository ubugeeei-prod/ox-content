---
title: JSON-LD structured data
description: Opt-in TechArticle, WebSite, and BreadcrumbList JSON-LD in the page head.
---

# JSON-LD structured data

When `ssg.jsonLd` is enabled, themed pages emit a
`<script type="application/ld+json">` block in `<head>` after the existing
Open Graph tags. The payload describes the page as a `TechArticle` that is
part of a `WebSite`. If a visible breadcrumb trail exists and JSON-LD
breadcrumbs are not turned off, a `BreadcrumbList` is included too.

The feature is off unless you turn it on. Existing sites stay unchanged.
Enabling breadcrumbs alone does **not** emit JSON-LD. See
[Breadcrumbs](./breadcrumbs.md) and
[#696](https://github.com/ubugeeei-prod/ox-content/issues/696).

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        jsonLd: true,
        siteUrl: "https://example.com",
      },
    }),
  ],
};
```

`false` or omitted keeps structured data off. `true` enables the defaults. An
object also enables the feature and can hide `BreadcrumbList` or supply a
publisher.

```ts
oxContent({
  ssg: {
    jsonLd: {
      breadcrumbs: true,
      publisher: {
        name: "Ox Content",
        url: "https://oxc.rs",
      },
    },
    breadcrumbs: true,
    siteUrl: "https://example.com",
  },
});
```

| Field         | Default | Effect                                                                          |
| ------------- | ------- | ------------------------------------------------------------------------------- |
| `breadcrumbs` | `true`  | Emit `BreadcrumbList` only when a visible trail exists. Set `false` to hide it. |
| `publisher`   | omitted | Optional `{ name?, url? }`. Missing fields are not invented.                    |

## What is emitted

The script is a single `@graph` document:

| `@type`          | When                                                                  | Typical fields                                                                                                |
| ---------------- | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `WebSite`        | `jsonLd` is on                                                        | `name` from `siteName`; `url` / `@id` when `siteUrl` is set                                                   |
| `TechArticle`    | `jsonLd` is on                                                        | `headline`, `description`; `url` / `@id` / `isPartOf` when `siteUrl` is set; `publisher` only when configured |
| `BreadcrumbList` | Visible breadcrumbs exist **and** `jsonLd.breadcrumbs` is not `false` | `itemListElement` with `position`, `name`, and `item` when an absolute URL can be built                       |

`siteUrl` is required for `@id` and `url`. If it is missing, those absolute
URL fields are omitted. The build does not invent a host, a logo, or a
publisher.

## Breadcrumbs

`ssg.breadcrumbs` / `theme.breadcrumbs` control the visible trail.
`ssg.jsonLd.breadcrumbs` only controls whether that trail is also described
as `BreadcrumbList`.

| Visible trail | `jsonLd.breadcrumbs` | `BreadcrumbList` |
| ------------- | -------------------- | ---------------- |
| off           | `true` (default)     | omitted          |
| on            | `true` (default)     | emitted          |
| on            | `false`              | omitted          |

Entry pages skip the visible trail, so they also skip `BreadcrumbList`.

## Safety

Every string is JSON-encoded. `<`, `>`, and `&` are written as JSON `\u`
escapes so a hostile title cannot break out of the `<script>` tag.

`javascript:`, `data:`, `vbscript:`, and protocol-relative `//` URLs are
rejected for publisher and breadcrumb `item` URLs. Only `http:` / `https:`
absolute URLs, or site-relative paths that can be resolved with `siteUrl`,
are emitted.

Bare mode never emits JSON-LD.

## Related

- [Breadcrumbs](./breadcrumbs.md)
- [Site Generation](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
- Tracking issue: [#696](https://github.com/ubugeeei-prod/ox-content/issues/696)
