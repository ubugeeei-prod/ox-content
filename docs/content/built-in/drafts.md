---
title: Draft, Unlisted, and Scheduled Pages
description: Opt-in frontmatter publish states for production HTML, navigation, search, and sitemaps.
---

# Draft, Unlisted, and Scheduled Pages

When `publishState` is enabled, production builds honor frontmatter publish
fields. Omitted or `false` keeps the current behavior: every Markdown page is
published.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      publishState: true,
    }),
  ],
};
```

`false` or omitted leaves filtering off. `true` enables the defaults. An object
enables the feature and can inject a build-time clock:

```ts
oxContent({
  publishState: {
    now: "2026-08-24T00:00:00Z",
  },
});
```

| Option         | Type                              | Default      |
| -------------- | --------------------------------- | ------------ |
| `publishState` | `boolean` / `PublishStateOptions` | `false`      |
| `now`          | `string` (ISO-8601)               | system clock |

The dev server keeps draft and not-yet-scheduled pages visible so you can
preview them. Production HTML, search, and sitemaps omit those pages.

## Frontmatter

```md
---
title: Work in progress
draft: true
---
```

| Field            | Production result                                       |
| ---------------- | ------------------------------------------------------- |
| `draft: true`    | No HTML, nav, search, or sitemap                        |
| `unlisted: true` | HTML is written; omitted from nav, search, sitemap      |
| `scheduled`      | Unpublished until that instant                          |
| `date`           | Same as `scheduled` when the value is a valid timestamp |
| `expiry`         | Unpublished after that instant                          |

`scheduled` wins over `date` when both are set. Only JSON `true` counts as
`draft` or `unlisted`.

## Timezone and invalid dates

Naive timestamps (`2026-08-24` and `2026-08-24T12:00:00`) are UTC. Offsets such
as `+09:00` or `Z` are honored.

Invalid `scheduled` or `expiry` values unpublish the page. Invalid `date`
values are ignored because `date` is also used as display metadata. An invalid
`now` option falls back to the system clock.

## Related

- [Site Generation](./site-generation.md)
- [Search](./search.md)
- [Sitemap / robots / llms.txt](./site-maps.md)
- [Built-in Features overview](../built-in-features.md)
