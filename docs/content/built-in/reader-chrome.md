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
is not copied at build time, and annotated fences prefer `data-ox-code-source`
so the copied value matches the authored block. Page-level Copy as Markdown is
a separate opt-in on [`ssg.markdownSource.copy`](./markdown-source.md).

The built-in SSG stylesheet and
`@ox-content/vite-plugin/styles/reader-chrome.css` expose stable copy-control
sizing tokens:

| Token                    | Default     | Effect                                   |
| ------------------------ | ----------- | ---------------------------------------- |
| `--ox-copy-control-size` | `1.75rem`   | Square button size                       |
| `--ox-copy-icon-size`    | `0.8125rem` | Copy and copied-state glyph size         |
| `--ox-copy-inset`        | `0.5rem`    | Block-start and inline-end button offset |

Set them on `.content` or another reader root instead of overriding internal
`.ox-copy` selectors:

```css
.content {
  --ox-copy-icon-size: 1rem;
}
```

The code title and inline-end gutter reservation follow customized control and
inset sizes.

Outbound icons skip relative, hash, `mailto:`, and `tel:` links. Links inside
fenced blocks or inline code spans are left alone. `javascript:`, `data:`, and
`vbscript:` hrefs are not given a live action.

The back-to-top control respects `prefers-reduced-motion`. Entry pages skip it.

Bare mode and `ssg.render` can use the same code-copy and outbound-link chrome
without switching to the built-in theme:

```ts
oxContent({
  ssg: {
    bare: true,
    readerChrome: { copy: true, externalLinks: false, backToTop: false },
  },
});
```

For a host that renders Markdown outside `buildSsg`, compose the public helper,
stylesheet, and browser initializer:

```ts
import {
  applyReaderChromeHtml,
  renderReaderChromeAttributes,
} from "@ox-content/vite-plugin/reader-chrome";
import { initReaderChrome } from "@ox-content/vite-plugin/reader-chrome/client";
import "@ox-content/vite-plugin/styles/reader-chrome.css";

const chrome = { copy: true, externalLinks: false, backToTop: false };
const html = `<article class="content"${renderReaderChromeAttributes(chrome)}>${applyReaderChromeHtml(
  rendered.html,
  chrome,
)}</article>`;

initReaderChrome(document);
```
