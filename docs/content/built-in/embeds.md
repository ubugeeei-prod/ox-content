---
title: Embeds
description: GitHub cards, Open Graph cards, package-manager tabs, media embeds, and social cards authored as HTML-like tags in Markdown.
---

# Embeds

Embeds are HTML-like tags in Markdown that expand into static HTML at
transform time. Two are enabled by default because they produce plain static
markup; everything else is opt-in.

| Embed                | Option                   | Default | Authoring form                      |
| -------------------- | ------------------------ | ------- | ----------------------------------- |
| GitHub card          | `embeds.github`          | `true`  | `<GitHub repo="owner/name" />`      |
| Open Graph link card | `embeds.openGraph`       | `true`  | `<OgCard url="https://..." />`      |
| Package manager tabs | `embeds.pm`              | `false` | `<pm>npm install pkg</pm>`          |
| Twitter/X            | `embeds.twitter`         | `false` | `<Tweet />` or `<XPost />`          |
| Reddit               | `embeds.reddit`          | `false` | `<Reddit url="https://..." />`      |
| Bluesky              | `embeds.bluesky`         | `false` | `<Bluesky />`                       |
| Google Maps          | `embeds.googleMaps`      | `false` | `<GoogleMaps url="https://..." />`  |
| Qiita                | `embeds.qiita`           | `false` | `<Qiita url="https://..." />`       |
| Zenn                 | `embeds.zenn`            | `false` | `<Zenn url="https://..." />`        |
| Package registries   | `embeds.packageRegistry` | `false` | `<NpmPackage url="https://..." />`  |
| Playgrounds          | `embeds.playgrounds`     | `false` | `<CodePen url="https://..." />`     |
| Vimeo                | `embeds.vimeo`           | `false` | `<Vimeo url="https://..." />`       |
| Twitch               | `embeds.twitch`          | `false` | `<Twitch url="https://..." />`      |
| Discord              | `embeds.discord`         | `false` | `<Discord url="https://..." />`     |
| Fediverse            | `embeds.fediverse`       | `false` | `<Mastodon url="https://..." />`    |
| Facebook             | `embeds.facebook`        | `false` | `<Facebook url="https://..." />`    |
| Threads              | `embeds.threads`         | `false` | `<Threads url="https://..." />`     |
| Instagram            | `embeds.instagram`       | `false` | `<Instagram url="https://..." />`   |
| Spotify              | `embeds.spotify`         | `false` | `<Spotify url="https://..." />`     |
| Apple Music          | `embeds.appleMusic`      | `false` | `<AppleMusic url="https://..." />`  |
| Speaker Deck         | `embeds.speakerDeck`     | `false` | `<SpeakerDeck url="https://..." />` |
| Audio                | `embeds.audio`           | `false` | `<Audio src="https://..." />`       |
| Video                | `embeds.video`           | `false` | `<Video src="https://..." />`       |
| StackBlitz           | `embeds.stackBlitz`      | `false` | `<StackBlitz url="https://..." />`  |
| WebContainer         | `embeds.webContainer`    | `false` | `<WebContainer />`                  |

