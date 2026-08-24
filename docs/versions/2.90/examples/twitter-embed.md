---
title: Twitter/X Embed
description: Render X posts as privacy-conscious static cards.
---

# Twitter/X Embed

Twitter/X embeds are opt-in and never load a third-party widget script.
`twitter: true` renders the privacy-conscious link card. Use the object form to
fetch the post body, author, avatar, and photos at build time:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        twitter: {
          fetch: true,
          lang: "en",
          mediaOutputDir: "public/ox-content/twitter",
          mediaPublicPath: "/ox-content/twitter",
        },
      },
    }),
  ],
};
```

```html
<XPost url="https://x.com/ox_content/status/1234567890" />
```

Fetched metadata is cached in memory and under `.cache/ox-content/twitter` by
default. Avatars and photos are copied into the configured output directory, so
the generated page can use a strict `img-src 'self'` policy. If the post is
deleted, private, or unavailable during a build, Ox Content falls back to the
link-only card instead of failing the build.

| Option            | Default                     | Purpose                                      |
| ----------------- | --------------------------- | -------------------------------------------- |
| `fetch`           | `false`                     | Fetch post content from X at build time.     |
| `lang`            | `"en"`                      | Syndication language and displayed date.     |
| `timeout`         | `10000`                     | Metadata request timeout in milliseconds.    |
| `cache`           | `true`                      | Enable in-memory and persistent JSON caches. |
| `cacheDir`        | `.cache/ox-content/twitter` | Persistent metadata cache directory.         |
| `mediaOutputDir`  | `public/ox-content/twitter` | Local directory for avatars and photos.      |
| `mediaPublicPath` | `/ox-content/twitter`       | URL prefix emitted for downloaded media.     |
