---
title: Self-hosted Iconify CSS
description: Opt-in build-time Iconify CSS so SSG pages do not request api.iconify.design.
---

# Self-hosted Iconify CSS

Ox Content can resolve Iconify names at build time and emit a small CSS-mask
stylesheet for the icons that are actually used. Entry-page feature icons and
custom social links reuse the same resolver. The published site does not request
`https://api.iconify.design`.

Off by default. Install `@iconify/json` or individual `@iconify-json/*`
packages so the build can read collections from disk. Tests and CI should use
those packages or a local fixture — the resolver never fetches the Iconify API.

```ts
import { oxContent } from "@ox-content/vite-plugin";

export default {
  plugins: [
    oxContent({
      icons: {
        mode: "css-mask",
        syntax: "unocss",
        include: ["src/**/*.{md,mdx,svelte,ts,json}"],
        safelist: ["carbon:checkbox", "ri:markdown-line"],
      },
    }),
  ],
};
```

`include` also accepts explicit names when you do not want to scan sources:

```ts
oxContent({
  icons: {
    include: ["ri:markdown-line", "line-md:rss", "ph:github-logo-duotone"],
  },
});
```

| Option     | Default      | Purpose                                                       |
| ---------- | ------------ | ------------------------------------------------------------- |
| `mode`     | `"css-mask"` | Monochrome icons become `currentColor` CSS masks.             |
| `syntax`   | `"unocss"`   | Emits `icon-[prefix--name]` classes.                          |
| `include`  | `[]`         | Glob patterns to scan, or explicit `prefix:name` icons.       |
| `safelist` | `[]`         | Names that are always emitted, including dynamic class names. |

The build also collects Iconify names from:

- entry-page `features[].icon` frontmatter
- theme `socialLinks` array entries whose `icon` is `prefix:name`

Missing collections or icon names are reported as build errors. Unused icons in
an installed collection are not written to CSS.

Generated CSS is written to `__ox_icons__/icons.css` and linked from the theme
`<head>`, next to [self-hosted fonts](../theming.md#fonts). Existing
`icon-[prefix--name]` markup can keep working while you migrate templates.
Custom Vite hosts can import `virtual:ox-content/assets.css` or read
`virtual:ox-content/asset-manifest`; the same stylesheet is served in dev and
written during production builds.

`false` or omitted keeps the current CDN fallback on entry-page Iconify icons.
