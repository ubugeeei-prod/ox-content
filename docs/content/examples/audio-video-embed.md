---
title: Audio and Video Embed
description: Render native audio and video players.
---

# Audio and Video Embed

Native audio and video embeds are opt-in and render to `<audio>` / `<video>`,
not third-party iframes.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        audio: true,
        video: true,
      },
    }),
  ],
};
```

```html
<audio src="https://cdn.example.com/intro.mp3" title="Intro" transcript="/intro.txt"></audio>
<video src="/talk.mp4" poster="/talk.jpg" captions="/talk.en.vtt" width="1280" height="720"></video>
```

`src` (or `url` / `href`) must be HTTPS or a same-origin relative path.
`javascript:`, `data:`, `http:`, and `//host/...` sources are rejected and left
as authored markup. Video accepts `poster`, `captions` / nested `<track>`,
`title`, `transcript`, `download`, and `width` / `height` for aspect-ratio
reservation. Native controls stay labeled.
