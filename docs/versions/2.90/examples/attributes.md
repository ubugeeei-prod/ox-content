---
title: Attribute Syntax
description: Add IDs, classes, and attributes with markdown-it-attrs syntax.
---

# Attribute Syntax

Attribute syntax is opt-in and runs as a post-render HTML transform.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      attrs: true,
    }),
  ],
};
```

```md
## Install {.anchor .highlight data-section=install}

[Docs](./index.md){.external}
```

Supported tokens include `#id`, `.class`, and `key=value`.
