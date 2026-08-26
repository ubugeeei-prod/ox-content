---
title: NotByAI Badge
description: Opt-in static authorship disclosure using the official Not By AI badge.
---

# NotByAI Badge

Documentation sites sometimes need to disclose that a page was written by a
human. `<NotByAI />` is an opt-in authorship badge — not a status label.

Status labels such as beta or deprecated belong on
[Inline Badges](./badges.md) (`{badge:tip}`). This feature only emits the
official [Not By AI](https://notbyai.fyi) artwork as static HTML.

| Option    | Type                         | Default |
| --------- | ---------------------------- | ------- |
| `notByAi` | `boolean` / `NotByAiOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      notByAi: true,
    }),
  ],
};
```

`false` or omitted leaves the source unchanged. `true` or an object enables
the badge. There is no client JavaScript and no hydration.

## Authoring

The canonical form is `<NotByAI />`. The compact self-closing form is also
accepted.

```md
This page was written by a person.

<NotByAI />
```

This page was written by a person.

<NotByAI />

The default accessible label is `Written by human, not by AI`. The default
link is `https://notbyai.fyi`. Both can be overridden:

```ts
oxContent({
  notByAi: {
    label: "Written by the docs team",
    href: "https://example.com/authorship",
  },
});
```

Unsafe `href` values (`javascript:`, `data:`, protocol-relative URLs) fall
back to the official Not By AI URL. The label and href are HTML-escaped.

Fenced, indented, and inline code, plus HTML comments, are not rewritten.
Malformed or unclosed tags stay literal. `.md` and `.mdx` emit the same
static markup; `NotByAI` is reserved so MDX does not turn it into an island.

## Light and dark artwork

The badge includes official light and dark SVGs. Built-in SSG CSS switches
them with `prefers-color-scheme` and host scheme classes (`[data-theme]`,
`.dark`, `.light`). Custom hosts import the same sheet:

```css
@import "@ox-content/vite-plugin/styles/not-by-ai.css";
```

See [Component styles](./component-styles.md).

## Migrating from a site preprocessor

If a site already replaces `<NotByAI />` with a placeholder, then swaps in
vendored SVGs after render (the ryoppippi.com pattern), enable `notByAi` and
delete that preprocessor, the SVG imports, the post-render replacement, and
their protection tests.

## Related

- [Inline Badges](./badges.md) — status labels such as beta or required
- [Markdown Baseline](./markdown.md)
- [Built-in Features overview](../built-in-features.md)
