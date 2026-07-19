---
title: Embeds
description: GitHub cards, Open Graph cards, package-manager tabs, media embeds, and social cards authored as HTML-like tags in Markdown.
---

# Embeds

Embeds are HTML-like tags in Markdown that expand into static HTML at
transform time. Two are enabled by default because they produce plain static
markup; everything else is opt-in.

| Embed                | Option                | Default | Authoring form                     |
| -------------------- | --------------------- | ------- | ---------------------------------- |
| GitHub card          | `embeds.github`       | `true`  | `<GitHub repo="owner/name" />`     |
| Open Graph link card | `embeds.openGraph`    | `true`  | `<OgCard url="https://..." />`     |
| Package manager tabs | `embeds.pm`           | `false` | `<pm>npm install pkg</pm>`         |
| Twitter/X            | `embeds.twitter`      | `false` | `<Tweet />` or `<XPost />`         |
| Bluesky              | `embeds.bluesky`      | `false` | `<Bluesky />`                      |
| Spotify              | `embeds.spotify`      | `false` | `<Spotify url="https://..." />`    |
| StackBlitz           | `embeds.stackBlitz`   | `false` | `<StackBlitz url="https://..." />` |
| WebContainer         | `embeds.webContainer` | `false` | `<WebContainer />`                 |

Tabs and YouTube embeds are not part of the `embeds` option: they are always
processed in SSG builds and dev preview, with no configuration needed. They are
covered [below](#tabs) because they share the same authoring model.

Disable every built-in embed with `embeds: false`, or configure embeds
individually:

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      embeds: {
        github: { maxSourceLines: 120 },
        openGraph: { timeout: 5000 },
        pm: { sync: true },
        twitter: true,
        bluesky: true,
      },
    }),
  ],
};
```

## GitHub Cards

`embeds.github` renders repository cards and source snippets from the GitHub
API at build time. The output is static HTML — no client-side JavaScript, no
third-party widget script.

A repository card:

```md
<GitHub repo="ubugeeei-prod/ox-content"></GitHub>
```

<GitHub repo="ubugeeei-prod/ox-content"></GitHub>

A source snippet pinned to a ref and line range:

```md
<GitHub repo="ubugeeei-prod/ox-content" path="README.md" ref="main" loc="1-10"></GitHub>
```

<GitHub repo="ubugeeei-prod/ox-content" path="README.md" ref="main" loc="1-10"></GitHub>

A permalink form is also supported — paste a GitHub blob URL with `#L2-L8`
line anchors:

```md
<GitHub permalink="https://github.com/owner/repo/blob/abc123/src/index.ts#L2-L8"></GitHub>
```

| Option           | Default   | Purpose                                             |
| ---------------- | --------- | --------------------------------------------------- |
| `token`          | `""`      | GitHub API token for rate limits and private repos. |
| `cache`          | `true`    | Cache API responses in memory.                      |
| `cacheTTL`       | `3600000` | Cache lifetime in milliseconds.                     |
| `maxSourceBytes` | `200000`  | Skip files larger than this.                        |
| `maxSourceLines` | `120`     | Max inline lines when no range is given.            |

`process.env.GITHUB_TOKEN` is picked up automatically when no explicit `token`
is configured. If a repository or file cannot be fetched during the build —
offline CI, rate limits, an invalid path — the embed renders a fallback link
card instead of failing the build.

## Open Graph Cards

`embeds.openGraph` fetches a page's Open Graph metadata at build time and
renders a static link card:

```md
<OgCard url="https://vite.dev"></OgCard>
```

<OgCard url="https://vite.dev"></OgCard>

| Option      | Default                      | Purpose                         |
| ----------- | ---------------------------- | ------------------------------- |
| `timeout`   | `10000`                      | Fetch timeout in milliseconds.  |
| `cache`     | `true`                       | Cache fetched metadata.         |
| `cacheTTL`  | `3600000`                    | Cache lifetime in milliseconds. |
| `userAgent` | `ox-content-ogp-bot/1.0 ...` | User agent sent to the target.  |

Unreachable pages fall back to a plain link card. Requests to localhost,
private IP ranges, and non-HTTP(S) schemes are rejected, so Markdown content
cannot probe the network the build runs in.

## Package Manager Tabs

`embeds.pm` expands one npm-style command into an accessible tab group for
npm, pnpm, yarn, and bun:

```ts
oxContent({
  embeds: {
    pm: true,
  },
});
```

```md
<pm>npm install -D @ox-content/vite-plugin</pm>
```

<pm>npm install -D @ox-content/vite-plugin</pm>

