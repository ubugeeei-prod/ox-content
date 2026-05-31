---
title: WebContainer Embed
description: Render lazy WebContainer placeholders.
---

# WebContainer Embed

WebContainer embeds are opt-in. The build transform emits a static placeholder
with the project source and marks the block as requiring cross-origin isolation.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        webContainer: true,
      },
    }),
  ],
};
```

```html
<WebContainer entry="index.html" title="Demo"> npm install npm run dev </WebContainer>
```
