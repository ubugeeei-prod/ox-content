---
title: Taxonomies and related pages
description: Opt-in tag and category term pages with related-page lists.
---

# Taxonomies and related pages

When `taxonomies` is enabled, the SSG build reads terms from page frontmatter
and writes:

- Term list pages such as `/tags/index.html` and `/categories/index.html`
- Per-term pages such as `/tags/rust/index.html`
- A related-pages block on source pages that share at least one term

The feature is off unless you turn it on. Existing sites stay unchanged.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      taxonomies: true,
    }),
  ],
};
```

`false` or omitted keeps term pages and related lists off. `true` enables the
defaults: taxonomies `tags` and `categories`, and a related-page limit of 5.
An object enables the feature and overrides only the fields you set:

```ts
oxContent({
  taxonomies: {
    taxonomies: ["topics"],
    relatedLimit: 3,
  },
});
```

| Option         | Type                            | Default |
| -------------- | ------------------------------- | ------- |
| `taxonomies`   | `boolean` / `TaxonomiesOptions` | `false` |
| `relatedLimit` | `number`                        | `5`     |

The object may also set `taxonomies` to replace the default keys
`tags` and `categories`.

Terms come from frontmatter only. A string or a string array is accepted.
Mentions of `tags` or `categories` inside fenced or inline code do not create
terms.

```md
---
title: Install
tags:
  - rust
  - napi
categories: guide
---
```

`categories: guide` is a single string term. `tags` above is a string array.

Term slugs are stable and restricted to `[a-z0-9-]`. Hostile values such as
`javascript:`, `../`, or `//evil.com` are dropped from hrefs. Every term,
title, and href is HTML-escaped.

When `publishState` is on, draft, unlisted, and scheduled pages do not appear
on term pages or in related lists.

## Related

- [Draft / unlisted / scheduled](./drafts.md)
- [Collections](./collections.md)
- [Site Generation](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
