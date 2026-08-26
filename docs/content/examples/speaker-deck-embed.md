---
title: Speaker Deck Embed
description: Render Speaker Deck talks with a lazy iframe or a fallback link card.
---

# Speaker Deck Embed

Speaker Deck embeds are opt-in. A player URL or oEmbed-resolved share URL
renders title, author, and a lazy iframe. Fetch or parse failures become a
safe link card.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        speakerDeck: true,
      },
    }),
  ],
};
```

Resolved player URL (no network request):

```html
<SpeakerDeck
  url="https://speakerdeck.com/player/abcdef1234567890"
  title="My Talk"
  author="Jane Doe"
></SpeakerDeck>
```

<SpeakerDeck url="https://speakerdeck.com/player/abcdef1234567890" title="My Talk" author="Jane Doe"></SpeakerDeck>

Share URLs (`https://speakerdeck.com/{user}/{slug}`) fetch oEmbed metadata at
build time and inject the player id, title, and author. `javascript:` and
`data:` URLs are rejected and stay as authored markup.

The iframe is lazy-loaded with `sandbox` and
`referrerpolicy="strict-origin-when-cross-origin"`. Pages that set a
Content-Security-Policy need `frame-src https://speakerdeck.com`.
