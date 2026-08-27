---
title: Component styles
description: Official CSS entry points for ssg: false hosts and transformAllPlugins() consumers.
---

# Component styles

The built-in SSG inlines feature CSS next to generated HTML. Custom hosts that
set `ssg: false`, call `transformAllPlugins()`, or own the document with
`ssg.render` get the same markup but not those styles.

`@ox-content/vite-plugin` publishes the crate stylesheets the SSG already uses.
Import what you render. Site-specific theming stays in your app.

```css
@import "@ox-content/vite-plugin/styles/core.css";
@import "@ox-content/vite-plugin/styles/markdown-tables.css";
@import "@ox-content/vite-plugin/styles/magic-links.css";
@import "@ox-content/vite-plugin/styles/social.css";
@import "@ox-content/vite-plugin/styles/twitter-full.css";
```

Or pull every feature sheet:

```css
@import "@ox-content/vite-plugin/styles/all.css";
```

`transformAllPlugins()` still returns HTML only. CSS is an explicit import so
you can load compact Tweet chrome without the full-card sheet.

## Entry points

| Import                       | Covers                                                                                                         |
| ---------------------------- | -------------------------------------------------------------------------------------------------------------- |
| `styles/core.css`            | Base tokens (`--octc-*`) and default prose/chrome from the SSG stylesheet                                      |
| `styles/markdown-tables.css` | Responsive Markdown table scroll containers and focus rings, without prose or theme globals                    |
| `styles/magic-links.css`     | `{link:...}` chips                                                                                             |
| `styles/social.css`          | Compact Tweet/X, Reddit, Bluesky, provider cards, Spotify, Apple Music, audio, video, StackBlitz, WebContainer |
| `styles/twitter-full.css`    | `appearance: "full"` Tweet cards, including the react-tweet / sveltweet MIT notice                             |
| `styles/ogp.css`             | Open Graph cards                                                                                               |
| `styles/github.css`          | GitHub repository and source cards                                                                             |
| `styles/youtube.css`         | YouTube embeds                                                                                                 |
| `styles/tabs.css`            | Tabs and package-manager tabs                                                                                  |
| `styles/mermaid.css`         | Mermaid diagrams                                                                                               |
| `styles/graphviz.css`        | Graphviz DOT diagrams                                                                                          |
| `styles/citations.css`       | Citation links and generated bibliography sections                                                             |
| `styles/not-by-ai.css`       | `<NotByAI />` authorship badge                                                                                 |
| `styles/all.css`             | The feature sheets above, in that order                                                                        |

Feature sheets that use `var(--octc-*)` expect `core.css` first, or the same
tokens defined on your host. Full Tweet chrome defines its own `--ox-tweet-*`
variables and does not require `core.css`.

These files are copied from `crates/ox_content_ssg` at package build time. The
built-in SSG includes the same sources, so official chrome cannot drift from
what custom hosts import.

## Custom hosts

Module transformer (`ssg: false`):

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      srcDir: "content",
      ssg: false,
      embeds: { twitter: { fetch: true, appearance: "full" } },
    }),
  ],
};
```

```css
@import "@ox-content/vite-plugin/styles/core.css";
@import "@ox-content/vite-plugin/styles/social.css";
@import "@ox-content/vite-plugin/styles/twitter-full.css";
```

Direct `transformAllPlugins()`:

```ts
import { transformAllPlugins } from "@ox-content/vite-plugin";

const html = await transformAllPlugins(sourceHtml, {
  twitter: { fetch: true, appearance: "full" },
});
```

Import the matching stylesheets in the host that renders `html`. Do not copy
crate CSS into the app.

`renderMarkdown()` and `createMarkdownProcessor()` follow the same rule: they
return markup, and you import the official sheets for the features you enabled.

For responsive Markdown tables in a custom host, import only
`styles/markdown-tables.css` when you already own body typography, prose width,
links, blockquotes, and table cell styling. Use `core.css` only when you want
the full built-in SSG prose theme. Add the framework-neutral helper when your
host owns the document or serves transformed Markdown through a custom dev
server:

```ts
import { enhanceMarkdownTables } from "@ox-content/vite-plugin/markdown-tables";

enhanceMarkdownTables(document, {
  label: "Scrollable table",
});

window.addEventListener("resize", () => enhanceMarkdownTables(document));
```

The helper measures `.content table` and adds `tabindex="0"` only when a table
actually overflows. It preserves table semantics, captions, headers, direction,
and existing accessible names. Pass a localized `label` from the same locale
path as the rest of your host chrome.

## Related

- [Site Generation](./site-generation.md)
- [Magic Links](./magic-links.md)
- [NotByAI Badge](./not-by-ai.md)
- [Embeds](./embeds.md)
- [Twitter/X Embed](../examples/twitter-embed.md)
- [@ox-content/vite-plugin](../packages/vite-plugin-ox-content.md)
