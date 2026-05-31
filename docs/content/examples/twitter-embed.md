---
title: Twitter/X Embed
description: Render X posts as privacy-conscious static cards.
---

# Twitter/X Embed

Twitter/X embeds are opt-in and render static cards by default. No third-party
widget script is loaded.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        twitter: true,
      },
    }),
  ],
};
```

```html
<Tweet id="1234567890">Post summary shown in the static card.</Tweet>
```
