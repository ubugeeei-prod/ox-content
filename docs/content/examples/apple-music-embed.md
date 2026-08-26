---
title: Apple Music Embed
description: Render Apple Music albums, playlists, and songs.
---

# Apple Music Embed

Apple Music embeds are opt-in and render to Apple's official iframe player.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        appleMusic: true,
      },
    }),
  ],
};
```

```html
<AppleMusic url="https://music.apple.com/gb/album/1989-taylors-version/1708308989"></AppleMusic>
```

Supported share hosts are `music.apple.com` and already-embedded
`embed.music.apple.com` URLs. The transform rewrites the share origin to
`https://embed.music.apple.com` and keeps the storefront/path plus the `i=`
song-selection query.

The iframe is lazy-loaded, titled, and uses Apple's recommended `allow`,
`sandbox`, and `referrerpolicy`. It is a third-party player, so it stays
disabled until you opt in. Pages that set a Content-Security-Policy need
`frame-src https://embed.music.apple.com` (or the equivalent `child-src`).
