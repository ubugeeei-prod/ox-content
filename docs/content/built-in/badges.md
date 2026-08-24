---
title: Inline Badges
description: Opt-in status labels next to headings or in prose.
---

# Inline Badges

Guides often need a small status label next to a heading or in a sentence —
beta, required, deprecated. `{badge:variant}` markup is opt-in and off by
default.

| Option   | Type                       | Default |
| -------- | -------------------------- | ------- |
| `badges` | `boolean` / `BadgeOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      badges: true,
    }),
  ],
};
```

`false` or omitted leaves the source unchanged. `true` or an object enables
the built-in variants.

## Authoring

The form is `{badge:VARIANT}TEXT{/badge}`. `VARIANT` is lowercase and
case-sensitive.

{badge:tip}Beta{/badge} {badge:note}Note{/badge} {badge:info}Info{/badge}
{badge:warning}Warning{/badge} {badge:danger}Danger{/badge}
{badge:success}Stable{/badge} {badge:deprecated}Deprecated{/badge}
{badge:required}Required{/badge}

```md
API {badge:tip}Beta{/badge} — the `token` field is {badge:required}required{/badge}.
```

API {badge:tip}Beta{/badge} — the `token` field is {badge:required}required{/badge}.

Allowed variants: `tip`, `note`, `info`, `warning`, `danger`, `success`,
`deprecated`, and `required`.

Unknown, uppercase, empty, or unclosed tags stay literal. Badge text is
HTML-escaped; fenced, indented, and inline code are not rewritten.

```md
`{badge:tip}ignored{/badge}`
```

`{badge:tip}ignored{/badge}`

## Related

- [Custom Containers](./containers.md)
- [Built-in Features overview](../built-in-features.md)
