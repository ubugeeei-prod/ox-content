---
title: Redirects and aliases
description: Opt-in static HTML redirects from frontmatter aliases and a rewrite map.
aliases:
  - /built-in/aliases
---

# Redirects and aliases

When `redirects` is enabled, the SSG build writes a small static HTML page at
each old path. The page uses a meta refresh plus a canonical link so inbound
URLs keep working after a rename. That works on any static host.

This page also declares `aliases: [/built-in/aliases]`, so the docs site
itself ships a live redirect for that old path.

The feature is off unless you turn it on. Existing sites stay unchanged.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      redirects: true,
    }),
  ],
};
```

`false` or omitted writes nothing. `true` enables the defaults. An object
enables the feature and overrides only the fields you set:

```ts
oxContent({
  redirects: {
    map: {
      "/old-guide": "/guide",
    },
  },
});
```

A path map such as `{ "/old-guide": "/guide" }` can be passed in place of the
options object and enables the feature with that map.

| Option          | Type                                      | Default |
| --------------- | ----------------------------------------- | ------- |
| `redirects`     | `boolean` / path map / `RedirectsOptions` | `false` |
| `map`           | `Record<string, string>`                  | `{}`    |
| `netlify`       | `boolean`                                 | `false` |
| `headers`       | `boolean`                                 | `false` |
| `json`          | `boolean`                                 | `false` |
| `allowExternal` | `boolean`                                 | `false` |

## Frontmatter

On a page, `aliases` and `redirect` name **old** paths. Each one emits a
redirect page that points at the current page path:

```md
---
title: Guide
aliases:
  - /old
  - /legacy
redirect: /retired
---
```

`/old`, `/legacy`, and `/retired` each become `old/index.html`,
`legacy/index.html`, and `retired/index.html` with a refresh to `/guide`.

Redirects are not a Markdown syntax. Text inside fences or code spans is
ignored because only frontmatter and the config map are read.

## Safety

Destinations must be same-origin paths: they start with `/` and must not start
with `//`. `javascript:`, `data:`, and absolute URLs such as `https://evil`
are ignored unless `allowExternal` is set. Even then, only `http://` and
`https://` destinations are accepted.

A destination that is allowed but contains markup characters is HTML-escaped
in the refresh URL, canonical href, and visible link.

A source that matches a real published page is skipped so a redirect cannot
overwrite content.

## Trailing slashes and overlaps

`/old` and `/old/` are the same source after a trailing slash is stripped
(except `/` itself). Destinations are normalized the same way.

When two rules share a normalized source, **the last rule wins**. Frontmatter
aliases and `redirect` are applied first. The config `map` is applied last, so
an explicit map entry overrides a page alias for the same old path.

## Host files

Set `netlify: true` to also write a `_redirects` file
(`/old /guide 301`). Set `headers: true` to write `_headers` with a
`Location` line per source. Set `json: true` to write `redirects.json`.
Normal (non-wildcard) sources still get HTML fallback pages.

Sources that contain `*` are host-rule syntax (Netlify, Cloudflare Pages),
not a literal URL segment. They still appear in `_redirects`, `_headers`,
and `redirects.json` when those flags are on, but the SSG does not write
a static HTML file such as `talks*/index.html`.

```ts
oxContent({
  redirects: {
    map: {
      "/talks*": "/works/talks",
      "/old-guide": "/guide",
    },
    netlify: true,
  },
});
```

That map writes `/talks* /works/talks 301` to `_redirects` and an HTML
page only for `/old-guide`.

## Drafts

Draft, unlisted, and scheduled pages are out of scope on this feature. A
later draft option may omit aliases on unpublished pages.

## Related

- [Site Generation](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
