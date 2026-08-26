---
title: Custom Containers
description: Opt-in ::: tip / ::: warning / ::: details blocks for documentation callouts.
---

# Custom Containers

`::: type` containers are opt-in. GitHub-style `> [!NOTE]` callouts stay on the
default renderer path and do not require this option.

| Option       | Type                           | Default |
| ------------ | ------------------------------ | ------- |
| `containers` | `boolean` / `ContainerOptions` | `false` |

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      containers: true,
    }),
  ],
};
```

## Built-in types

`tip`, `note`, `info`, `important`, `warning`, `danger`, `caution`, and
`details`.

```md
::: tip
Install the plugin first.
:::

::: warning Watch out
This changes rendered markup.
:::

::: details{open}
Optional extra context.
:::
```

Titles can be written as `::: tip Title` or `::: tip[Title]`. Attributes accept
`#id`, `.class`, and the boolean `open` flag on `details`.

## Custom types

```ts
oxContent({
  containers: {
    types: {
      cli: { title: "CLI" },
      spoiler: { title: "Spoiler", tag: "details" },
    },
  },
});
```

Type names must be ASCII identifiers. Hostile names and attributes are dropped.

## Related

- [Code Groups](./code-groups.md)
- [Syntax Extensions](./syntax-extensions.md)
- [Built-in Features overview](../built-in-features.md)
