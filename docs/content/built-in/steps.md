---
title: Step Lists
description: Opt-in ::: steps wrappers that restyle ordered lists for tutorials.
---

# Step Lists

Tutorial steps stay visually distinct from ordinary ordered lists. The feature
is opt-in: omitted or `false` leaves `::: steps` as literal text.

| Option  | Type                       | Default |
| ------- | -------------------------- | ------- |
| `steps` | `boolean` / `StepsOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      steps: true,
    }),
  ],
};
```

Passing `{}` also enables the defaults.

## Authoring

Wrap an ordered list in `::: steps`. Nested Markdown — fences, emphasis, and
nested lists — still renders inside each item.

::: steps

1. Install the CLI

   ```sh
   npm i -g ox-content
   ```

2. Run **build**

:::

````md
::: steps

1. Install the CLI

   ```sh
   npm i -g ox-content
   ```

2. Run **build**

:::
````

When enabled, the wrapper becomes `<div class="ox-steps">` with
`<ol class="ox-steps__list">` and `<li class="ox-steps__item">` for each item.

An unclosed `::: steps` stays literal and does not consume the rest of the
file. A plain `1. foo` list outside the wrapper is unchanged.

If custom containers are also enabled, `steps` is handled by this feature, not
as an unknown container type.

## Related

- [Custom Containers](./containers.md)
- [Built-in Features overview](../built-in-features.md)
