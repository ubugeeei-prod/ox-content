---
title: Previous and Next Links
description: Opt-in previous/next page links generated from sidebar order.
---

# Previous and Next Links

When `ssg.pagination` is enabled, each article gets previous and next links
after the body and before the last-updated line. The order comes from the
sidebar: groups are flattened depth-first, and items without an in-site href
are skipped.

The feature is off unless you turn it on.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        pagination: true,
      },
    }),
  ],
};
```

`false` or omitted keeps the pager off. `true` enables the defaults. An object
also enables the feature.

The first page has no previous link. The last page has no next link. A
single-page sidebar emits nothing. Entry pages skip the pager. Bare mode never
emits pager chrome.

## Frontmatter

Override one side, or hide it:

```md
---
title: Guide
prev:
  text: Back to intro
  link: /intro/
next: false
---
```

| Value                                 | Result                         |
| ------------------------------------- | ------------------------------ |
| omitted                               | Auto neighbor from the sidebar |
| `false`                               | Hide that side                 |
| `{ text, link }` or `{ title, href }` | Replace that side              |

Override hrefs that use `javascript:` or `data:` are dropped.
