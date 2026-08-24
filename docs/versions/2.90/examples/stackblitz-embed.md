---
title: StackBlitz Embed
description: Render StackBlitz project links as iframes.
---

# StackBlitz Embed

StackBlitz embeds are opt-in through the built-in embed options.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        stackBlitz: true,
      },
    }),
  ],
};
```

```html
<StackBlitz url="https://stackblitz.com/edit/vitejs-vite-abc123"></StackBlitz>
```

The transform appends `embed=1` and renders a sandboxed iframe.
