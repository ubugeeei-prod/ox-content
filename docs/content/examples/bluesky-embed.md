---
title: Bluesky Embed
description: Render Bluesky posts as static cards.
---

# Bluesky Embed

Bluesky embeds are opt-in and render static cards with author, time, and
engagement metadata when provided.

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

```mdx
<Bluesky
  url="https://bsky.app/profile/example.com/post/abc123"
  displayName="Example Author"
  handle="example.com"
  avatar="https://bsky.app/static/apple-touch-icon.png"
  dateTime="2024-02-06T12:34:56Z"
  dateLabel="Feb 6, 2024"
  replies="12"
  reposts="34"
  likes="256"
>
  Post text shown in the static card.
</Bluesky>
```
