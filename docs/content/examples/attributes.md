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

[Docs](./index.md){.external data-kind=guide}

![Package graph](./package-graph.png){.w-1/2 .mx-auto width=480}
```

Supported tokens include `#id`, `.class`, and `key=value`. Link attributes
attach to the generated `<a>`, and image classes, dimensions, and safe data
attributes stay on the generated `<img>`.
