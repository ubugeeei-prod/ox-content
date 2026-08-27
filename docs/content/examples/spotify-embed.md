---
title: Spotify Embed
description: Render Spotify tracks, albums, playlists, and episodes.
---

# Spotify Embed

Spotify embeds are opt-in and render to the official iframe player.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        spotify: true,
      },
    }),
  ],
};
```

```html
<Spotify url="https://open.spotify.com/track/2VEQTuWiuEC7J8kkA7h7xq"></Spotify>
```

Supported paths include tracks, albums, playlists, episodes, shows, and artists.
