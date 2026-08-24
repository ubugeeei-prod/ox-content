---
title: Header Nav, Announcement, and Page Chrome
description: Opt-in header navigation, an announcement bar, and per-page chrome flags.
---

# Header Nav, Announcement, and Page Chrome

The default theme ships a header title, search, and theme toggle. Header
navigation, an announcement bar, and per-page chrome flags stay **off** until
you opt in. Existing sites do not change unless they set the new options.

## Header nav

Set `theme.nav` to an array of `{ text, link }` items or `{ text, items }`
dropdowns. `text` may be a string or a locale map
(`{ en: "Guide", ja: "ガイド" }`); the current page locale is used when
present:

```ts
import { oxContent, defineTheme } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        theme: defineTheme({
          nav: [
            { text: "Guide", link: "/guide/" },
            {
              text: "API",
              items: [
                { text: "SSG", link: "/api/ssg/" },
                { text: "Search", link: "/api/search/" },
              ],
            },
          ],
        }),
      },
    }),
  ],
};
```

Labels are escaped. Items whose `link` uses `javascript:`, `data:`,
`vbscript:`, or a protocol-relative `//` href are omitted.

Dropdowns use a `button` with `aria-expanded` and `aria-haspopup`. Escape
closes an open menu. On small viewports the list scrolls horizontally so the
page does not overflow.

## Announcement bar

Set `theme.announcement` to show a bar above the header:

```ts
oxContent({
  ssg: {
    theme: defineTheme({
      announcement: {
        text: "Ox Content 3 is in progress.",
        link: "/v3-roadmap/",
        dismissKey: "v3-wip",
      },
    }),
  },
});
```

| Field        | Required | Effect                                                                  |
| ------------ | -------- | ----------------------------------------------------------------------- |
| `text`       | yes      | Escaped. There is no raw HTML slot.                                     |
| `link`       | no       | `https:` or same-origin only. Other schemes are dropped.                |
| `dismissKey` | no       | Best-effort `localStorage` key. Invalid keys still render a static bar. |

## Per-page chrome

`ssg.pageChrome` is off by default. `true` or `{}` enables reading these
frontmatter flags. Omitted flags keep the current layout. `false` hides that
region:

```md
---
title: Landing
sidebar: false
outline: false
footer: false
navbar: false
lastUpdated: false
editLink: false
---
```

`aside: false` is an alias for `outline: false`. When `pageChrome` is off,
these flags are ignored so existing frontmatter cannot change the shell.

Bare mode never emits header nav, the announcement bar, or page-chrome
classes.
