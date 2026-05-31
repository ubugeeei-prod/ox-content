---
title: HTML Sanitizer
description: Sanitize rendered HTML with configurable allow lists.
---

# HTML Sanitizer

The sanitizer is opt-in. When enabled with `true`, it uses safe defaults for
common documentation HTML.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      sanitize: {
        allowedUrlSchemes: ["http", "https", "mailto"],
      },
    }),
  ],
};
```

Raw scripts, event-handler attributes, and unsafe URL schemes are removed during
the final HTML pass.