The command is converted natively in Rust — `npm install -D` becomes
`pnpm add -D`, `yarn add -D`, and `bun add -D`. The tabs work without
client-side JavaScript; selection uses CSS `:has()`. Opt in to
`pm: { sync: true }` to synchronize the selected package manager across every
block on the page via `localStorage`. See
[Package Manager Tabs](../examples/package-manager-tabs.md) for the full
conversion table.

## Tabs

Generic tab groups use the same widget as package-manager tabs and are always
available in SSG builds and dev preview:

```md
<tabs>
  <tab label="macOS">brew install oxc</tab>
  <tab label="Linux">apt install oxc</tab>
  <tab label="Windows">winget install oxc</tab>
</tabs>
```

<tabs>
  <tab label="macOS">brew install oxc</tab>
  <tab label="Linux">apt install oxc</tab>
  <tab label="Windows">winget install oxc</tab>
</tabs>

A `<tab>` without a `label` attribute falls back to `Tab 1`, `Tab 2`, and so
on.

## YouTube

YouTube embeds are always processed in SSG builds and dev preview. The iframe
uses privacy-enhanced mode (`youtube-nocookie.com`) and lazy loading by
default:

```md
<youtube id="aqz-KE-bpKQ" title="Big Buck Bunny"></youtube>
```

<youtube id="aqz-KE-bpKQ" title="Big Buck Bunny"></youtube>

`id`, `url`, and `href` attributes are accepted; `youtu.be`, `watch?v=`,
`shorts`, and `embed` URL shapes are all recognized.

## Twitter/X

`embeds.twitter` renders posts as static cards and never loads the third-party
widget script. With `twitter: true`, the embed is a privacy-conscious link
card:

```md
<XPost url="https://x.com/jack/status/20"></XPost>
```

<XPost url="https://x.com/jack/status/20"></XPost>

Use the object form to fetch the post body, author, avatar, and photos at
build time and serve them from your own origin:

```ts
oxContent({
  embeds: {
    twitter: {
      fetch: true,
      lang: "en",
      mediaOutputDir: "public/ox-content/twitter",
      mediaPublicPath: "/ox-content/twitter",
    },
  },
});
```

| Option            | Default                     | Purpose                                   |
| ----------------- | --------------------------- | ----------------------------------------- |
| `fetch`           | `false`                     | Fetch post content at build time.         |
| `lang`            | `"en"`                      | Syndication language and displayed date.  |
| `timeout`         | `10000`                     | Metadata request timeout in milliseconds. |
| `cache`           | `true`                      | In-memory and persistent JSON caches.     |
| `cacheDir`        | `.cache/ox-content/twitter` | Persistent metadata cache directory.      |
| `mediaOutputDir`  | `public/ox-content/twitter` | Local directory for avatars and photos.   |
| `mediaPublicPath` | `/ox-content/twitter`       | URL prefix emitted for downloaded media.  |

Downloaded media is served from your site, so a strict `img-src 'self'` CSP
keeps working. Deleted or private posts fall back to the link-only card
instead of failing the build. See
[Twitter/X Embed](../examples/twitter-embed.md) for details.

## Bluesky

`embeds.bluesky` renders a static card. The element body provides the text
shown in the card, so no network request is needed at all:

```md
<Bluesky url="https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l">
  👋 Bluesky is an open social network
</Bluesky>
```

<Bluesky url="https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l">
  👋 Bluesky is an open social network
</Bluesky>

## Spotify

`embeds.spotify` renders the official iframe player for tracks, albums,
playlists, episodes, shows, and artists:

```md
<Spotify url="https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC"></Spotify>
```

The output is an `<iframe>` pointing at `open.spotify.com/embed/...`. It is
opt-in — and not enabled on this site — because the player loads third-party
resources in the reader's browser.

## StackBlitz

`embeds.stackBlitz` turns a StackBlitz project URL into a sandboxed iframe
with `embed=1` appended:

```md
<StackBlitz url="https://stackblitz.com/edit/vitejs-vite-abc123"></StackBlitz>
```

## WebContainer

`embeds.webContainer` emits a lazy placeholder carrying the project source and
cross-origin isolation metadata, for sites that boot
[WebContainers](https://webcontainers.io/) on interaction:

```md
<WebContainer entry="index.html" title="Demo">
  npm install
  npm run dev
</WebContainer>
```

See [WebContainer Embed](../examples/webcontainer-embed.md) for the isolation
requirements.

## Related

- [Mermaid Diagrams](./mermaid.md) — diagram fences rendered to static SVG.
- [Built-in Features overview](../built-in-features.md)
