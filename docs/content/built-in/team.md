---
title: Team / Members Page
description: Opt-in static member cards for a layout: team page.
---

# Team / Members Page

When `ssg.team` is enabled, a Markdown page with `layout: team` renders a
static card grid of people instead of (or around) the page body. Names, roles,
and link labels are escaped. Avatar and link URLs must be `https:` or a
site-relative path that starts with `/` and is not `//`.

The feature is off unless you turn it on. Existing sites stay unchanged. While
it is off, `layout: team` is ignored and the file stays an ordinary page.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        team: {
          members: [
            {
              name: "Ada Lovelace",
              role: "Mathematician",
              avatar: "https://example.com/ada.png",
              links: [{ label: "Website", href: "https://example.com/ada" }],
            },
          ],
        },
      },
    }),
  ],
};
```

`false` or omitted keeps the layout inert. `true` enables an empty member
list. An object enables the feature and supplies `members`.

```md
---
title: Team
layout: team
---

Optional introduction. Safe member cards are rendered above this body.
```

| Option     | Type                      | Default |
| ---------- | ------------------------- | ------- |
| `ssg.team` | `boolean` / `TeamOptions` | `false` |
| `members`  | `TeamMember[]`            | `[]`    |
| `name`     | `string`                  | —       |
| `role`     | `string`                  | —       |
| `avatar`   | `string`                  | —       |
| `links`    | `{ label, href }[]`       | —       |

Rejected avatar and link URLs (`javascript:`, `data:`, `http:`, protocol-
relative `//`) are omitted from the markup. Site-relative paths such as
`/avatars/ada.png` are kept.

Cards use the `.ox-team` class. Bare mode still runs the same URL and escape
rules when `generateHtmlPage` receives the option.

## Related

- [Site Generation](./site-generation.md)
- [Built-in Features overview](../built-in-features.md)
