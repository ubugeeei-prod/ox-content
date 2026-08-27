---
title: Twitter/X Embed
description: Render X posts as privacy-conscious static cards.
---

# Twitter/X Embed

Twitter/X embeds are opt-in and never load a third-party widget script.
`twitter: true` renders the privacy-conscious link card. Use the object form to
fetch the post body, author, avatar, photos, and video posters at build time.
Fetched cards also render timestamp, source link, available engagement metrics,
a nested quoted-post card, and a “Replying to @…” link when that metadata is
present. Set `appearance: "full"` for a sveltweet / react-tweet-shaped static
card, or override one embed with `appearance="full"`:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        twitter: {
          fetch: true,
          lang: "en",
          appearance: "compact",
          timeZone: "UTC",
          mediaOutputDir: "public/ox-content/twitter",
          mediaPublicPath: "/ox-content/twitter",
        },
      },
    }),
  ],
};
```

```html
<XPost url="https://x.com/evanyou/status/1688035849638977536" />
<XPost url="https://x.com/evanyou/status/1688035849638977536" appearance="full" />
```

Fetched metadata is cached in memory and under `.cache/ox-content/twitter` by
default. Avatars, photos, and video posters — including those on a quoted post —
are copied into the configured output directory, so the generated page can use a
strict `img-src 'self'` policy. Video and animated GIF files are downloaded only
when `downloadVideo` is true, stay within `maxVideoBytes`, and are never
hotlinked from `video.twimg.com`. If the post is deleted, private, or
unavailable during a build, Ox Content falls back to the link-only card instead
of failing the build. A missing quoted post is omitted without discarding the
root card.

| Option            | Default                     | Purpose                                          |
| ----------------- | --------------------------- | ------------------------------------------------ |
| `fetch`           | `false`                     | Fetch post content from X at build time.         |
| `lang`            | `"en"`                      | Syndication language and displayed date.         |
| `timeout`         | `10000`                     | Metadata request timeout in milliseconds.        |
| `cache`           | `true`                      | Enable in-memory and persistent JSON caches.     |
| `cacheDir`        | `.cache/ox-content/twitter` | Persistent metadata cache directory.             |
| `mediaOutputDir`  | `public/ox-content/twitter` | Local directory for avatars, photos, and videos. |
| `mediaPublicPath` | `/ox-content/twitter`       | URL prefix emitted for downloaded media.         |
| `downloadVideo`   | `false`                     | Download MP4 video and animated GIF assets.      |
| `maxVideoBytes`   | `8388608`                   | Skip videos larger than this (8 MiB).            |
| `appearance`      | `"compact"`                 | `"full"` for sveltweet-shaped static chrome.     |
| `timeZone`        | `"UTC"`                     | IANA zone for full-card timestamps.              |

Full appearance is static HTML/CSS: no hydration, widget iframe, or per-card
listeners. It reuses the same materialized avatar/photo/video assets as compact.
Pages that only render compact cards do not ship the full-card CSS.
Custom `ssg: false` hosts import `@ox-content/vite-plugin/styles/social.css`
and, for full cards, `styles/twitter-full.css`. See
[Component styles](../built-in/component-styles.md).

The full-card visual contract follows MIT-licensed
[react-tweet](https://github.com/vercel/react-tweet) (Copyright (c) 2023 Luis
Alvarez) and [sveltweet](https://github.com/ryoppippi/sveltweet) (Copyright (c)
2024 ryoppippi). Notices are in [Credits](../credits.md) and
`social-tweet-full.css`. X, Twitter, and related marks are trademarks of their
respective owners.

Intentional differences from `sveltweet@0.5.1` / `react-tweet`:

- Copy link is a static permalink with `data-ox-tweet-copy` hooks. The card stays
  useful without JavaScript; integrations that enhance those hooks get the same
  Copy link / Copied! swap as react-tweet.
- No fonts or assets from the X/Twitter CDN. The card uses the page font stack.
- Timestamps default to UTC so build output does not depend on the builder
  timezone. Set `timeZone: "Europe/London"` (or another IANA zone) to match a
  site-local clock. Invalid zones fall back to UTC.
- Downloaded video uses native `<video controls>` instead of a custom player.
- Government / legacy verified badges share the checkmark glyph and change color
  only. Side-by-side sveltweet VRT is not in CI; HTML fixtures cover the states.
