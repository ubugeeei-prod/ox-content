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
  url="https://bsky.app/profile/danabra.mov/post/3mqzxmtfnxk2b"
  displayName="dan"
  handle="danabra.mov"
  avatar="https://cdn.bsky.app/img/avatar/plain/did:plc:fpruhuo22xkm5o7ttr2ktxdo/bafkreif43mhqajnbnl62u3ezf37g6x22nd762im54thxbil4ga46eugcga"
  dateTime="2026-07-19T23:46:21.231Z"
  dateLabel="Jul 19, 2026"
  replies="2"
  reposts="4"
  likes="72"
>
  the urge to fix everything incorrectly
</Bluesky>
```
