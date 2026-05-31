---
title: CJK Emphasis
description: Recognize emphasis adjacent to CJK text.
---

# CJK Emphasis

The native parser recognizes emphasis next to CJK characters without requiring
ASCII spaces.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      cjkEmphasis: true,
    }),
  ],
};
```

```md
これは**重要**です。
これは*強調*です。
```

The option is explicit for compatibility, while the parser keeps the fast common
inline path.
