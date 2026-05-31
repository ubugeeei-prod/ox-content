---
title: Edit This Page
description: Append source edit links to rendered pages.
---

# Edit This Page

Edit links are opt-in and use the Markdown file path passed into the transform.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      editThisPage: {
        repoUrl: "https://github.com/owner/repo",
        branch: "main",
        rootDir: process.cwd(),
        label: "Suggest an edit",
      },
    }),
  ],
};
```

The rendered link points to `repoUrl/edit/<branch>/<relative-source-path>`.
