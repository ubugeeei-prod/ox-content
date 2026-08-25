---
title: Breadcrumbs
description: Opt-in breadcrumb trail from the site root through sidebar ancestors.
---

# Breadcrumbs

When `ssg.breadcrumbs` or `theme.breadcrumbs` is enabled, each article gets a
trail from the site root through sidebar ancestors to the current page. The
trail is placed above the article. The current page is not a link.

The feature is off unless you turn it on.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        breadcrumbs: true,
      },
    }),
  ],
};
```

`false` or omitted keeps the trail off. `true` enables the defaults. An object
also enables the feature.

The same flag can live on the theme:

```ts
oxContent({
  ssg: {
    theme: {
      breadcrumbs: true,
    },
  },
});
```

The visible trail is independent of structured data. Enabling breadcrumbs does
not emit JSON-LD. See [JSON-LD](./json-ld.md) to opt in. Entry pages skip the
trail.

## Frontmatter

Hide the trail on one page:

```md
---
title: Landing
breadcrumbs: false
---
```

| Value   | Result                                    |
| ------- | ----------------------------------------- |
| omitted | Follow the site `breadcrumbs` option      |
| `false` | Hide the trail on this page               |
| `true`  | Keep the trail when the site option is on |

Ancestor hrefs that use `javascript:`, `data:`, `vbscript:`, or `//` are not
emitted as links. Bare mode never emits breadcrumb chrome.
