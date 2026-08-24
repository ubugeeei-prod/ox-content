---
title: Cards
description: Opt-in card, link-card, and card-grid blocks for overview pages.
---

# Cards

Card, link-card, and card-grid blocks are opt-in. When disabled, the `:::`
forms stay literal.

| Option  | Type                      | Default |
| ------- | ------------------------- | ------- |
| `cards` | `boolean` / `CardOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      cards: true,
    }),
  ],
};
```

## Card

`::: card` becomes `<article class="ox-card">`. Inner Markdown is parsed. The
title comes from a leading heading or from `::: card[Title]`.

```md
::: card

### Install

Copy the package and run the CLI.
:::
```

This site enables `cards`, so the next block is a live card:

::: card

### Install

Copy the package and run the CLI.
:::

## Link card

`::: link-card[Title]{HREF}` becomes `<a class="ox-link-card" href="...">`.
Titles and descriptions are escaped. `javascript:`, `data:`, `vbscript:`, and
protocol-relative `//` hrefs are rejected: the block still renders, but no
anchor is emitted with that href.

```md
::: link-card[Guide]{/getting-started}
Short description
:::
```

::: link-card[Guide]{/getting-started}
Short description
:::

## Card grid

`::: card-grid` wraps inner cards in `<div class="ox-card-grid">`. Lone cards
and link cards also work outside a grid.

```md
::: card-grid
::: card

### Install

Copy the package and run the CLI.
:::
::: link-card[Guide]{/getting-started}
Short description
:::
:::
```

::: card-grid
::: card

### Install

Copy the package and run the CLI.
:::
::: link-card[Guide]{/getting-started}
Short description
:::
:::

Unclosed blocks stay literal and do not swallow the rest of the file. Markers
inside fenced, inline, or indented code are left alone.

## Related

- [Custom Containers](./containers.md)
- [Built-in Features overview](../built-in-features.md)
