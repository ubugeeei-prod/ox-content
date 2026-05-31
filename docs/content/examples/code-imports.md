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

```md
<<< @/src/example.ts
<<< @/src/example.ts{1-12}
<<< @/src/example.ts{demo}
```

Line selectors use `1-12`. Named regions match comments containing `#region
name` and `#endregion name`.
