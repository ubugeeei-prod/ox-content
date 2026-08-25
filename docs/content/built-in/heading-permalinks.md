---
title: Heading Permalinks
description: Opt-in visible # links on headings that reuse the generated heading id.
---

# Heading Permalinks

Headings already get a stable `id`. This option adds a discoverable permalink
control next to the heading so readers can copy or open `#section` without
editing the URL.

The feature is off unless you turn it on. Disabled output is unchanged.

| Option                   | Type                                   | Default   |
| ------------------------ | -------------------------------------- | --------- |
| `headingPermalinks`      | `boolean` / `HeadingPermalinksOptions` | `false`   |
| `theme.headingPermalink` | `"hover"` / `"always"`                 | `"hover"` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      headingPermalinks: true,
    }),
  ],
};
```

`false` or omitted keeps the HTML byte-stable. `true` or `{}` enables the
defaults. An object can set `enabled: false` while keeping the option present.

## Markup contract

The renderer reuses the exact generated heading id. It does not slugify
again. Duplicate and Unicode ids match the outline.

```html
<h2 id="hello-world">
  Hello World<a class="header-anchor" href="#hello-world" aria-label='Permalink to "Hello World"'
    >#</a
  >
</h2>
```

- The control is a real `<a href="#id">` in the HTML. It works with CSS or
  JavaScript disabled.
- The accessible name includes the heading text. Empty headings use
  `Permalink to this section`.
- Headings that already contain `class="header-anchor"` or a `#` link to the
  same id do not get a second marker.
- Explicit `{#custom-id}` attrs rewrite the permalink `href` to that id.

This site enables `headingPermalinks`, so the headings on this page include
the control.

## Visibility

`theme.headingPermalink` changes only CSS. The heading HTML stays the same.

```ts
oxContent({
  headingPermalinks: true,
  ssg: {
    theme: {
      headingPermalink: "always",
    },
  },
});
```

| Value    | Presentation                                         |
| -------- | ---------------------------------------------------- |
| `hover`  | Visible on hover / `:focus-visible`; always on touch |
| `always` | Always visible                                       |

Hover reveal is CSS-only. There is no client JavaScript, hydration, or layout
measurement. `prefers-reduced-motion` disables the opacity transition. Spacing
uses logical properties so RTL layout stays aligned.

## Related

- [Markdown Baseline](./markdown.md) — heading ids and the outline.
- [Theming](../theming.md) — `headingPermalink` on the default theme.
- [Built-in Features overview](../built-in-features.md)
