---
title: Code Imports
description: Import source snippets into Markdown fences.
---

# Code Imports

Code imports are opt-in and resolve files inside the configured root directory.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      codeImports: {
        rootDir: process.cwd(),
      },
    }),
  ],
};
```

Each import is a `<<<` reference on its own line: `<<< @/src/example.ts`
imports the whole file, `<<< @/src/example.ts{1-12}` imports a line range, and
`<<< @/src/example.ts{demo}` imports a named region. Named regions match
comments containing `#region name` and `#endregion name`.

See [Code Blocks](../built-in/code-blocks.md#code-imports) for live rendered
examples.
