---
title: Bluesky Embed
description: Render Bluesky posts as static cards.
---

# Bluesky Embed

Bluesky embeds are opt-in and render static links/cards.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        bluesky: true,
      },
    }),
  ],
};
```

```html
<Bluesky url="https://bsky.app/profile/example.com/post/abc123">
  Post text shown in the static card.
</Bluesky>
```
