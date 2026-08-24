---
title: Redirects and aliases
description: Opt-in static HTML redirects from frontmatter aliases and a rewrite map.
---

# Redirects and aliases

When `redirects` is enabled, the SSG build writes a small static HTML page at
each old path. The page uses a meta refresh and a canonical link so inbound
URLs keep working after a rename.

The feature is off unless you turn it on. Existing sites stay unchanged. This
documentation site does not enable a live redirect.

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

`false` or omitted writes nothing. `true` or `{}` enables the defaults. A path
map enables the feature with those rewrites. An options object enables the
feature and overrides only the fields you set:

```ts
oxContent({
  redirects: {
    map: {
      "/old-guide": "/guide",
    },
  },
});
```

| Option         | Type                                      | Default |
| -------------- | ----------------------------------------- | ------- |
| `redirects`    | `boolean` / path map / `RedirectsOptions` | `false` |
| `map`          | `Record<string, string>`                  | `{}`    |
| `netlify`      | `boolean`                                 | `false` |
| `writeNetlify` | alias for `netlify`                       | `false` |

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

## Safety

Destinations must be same-origin paths: they start with `/` and must not start
with `//`. `javascript:`, `data:`, and absolute URLs such as `https://evil`
are ignored.

A destination that is allowed but contains markup characters is HTML-escaped
in the refresh URL, canonical href, and visible link.

## Trailing slashes and overlaps

`/old` and `/old/` are the same source after a trailing slash is stripped
(except `/` itself). Destinations are normalized the same way.

When two rules share a normalized source, **the last rule wins**. Frontmatter
aliases and `redirect` are applied first. The config `map` is applied last, so
an explicit map entry overrides a page alias for the same old path.

## Host `_redirects`

Set `netlify: true` or `writeNetlify: true` to also write a Netlify-style
`_redirects` file (`/old /guide 301`). The HTML pages are still written.

## Related

- [Site Generation](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
