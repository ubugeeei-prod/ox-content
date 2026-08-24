---
title: Code Block Linting
description: Lint fenced code blocks during Markdown transforms.
---

# Code Block Linting

Code block linting is opt-in and runs only when Markdown contains fenced code
blocks.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      codeBlockLint: {
        languages: ["ts", "tsx"],
        requireLanguage: true,
        trailingSpaces: true,
        mode: "error",
      },
    }),
  ],
};
```

The native scanner reports diagnostics with source line numbers. Use
`mode: "warn"` to annotate builds without failing them.