Tabs and YouTube embeds are not part of the `embeds` option: they are always
processed in SSG builds and dev preview, with no configuration needed. They are
covered [below](#tabs) because they share the same authoring model.

Documented PascalCase tags such as `<Tweet>` and `<OgCard>` work in both `.md`
and `.mdx`. A document-local import of the same name (`import Tweet from
"./Tweet"`) overrides the built-in and stays an MDX island.

### One tag, one line, in `.md`

The examples below spread attributes over several lines for readability. That
form needs MDX. In a plain `.md` file a tag has to open and close its `>` on the
same line:

```md
<Bluesky url="https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l" handle="bsky.app">…</Bluesky>
```

CommonMark only starts a raw HTML block when the opening tag finishes on the
line it began on. A tag left open at the end of a line is prose, so its
attributes render as text, its URLs turn into links, and the lone `>` line
becomes a blockquote. Enable `mdx` to write the multi-line form.

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
        reddit: true,
        bluesky: true,
        qiita: true,
        zenn: true,
        packageRegistry: true,
        playgrounds: true,
        vimeo: true,
        twitch: { iframe: true, parent: "docs.example.com" },
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

```mdx
<GitHub repo="ubugeeei-prod/ox-content" />
```

<GitHub repo="ubugeeei-prod/ox-content" />

A source snippet pinned to a ref and line range:

```mdx
<GitHub repo="ubugeeei-prod/ox-content" path="README.md" ref="main" loc="1-10" />
```

<GitHub repo="ubugeeei-prod/ox-content" path="README.md" ref="main" loc="1-10" />

A permalink form is also supported — paste a GitHub blob URL with `#L10-L18`
line anchors:

```mdx
<GitHub permalink="https://github.com/ubugeeei-prod/ox-content/blob/278098b/npm/vite-plugin-ox-content/src/plugins/github.ts#L10-L18" />
```

<GitHub permalink="https://github.com/ubugeeei-prod/ox-content/blob/278098b/npm/vite-plugin-ox-content/src/plugins/github.ts#L10-L18" />

The source card header links to the blob and, when the GitHub API returns it,
shows the latest commit that touched that path at the pinned ref.

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

GitHub issue, pull request, commit, discussion, and gist URLs also render as
static cards through the same `embeds.github` option:

```mdx
<GitHub url="https://github.com/ubugeeei-prod/ox-content/issues/699" />
<GitHub url="https://github.com/ubugeeei-prod/ox-content/pull/1025" />
<GitHub url="https://github.com/ubugeeei-prod/ox-content/commit/5399e080b5320d730e410a49a5aab42ba670a1f1" />
<GitHub url="https://github.com/ubugeeei-prod/ox-content/discussions/1" />
<GitHub url="https://gist.github.com/ubugeeei/0123456789abcdef0123456789abcdef" />
```

These resource cards use unauthenticated public metadata fetches only. Deleted,
private, rate-limited, or unsupported resources fall back to deterministic
link-only cards.

## Open Graph Cards

`embeds.openGraph` fetches a page's Open Graph metadata at build time and
renders a static link card:

```mdx
<OgCard url="https://vite.dev" />
```

<OgCard url="https://vite.dev" />

| Option         | Default                      | Purpose                                                |
| -------------- | ---------------------------- | ------------------------------------------------------ |
| `timeout`      | `10000`                      | Fetch timeout in milliseconds.                         |
| `cache`        | `true`                       | Cache fetched metadata in memory for this process.     |
| `cacheTTL`     | `3600000`                    | Freshness window in milliseconds.                      |
| `persistCache` | `false`                      | Persist successful and negative entries across builds. |
| `cacheDir`     | `.cache/ox-content/ogp`      | Persistent metadata cache directory.                   |
| `refresh`      | `false`                      | Re-fetch even when a fresh cache entry exists.         |
| `userAgent`    | `ox-content-ogp-bot/1.0 ...` | User agent sent to the target.                         |

Set `persistCache: true` to reuse metadata across clean builds and CI workers.
Successful lookups and unavailable URLs are stored as one JSON file per
normalized URL under `cacheDir`. Fresh entries skip the network; expired
entries and `refresh: true` fetch again and replace the file atomically.
Corrupt files are ignored so they cannot poison later builds. Metadata you
already supply to the transform still wins over cache and fetch.

Unreachable pages fall back to a plain link card. Requests to localhost,
private IP ranges, and non-HTTP(S) schemes are rejected, so Markdown content
cannot probe the network the build runs in.

Card text is decoded from the page's own markup, so an `og:title` written as
`Tips &amp; Tricks` renders as `Tips & Tricks`. An `og:image` is resolved
against the page it was declared on — absolute, protocol-relative, and
document-relative forms all work — and is dropped when it resolves somewhere
the fetcher would refuse to go.

The favicon comes from the target page's own `<link rel="icon">`, falling back
to `/favicon.ico` on that origin. No third-party favicon service is contacted,
so rendering a card never tells an outside host which links a documentation
page carries.

## Package Manager Tabs

`embeds.pm` expands one npm-style command into an accessible tab group for
npm, pnpm, yarn, bun, and vp (Vite+):

```ts
oxContent({
  embeds: {
    pm: true,
  },
});
```

```mdx
<pm>npm install -D @ox-content/vite-plugin @ox-content/theme-swiss</pm>
```

<pm>npm install -D @ox-content/vite-plugin @ox-content/theme-swiss</pm>

The command is converted natively in Rust — `npm install -D` becomes
`pnpm add -D`, `yarn add -D`, `bun add -D`, and `vp install -D`, while
`npx <bin>` becomes `vp exec -- <bin>`. The tabs work without client-side
JavaScript; selection uses CSS `:has()`. Opt in to `pm: { sync: true }` to
synchronize the selected package manager across every block on the page via
`localStorage`. See
[Package Manager Tabs](../examples/package-manager-tabs.md) for the full
conversion table.

## Tabs

Generic tab groups use the same widget as package-manager tabs and are always
available in SSG builds and dev preview:

```html
<tabs>
  <tab label="Install">
    <pre><code>pnpm add -D @ox-content/vite-plugin
pnpm add -D @ox-content/theme-swiss</code></pre>
  </tab>
  <tab label="Config">
    <pre><code>oxContent({ srcDir: "content", embeds: { pm: true } })</code></pre>
  </tab>
  <tab label="Markdown">
    <pre><code>---
title: Install
---

Install Ox Content with the package manager you prefer.

&lt;pm&gt;npm install -D @ox-content/vite-plugin&lt;/pm&gt;</code></pre>
  </tab>
  <tab label="Build">
    <pre><code>pnpm vite build
pnpm vite preview</code></pre>
  </tab>
</tabs>
```

<tabs>
<tab label="Install">
<pre><code>pnpm add -D @ox-content/vite-plugin
pnpm add -D @ox-content/theme-swiss</code></pre>
</tab>
<tab label="Config">
<pre><code>oxContent({ srcDir: "content", embeds: { pm: true } })</code></pre>
</tab>
<tab label="Markdown">
<pre><code>---
title: Install
---

Install Ox Content with the package manager you prefer.

&lt;pm&gt;npm install -D @ox-content/vite-plugin&lt;/pm&gt;</code></pre>
</tab>
<tab label="Build">
<pre><code>pnpm vite build
pnpm vite preview</code></pre>
</tab>
</tabs>

A `<tab>` without a `label` attribute falls back to `Tab 1`, `Tab 2`, and so
on.

For adjacent code samples, prefer the opt-in `::: code-group` form instead of
hand-written `<tabs>`. See [Code Groups](./code-groups.md).

## YouTube

YouTube embeds are always processed in SSG builds and dev preview. The iframe
uses privacy-enhanced mode (`youtube-nocookie.com`) and lazy loading by
default:

```mdx
<youtube id="aqz-KE-bpKQ" title="Big Buck Bunny" />
```

<youtube id="aqz-KE-bpKQ" title="Big Buck Bunny" />

`id`, `url`, and `href` attributes are accepted; `youtu.be`, `watch?v=`,
`shorts`, and `embed` URL shapes are all recognized. `start` accepts a
non-negative integer number of seconds and becomes `?start=` on the iframe
URL. Invalid, negative, fractional, overflowing, or duplicated values are
ignored. Omitting `start` leaves the previous URL unchanged.

```mdx
<youtube id="aqz-KE-bpKQ" title="Big Buck Bunny" start="4190" />
```

## Twitter/X

`embeds.twitter` renders posts as static cards and never loads the third-party
widget script. With `twitter: true`, the embed is a privacy-conscious card. The
element body provides the post text, and optional attributes can add author,
avatar, timestamp, engagement metrics, and a clear original-post link without a
network request:

```mdx
<XPost
  url="https://x.com/jack/status/20"
  displayName="jack"
  handle="jack"
  dateLabel="Mar 21, 2006"
  likes="2.4M"
  views="10M"
>
  just setting up my twttr
</XPost>
```

<XPost url="https://x.com/jack/status/20" displayName="jack" handle="jack" dateLabel="Mar 21, 2006" likes="2.4M" views="10M">just setting up my twttr</XPost>

Use the object form to fetch the post body, author, avatar, photos, and video
posters at build time and serve them from your own origin. Fetched cards include
timestamp, source link, available reply/repost/quote/like/view metrics, a nested
quoted-post card, and a “Replying to @…” link when the syndication response has
that metadata. `appearance: "full"` opts into a sveltweet / react-tweet-shaped
static card:

```ts
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
});
```

| Option            | Default                     | Purpose                                          |
| ----------------- | --------------------------- | ------------------------------------------------ |
| `fetch`           | `false`                     | Fetch post content at build time.                |
| `lang`            | `"en"`                      | Syndication language and displayed date.         |
| `timeout`         | `10000`                     | Metadata request timeout in milliseconds.        |
| `cache`           | `true`                      | In-memory and persistent JSON caches.            |
| `cacheDir`        | `.cache/ox-content/twitter` | Persistent metadata cache directory.             |
| `mediaOutputDir`  | `public/ox-content/twitter` | Local directory for avatars, photos, and videos. |
| `mediaPublicPath` | `/ox-content/twitter`       | URL prefix emitted for downloaded media.         |
| `downloadVideo`   | `false`                     | Download MP4 video and animated GIF assets.      |
| `maxVideoBytes`   | `8388608`                   | Skip videos larger than this (8 MiB).            |
| `appearance`      | `"compact"`                 | `"full"` for sveltweet-shaped static chrome.     |
| `timeZone`        | `"UTC"`                     | IANA zone for full-card timestamps.              |

Downloaded media is served from your site, so a strict `img-src 'self'` CSP
keeps working. Video and animated GIF posts use a self-hosted poster and a
Watch on X permalink unless `downloadVideo` is enabled, and the generated HTML
never includes `video.twimg.com`. Deleted or private posts fall back to the
link-only card instead of failing the build. A missing quoted post is omitted
without discarding the root card. Full-card CSS ships only on pages that
render `.ox-tweet--full`. The full-card chrome follows the MIT-licensed
[react-tweet](https://github.com/vercel/react-tweet) and
[sveltweet](https://github.com/ryoppippi/sveltweet) visual contract; notices
are in [Credits](../credits.md). See
[Twitter/X Embed](../examples/twitter-embed.md) for details.
Custom hosts import `@ox-content/vite-plugin/styles/social.css` and, for
`appearance: "full"`, `styles/twitter-full.css`. See
[Component styles](./component-styles.md).

## Reddit

`embeds.reddit` renders Reddit posts as static cards. The provider is opt-in
and never loads Reddit's widget script. With `reddit: true`, ox-content fetches
the post JSON at build time and includes the subreddit, author, title, body
excerpt, score, comment count, timestamp, image preview, and original link when
Reddit returns them:

```ts
oxContent({
  embeds: {
    reddit: true,
  },
});
```

```mdx
<Reddit url="https://www.reddit.com/r/webdev/comments/abc123/release_notes/" />
```

`reddit.com/r/{subreddit}/comments/{id}/{slug}` URLs and `redd.it/{id}` share
links normalize to canonical `https://www.reddit.com/...` URLs before output.
New Reddit `/r/{subreddit}/s/{share}` links are accepted as link-only cards
because the post id is not present in the URL without following a remote
redirect.

| Option      | Default                         | Purpose                                          |
| ----------- | ------------------------------- | ------------------------------------------------ |
| `fetch`     | `true`                          | Fetch post metadata at build time.               |
| `timeout`   | `10000`                         | Metadata request timeout in milliseconds.        |
| `cache`     | `true`                          | Cache fetched metadata in memory for this build. |
| `cacheTTL`  | `3600000`                       | Freshness window in milliseconds.                |
| `userAgent` | `ox-content-reddit-bot/1.0 ...` | User agent sent to Reddit's JSON endpoint.       |

Set `reddit: { fetch: false }` to render a privacy-conscious link card without
network access. Deleted, private, rate-limited, or otherwise unavailable posts
also fall back to the link card instead of failing the build. Unsupported
schemes, credentials, non-Reddit hosts, and non-post paths render an error card
with `href="#"`.

## Bluesky

`embeds.bluesky` renders a static card with optional author, avatar, timestamp,
and engagement metadata. The element body provides the post text, so no network
request is needed at all:

```mdx
<Bluesky
  url="https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l"
  displayName="Bluesky"
  handle="bsky.app"
  avatar="https://bsky.app/static/apple-touch-icon.png"
  dateTime="2024-02-06T12:34:56Z"
  dateLabel="Feb 6, 2024"
  replies="1.2k"
  reposts="8.4k"
  likes="21k"
>
  👋 Bluesky is an open social network
</Bluesky>
```

<Bluesky url="https://bsky.app/profile/bsky.app/post/3l6oveex3ii2l" displayName="Bluesky" handle="bsky.app" avatar="https://bsky.app/static/apple-touch-icon.png" dateTime="2024-02-06T12:34:56Z" dateLabel="Feb 6, 2024" replies="1.2k" reposts="8.4k" likes="21k">👋 Bluesky is an open social network</Bluesky>

## Provider Cards

`embeds.googleMaps`, `embeds.qiita`, `embeds.zenn`,
`embeds.packageRegistry`, `embeds.playgrounds`, `embeds.vimeo`,
`embeds.twitch`, `embeds.discord`, `embeds.fediverse`, `embeds.facebook`,
`embeds.threads`, and `embeds.instagram` render static provider cards.
`qiita: true`, `zenn: true`, `packageRegistry: true`, Vimeo cards, and CodePen
playground cards fetch public metadata at build time; set `{ fetch: false }` to
render link cards without network access. Playground and video iframe URLs are
added only with `{ iframe: true }`. Twitch iframes also require a `parent`
domain, for example `twitch: { iframe: true, parent: "docs.example.com" }`.
Other provider cards do not fetch metadata or load third-party scripts by
default. Authors can pass stable metadata with attributes such as `title`,
`author`, `avatar`, `date`, `dateLabel`, `tags`, `likes`, `reposts`,
`replies`, `server`, `channel`, `address`, `image`, `version`, `license`,
`repository`, `downloads`, `stars`, `language`, `runtime`, `duration`,
`status`, and `views`.

```mdx
<GoogleMaps
  url="https://www.google.com/maps/place/Tokyo+Station/"
  place="Tokyo Station"
  address="1 Chome Marunouchi, Chiyoda City"
/>

<Qiita
  url="https://qiita.com/ubugeeei/items/abcdef123456"
  title="Rust docs pipeline"
  author="ubugeeei"
  tags="Rust, Markdown"
  likes="42"
>
  Static cards keep builds predictable.
</Qiita>

<NpmPackage url="https://www.npmjs.com/package/vite" />
<CratesIo url="https://crates.io/crates/serde" />
<PyPI url="https://pypi.org/project/requests" />
<DockerHub url="https://hub.docker.com/_/nginx" />

<CodePen url="https://codepen.io/ubugeeei/pen/abc123" />

<JSFiddle url="https://jsfiddle.net/ubugeeei/abc123/2/" />

<Observable url="https://observablehq.com/@d3/bar-chart" />

<Vimeo url="https://vimeo.com/123456789" />

<Twitch url="https://www.twitch.tv/videos/40464143" />

<Twitch url="https://clips.twitch.tv/FriendlySlug" />

<Twitch url="https://www.twitch.tv/twitchdev" />

<Mastodon
  url="https://mastodon.social/@docs/111"
  author="@docs@mastodon.social"
  replies="3"
  reposts="5"
  likes="8"
>
  Fediverse release note.
</Mastodon>
```

| Option     | Default   | Purpose                                          |
| ---------- | --------- | ------------------------------------------------ |
| `fetch`    | `true`    | Fetch article/package/playground/video metadata. |
| `timeout`  | `10000`   | Metadata request timeout in milliseconds.        |
| `cache`    | `true`    | Cache fetched metadata in memory for this build. |
| `cacheTTL` | `3600000` | Freshness window in milliseconds.                |
| `iframe`   | `false`   | Add lazy playground/video iframe URLs.           |
| `parent`   | `[]`      | Twitch iframe parent domain or domains.          |

`<Fediverse>`, `<Mastodon>`, `<Misskey>`, and `<Mixi2>` share the
`embeds.fediverse` option. Google Maps accepts an optional safe Google Maps
`embed` URL for a lazy iframe; otherwise it renders a link-only place card.
Package registry cards support npm, crates.io, PyPI, and Docker Hub package
URLs, including version or tag URLs where the provider exposes one. Unsupported
schemes, credentials, non-provider hosts, and unavailable metadata fall back to
the authored tag or a link-only card instead of failing the build; failed
package metadata fetches also emit an `[ox-content]` warning with the status or
error reason.
Vimeo cards use Vimeo's public oEmbed endpoint for metadata. Twitch cards avoid
authenticated API calls by default; pass `title`, `channel`, `duration`,
`status`, `views`, or `image` when you want richer static metadata. Twitch
player URLs are only generated when a safe `parent` domain is configured,
matching Twitch's embed requirements.

## Spotify

`embeds.spotify` renders the official iframe player for tracks, albums,
playlists, episodes, shows, and artists:

```mdx
<Spotify url="https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC" />
```

<Spotify url="https://open.spotify.com/track/4uLU6hMCjMI75M1A2tKUQC" />

The output is an `<iframe>` pointing at `open.spotify.com/embed/...` with lazy
loading. Unlike the static cards above it is a real third-party player, which
is why it stays opt-in.

The frame is named after what it plays — `Spotify track`, `Spotify playlist`,
and so on — so a screen reader announces something more useful than "frame".
Pass `title` to name it yourself:

```mdx
<Spotify url="https://open.spotify.com/album/..." title="The album we discuss below" />
```

## Apple Music

`embeds.appleMusic` renders Apple's official iframe player for albums,
playlists, songs, artists, and music videos:

```mdx
<AppleMusic url="https://music.apple.com/gb/album/1989-taylors-version/1708308989" />
```

<AppleMusic url="https://music.apple.com/gb/album/1989-taylors-version/1708308989" />

Share URLs on `music.apple.com` are rewritten to
`embed.music.apple.com`, keeping the storefront/path and the `i=` song
selection query. Already-embedded `embed.music.apple.com` URLs are accepted
after the same host and path checks. Non-HTTPS URLs, lookalike hosts,
credentials, fragments, and malformed paths stay as authored markup instead
of becoming an iframe.

The player is a third-party iframe, so the option stays off by default.
Sites that set a Content-Security-Policy need
`frame-src https://embed.music.apple.com` (or the equivalent `child-src`)
before the player can load. See [Apple Music Embed](../examples/apple-music-embed.md)
for authoring details.

## Speaker Deck

`embeds.speakerDeck` renders a lazy Speaker Deck player when a player URL or
oEmbed metadata can be resolved, and a safe link card when fetch or parse
fails:

```mdx
<SpeakerDeck url="https://speakerdeck.com/jane/my-talk" title="My Talk" author="Jane Doe" />
```

<SpeakerDeck url="https://speakerdeck.com/jane/my-talk" title="My Talk" author="Jane Doe" />

The deck above does not exist, so the example shows the link-card fallback
rather than a player.

Share URLs on `speakerdeck.com/{user}/{slug}` fetch [oEmbed](https://oembed.com/)
metadata at build time (`title`, `author_name`, player id, and thumbnail when
present). Already-embedded `speakerdeck.com/player/{id}` URLs render without a
network request — including ids that do not exist, which embed the provider's
own error page, so prefer a share URL in examples. `javascript:` and `data:`
URLs stay as authored markup.

When oEmbed fetch or player-id parse fails, the output is a fallback link card
that still points at the original HTTPS Speaker Deck URL. The iframe is
lazy-loaded, sandboxed, and uses `referrerpolicy="strict-origin-when-cross-origin"`.
Sites that set a Content-Security-Policy need
`frame-src https://speakerdeck.com`. See
[Speaker Deck Embed](../examples/speaker-deck-embed.md).

## Audio and Video

`embeds.audio` and `embeds.video` render native `<audio>` / `<video>` players.
They stay off by default and never load a third-party iframe.

```ts
oxContent({ embeds: { audio: true, video: true } });
```

```mdx
<Audio
  src="https://cdn.example.com/intro.mp3"
  title="Episode intro"
  transcript="/intro.txt"
  download="/intro.mp3"
/>

<Video
  src="/talk.mp4"
  poster="/talk.jpg"
  captions="/talk.en.vtt"
  srclang="en"
  label="English"
  width="1280"
  height="720"
  title="Release talk"
/>
```

Sources must be HTTPS or same-origin relative paths. `javascript:`, `data:`,
`http:`, and protocol-relative URLs stay as authored markup. Nested `<track>`
elements supply extra captions or subtitles. Native controls are labeled with
`title` (or `Audio` / `Video`). Width and height reserve the video aspect ratio
so the layout does not shift. See
[Audio and Video Embed](../examples/audio-video-embed.md).

## StackBlitz

`embeds.stackBlitz` turns a StackBlitz project URL into a sandboxed iframe
with `embed=1` appended:

```mdx
<StackBlitz url="https://stackblitz.com/edit/vitejs-vite" />
```

<StackBlitz url="https://stackblitz.com/edit/vitejs-vite" />

## WebContainer

`embeds.webContainer` emits a lazy placeholder carrying the project source and
cross-origin isolation metadata, for sites that boot
[WebContainers](https://webcontainers.io/) on interaction. The placeholder
itself is fully static:

<!-- prettier-ignore -->
```mdx
<WebContainer entry="index.html" title="Demo">
  npm install
  npm run dev
</WebContainer>
```

<WebContainer entry="index.html" title="Demo">
  npm install
  npm run dev
</WebContainer>

See [WebContainer Embed](../examples/webcontainer-embed.md) for the isolation
requirements.

## Related

- [Mermaid Diagrams](./mermaid.md) — diagram fences rendered to static SVG.
- [Component styles](./component-styles.md) — official CSS for custom hosts.
- [Built-in Features overview](../built-in-features.md)
