---
title: Reader Chrome
description: Opt-in copy buttons, outbound-link icons, and a back-to-top control.
---

# Reader Chrome

When `ssg.readerChrome` is enabled, themed pages get three small reading
controls:

- a **Copy** button on fenced code blocks
- an icon and `rel="noopener noreferrer"` on outbound `http(s)` links
- a **Back to top** control that appears after the page is scrolled

The feature is off unless you turn it on. Disabled pages emit no extra markup
or JavaScript.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      ssg: {
        readerChrome: true,
      },
    }),
  ],
};
```

`false` or omitted keeps the chrome off. `true` enables the defaults. An object
enables the feature and can turn one control off:

```ts
oxContent({
  ssg: {
    readerChrome: { copy: false },
  },
});
```

| Field           | Default | Effect                               |
| --------------- | ------- | ------------------------------------ |
| `copy`          | `true`  | Copy button on fenced `<pre>` blocks |
| `externalLinks` | `true`  | Icon and `rel` on outbound links     |
| `backToTop`     | `true`  | Back-to-top button after scroll      |

Copy uses the browser clipboard when the reader clicks the button. Fence text
is not copied at build time.

Outbound icons skip relative, hash, `mailto:`, and `tel:` links. Links inside
fenced blocks or inline code spans are left alone. `javascript:`, `data:`, and
`vbscript:` hrefs are not given a live action.

The back-to-top control respects `prefers-reduced-motion`. Entry pages skip
it. Bare mode never emits reader chrome.
