---
title: Theming
description: Customize the appearance of your documentation site with ox-content's Theme API.
---

# Theming

ox-content provides a flexible Theme API that allows you to customize the appearance of your documentation site. You can use CSS variables for simple customization or write full JSX themes for complete control.

Prefer not to build one from scratch? The [Theme Presets](/theme-presets.md)
**official catalog** ships 27 skins and 45 color schemes as
`@ox-content/theme-*` and `@ox-content/theme-color-*` packages that compose
through `ssg.theme`. See [Authoring a package](/theme-presets.md#authoring-a-package)
for the compatibility contract (required tokens, light and dark, screenshots,
and the rule that skins must not hard-code colors).

## Stable MPA Navigation

The built-in theme restores the saved light, dark, or system color preference
before the first paint. On browsers that support cross-document View
Transitions, same-origin page changes keep the current surface visible while
the next generated page loads. This remains an MPA: links still perform normal
document navigation, and unsupported browsers use their native fallback.

The transition is disabled automatically for `prefers-reduced-motion: reduce`.
To opt out for a theme, set `viewTransitions: false`:

```ts
defineTheme({
  viewTransitions: false,
});
```

External links, downloads, and hash-only links retain normal browser behavior.

## Theme Toggle Reveal

`viewTransitions` covers navigation between documents. The theme toggle is a
_same-document_ change, and it switches instantly by default. Opt into a
circular reveal that grows out of wherever the reader activated the control:

```ts
defineTheme({
  toggleTransition: "circle",
});
```

Switching to dark grows the incoming snapshot over the outgoing one; switching
to light collapses the outgoing snapshot to reveal the new palette underneath.
A pointer press reveals from the pointer, and a keyboard or programmatic
activation reveals from the centre of the control. Browsers without View
Transitions, and readers who ask for `prefers-reduced-motion: reduce`, keep the
immediate switch.

The two options are independent: `toggleTransition` never touches the
cross-document snapshots, and its stylesheet is scoped to an attribute the
runtime holds for the length of one toggle.

### From a custom host

A host that renders its own toggle can drive the same primitive instead of
reimplementing it. `apply` is your synchronous theme mutation — the reveal
wraps it, it does not own your state, markup, or icons:

```ts
import { applyThemeTransition } from "@ox-content/vite-plugin/theme-transition/client";
import "@ox-content/vite-plugin/styles/theme-transition.css";

button.addEventListener("click", (event) => {
  const next = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
  void applyThemeTransition({
    event,
    nextTheme: next,
    apply: () => setTheme(next),
  });
});
```

| Option      | Default         | Purpose                                              |
| ----------- | --------------- | ---------------------------------------------------- |
| `apply`     | —               | Synchronous theme mutation. Required.                |
| `event`     | —               | Activation event. Supplies the reveal origin.        |
| `nextTheme` | —               | `"light"` collapses the circle; anything else grows. |
| `duration`  | `420`           | Reveal duration in milliseconds.                     |
| `easing`    | `"ease-in-out"` | Reveal easing.                                       |

The returned promise resolves once the transition settles, including when it is
skipped, so a rapid double-toggle never leaves an unhandled rejection. The
stylesheet uses the public `data-theme` contract and pulls in no framework.

Prior art: the circular reveal is @hooray's VitePress implementation, by way of
[@ryoppippi](https://github.com/ryoppippi)'s
[svelte-fancy-darkmode](https://github.com/ryoppippi/svelte-fancy-darkmode).

## Localized Sidebar Labels

Every sidebar `text` accepts either one string or a locale map. The same map
works for top-level groups, linked parent items, and nested items:

```ts
defineTheme({
  sidebar: [
    {
      text: { en: "Guide", ja: "ガイド" },
      collapsed: true,
      stickyCollapsed: true,
      items: [
        {
          text: { en: "Built-in features", ja: "組み込み機能" },
          link: "/built-in-features.md",
          items: [{ text: { en: "Cards", ja: "カード" }, link: "/cards.md" }],
        },
      ],
    },
  ],
});
```

Resolution is deterministic: the exact page locale, its language subtag, the
configured default locale, its language subtag, then the first non-empty map
value. Labels are HTML-escaped. Localized links use an existing sibling page;
if that sibling is missing, the authored href remains valid. Sticky collapse
state uses the navigation tree position, so changing locale does not reset it.

## Quick Start

### CSS Variable Customization

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent, defineTheme, defaultTheme } from "@ox-content/vite-plugin";

export default defineConfig({
  plugins: [
    oxContent({
      ssg: {
        siteName: "My Docs",
        theme: defineTheme({
          extends: defaultTheme,
          colors: {
            primary: "#3498db",
          },
          socialLinks: {
            github: "https://github.com/your/repo",
          },
          footer: {
            message: "Released under the MIT License.",
            copyright: "Copyright © 2024 My Company",
          },
        }),
      },
    }),
  ],
});
```

### JSX Theme (Full Control)

ox-content supports JSX/TSX themes that render to static HTML with **zero client-side JavaScript** by default.

```tsx
// theme/Layout.tsx
import { usePageProps, useSiteConfig, useNav, raw, each } from "@ox-content/vite-plugin";

export function Layout({ children }) {
  const page = usePageProps();
  const site = useSiteConfig();
  const nav = useNav();

  return (
    <html lang="en">
      <head>
        <meta charset="UTF-8" />
        <title>
          {page.title} - {site.name}
        </title>
      </head>
      <body>
        <nav>
          {each(nav, (group) => (
            <div>
              <h3>{group.title}</h3>
              <ul>
                {each(group.items, (item) => (
                  <li>
                    <a href={item.href}>{item.title}</a>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </nav>
        <main>{children}</main>
      </body>
    </html>
  );
}
```

Configure your `tsconfig.json` for JSX:

```json
{
  "compilerOptions": {
    "jsx": "react-jsx",
    "jsxImportSource": "@ox-content/vite-plugin"
  }
}
```

## CSS Variables Reference

Every theme color, layout dimension, and font stack is emitted as a
`--octc-`-prefixed CSS custom property on `:root`. You can set them through the
theme config (below) or override them directly from [custom CSS](#custom-css-and-javascript)
— the variable is the single source of truth either way.

### Colors

| Option                     | CSS Variable                 | Description                                                    |
| -------------------------- | ---------------------------- | -------------------------------------------------------------- |
| `colors.primary`           | `--octc-color-primary`       | Primary accent color for links, active states                  |
| `colors.primaryHover`      | `--octc-color-primary-hover` | Primary color on hover                                         |
| `colors.background`        | `--octc-color-bg`            | Main background color                                          |
| `colors.backgroundAlt`     | `--octc-color-bg-alt`        | Alternative background (sidebar, code blocks)                  |
| `colors.text`              | `--octc-color-text`          | Main text color                                                |
| `colors.textMuted`         | `--octc-color-text-muted`    | Muted/secondary text color                                     |
| `colors.border`            | `--octc-color-border`        | Border color                                                   |
| `colors.codeBackground`    | `--octc-color-code-bg`       | Code block background                                          |
| `colors.codeBackgroundTop` | `--octc-color-code-bg-top`   | Code block gradient top; follows `codeBackground` when omitted |
| `colors.codeText`          | `--octc-color-code-text`     | Code block text color                                          |

### Layout

| Option                   | CSS Variable               | Description                              |
| ------------------------ | -------------------------- | ---------------------------------------- |
| `layout.sidebarWidth`    | `--octc-sidebar-width`     | Sidebar width (default: `260px`)         |
| `layout.headerHeight`    | `--octc-header-height`     | Header height (default: `60px`)          |
| `layout.maxContentWidth` | `--octc-max-content-width` | Maximum content width (default: `960px`) |

### Fonts

| Option        | CSS Variable         | Description                                 |
| ------------- | -------------------- | ------------------------------------------- |
| `fonts.sans`  | `--octc-font-sans`   | Sans-serif font stack or self-hosted family |
| `fonts.mono`  | `--octc-font-mono`   | Monospace font stack or self-hosted family  |
| `fonts.named` | `--octc-font-<name>` | Extra families for custom theme CSS         |

`sans` and `mono` accept either a CSS stack string or a web-font object. The
string form is unchanged:

```ts
fonts: {
  sans: "Inter, sans-serif",
  mono: "DM Mono, monospace",
}
```

The object form names a family. With `selfHost: true`, Ox Content copies the
requested weights and subsets into the SSG output and emits `@font-face`, so
the published site does not request Google Fonts at runtime:

```ts
fonts: {
  sans: {
    family: "Inter",
    provider: "google",
    weights: [400, 600],
    subsets: ["latin"],
    display: "swap",
    selfHost: true,
  },
  mono: "DM Mono, monospace",
  named: {
    code: {
      family: "JetBrains Mono",
      provider: "google",
      weights: [400],
      selfHost: true,
    },
  },
}
```

- `sans` / `mono` still map to `--octc-font-sans` and `--octc-font-mono`.
- `named` families expose `--octc-font-<name>` (for example `--octc-font-code`).
- `provider: "local"` reads a file or an `@fontsource/*` directory and needs no
  network. Use it in CI or when you already vendor the files.
- `preload: true` (or a weight list) emits `<link rel="preload">` for those
  faces.
- Downloads are cached under `node_modules/.cache/ox-content/fonts`.

Object families without `selfHost: true` only set the CSS stack; they do not
download or emit font files.

### Self-hosted assets in a custom host

The built-in SSG theme links self-hosted fonts and Iconify CSS automatically.
Custom hosts own their document shell, so they can use the Vite virtual asset
contract instead:

```ts
import "virtual:ox-content/assets.css";

// Or, when the server renderer owns <head>:
import { headTags } from "virtual:ox-content/asset-manifest";
```

Use the CSS import when the host's client entry owns styles. Use `headTags` (or
the exported `stylesheets` and `preloads`) when the server renderer owns
`<head>`. Both paths use the same `__ox_fonts__` and `__ox_icons__` URLs as the
built-in theme, work in dev, and write local assets during production builds.

Keep the theme visible to the plugin even when Ox Content is not rendering
pages:

```ts
oxContent({
  icons: { safelist: ["carbon:checkbox"] },
  ssg: {
    enabled: false,
    theme: {
      fonts: {
        sans: {
          family: "Inter",
          provider: "local",
          path: "@fontsource/inter",
          weights: [400, 600],
          selfHost: true,
        },
      },
    },
  },
});
```

Boolean `ssg: false` still disables SSG, but it has no place to carry the
theme. Use `ssg: { enabled: false, theme }` for self-hosted assets in a
bare/custom Vite host.

Only the keys you set are emitted. Omitted colors, fonts, and layout values fall
back to the [default theme](#default-theme-values), so overriding a single accent
never forces you to redeclare the rest of the palette.

## Dark Mode

`colors` defines the light palette and `darkColors` defines the dark one; Ox
Content emits both from a single build and switches between them with two
selectors:

- `[data-theme="dark"]` — used when the page (or the reader, via the built-in
  header theme toggle) explicitly opts into dark mode. The toggle persists the
  choice in `localStorage` so it survives navigation.
- `@media (prefers-color-scheme: dark) { :root:not([data-theme="light"]) { … } }`
  — honours the operating-system preference unless the reader has explicitly
  chosen light.

```ts
defineTheme({
  extends: defaultTheme,
  colors: { primary: "#3b82f6", background: "#ffffff" },
  darkColors: { primary: "#60a5fa", background: "#060816" },
});
```

`darkColors` follows the same key-by-key fallback as `colors`: any key you leave
out inherits the default dark palette.

## Theme Tokens in a Bare or Custom Host

`ssg.bare: true` and custom hosts render their own document, so Ox Content emits
no theme stylesheet for them. `renderThemeTokenCss()` returns the same `--octc-*`
declarations the built-in SSG would have written, from a subpath that pulls in
neither the Vite plugin, the SSG, the native binding, nor a filesystem API:

```ts
import { renderThemeTokenCss } from "@ox-content/vite-plugin/theme-tokens";
import { kanagawa } from "@ox-content/theme-color-kanagawa";

const css = renderThemeTokenCss(kanagawa);
```

Kanagawa's default export keeps Lotus light + Wave dark. Import
`kanagawaDragon` from the same package when a custom host needs Lotus light +
the canonical Dragon dark syntax palette:

```ts
import { kanagawaDragon } from "@ox-content/theme-color-kanagawa";

const css = renderThemeTokenCss(kanagawaDragon);
```

The built-in highlighter emits `var(--octc-syntax-*)` references, so a host that
wants a scheme's code colors while keeping its own page palette, typography, and
layout can select tokens by name. Names arrive without the `--octc-` prefix:

```ts
const syntaxOnly = renderThemeTokenCss(kanagawa, {
  include: (name) => name.startsWith("syntax-"),
});
```

The output uses the three selectors described under Dark Mode above — `:root`,
`[data-theme="dark"]`, and the `prefers-color-scheme` fallback that an explicit
light choice still overrides — because this is the renderer the built-in SSG
itself calls.

Layers compose exactly as `resolveTheme()` composes them: pass an array to stack
a skin and a color scheme, and each layer's `extends` chain is flattened
base-first.

```ts
import { pixel } from "@ox-content/theme-pixel";

const css = renderThemeTokenCss([pixel, kanagawa]);
```

Token names are lowercase kebab-case. An empty or malformed name throws instead
of emitting a broken custom property, and a token with an empty value is
skipped. The function is also re-exported from the package root, next to the
lower-level `tokensToCss(light, dark)`, when you are already importing the
plugin.

## Entry Page Modes

The default theme supports two landing page modes:

- `default` - the more branded, marketing-style entry page
- `subtle` - a quieter docs.rs-like presentation with tighter spacing and a more restrained hero

```ts
defineTheme({
  extends: defaultTheme,
  entryPage: {
    mode: "subtle",
  },
});
```

## Page Outline

The default theme can render a right-hand "On this page" outline from the page
headings. It is **off by default**. Set `aside: true` to enable it; the outline
still appears only on pages that have TOC entries. Entry pages skip the outline.

```ts
defineTheme({
  extends: defaultTheme,
  aside: true,
});
```

When enabled, the markup stays `<aside class="toc">` plus `main--with-toc` on
the article column — the same chrome as before this became opt-in. Existing
sites that want the outline must set `theme.aside: true`.

## Heading Permalinks

Visible `#` links on headings are **off by default**. Enable
`headingPermalinks: true` so the renderer appends
`<a class="header-anchor" href="#id">` using the exact generated id. Then
`theme.headingPermalink` chooses only the CSS presentation:

```ts
defineTheme({
  extends: defaultTheme,
  headingPermalink: "always",
});
```

`"hover"` (the default) reveals the control on hover and `:focus-visible`,
and keeps it visible on touch. `"always"` keeps it visible. The heading HTML
does not change. See [Heading Permalinks](./built-in/heading-permalinks.md).

## Page Props & Hooks

Access page data in your theme components using hooks:

### `usePageProps()`

Returns the current page's data:

```tsx
function PageHeader() {
  const page = usePageProps();

  return (
    <header>
      <h1>{page.title}</h1>
      {page.description && <p>{page.description}</p>}
    </header>
  );
}
```

**Available properties:**

- `title` - Page title
- `description` - Page description
- `html` - Rendered HTML content
- `toc` - Table of contents
- `path` - Source file path
- `url` - Output URL
- `frontmatter` - Raw frontmatter object
- `layout` - Layout name

### `useSiteConfig()`

Returns site-wide configuration:

```tsx
function SiteHeader() {
  const site = useSiteConfig();

  return <header>{site.name}</header>;
}
```

### `useNav()`

Returns navigation groups:

```tsx
function Sidebar() {
  const nav = useNav();

  return (
    <nav>
      {each(nav, (group) => (
        <section>
          <h3>{group.title}</h3>
          {each(group.items, (item) => (
            <a href={item.href}>{item.title}</a>
          ))}
        </section>
      ))}
    </nav>
  );
}
```

### `useIsActive(path)`

Checks if a path is the current page:

```tsx
function NavLink({ href, children }) {
  const isActive = useIsActive(href);

  return (
    <a href={href} class={isActive ? "active" : ""}>
      {children}
    </a>
  );
}
```

## JSX Utilities

### `raw(html)`

Renders raw HTML without escaping:

```tsx
<div>{raw(page.html)}</div>
```

### `each(items, render)`

Maps over arrays:

```tsx
{
  each(items, (item, index) => <li key={index}>{item.name}</li>);
}
```

### `when(condition, content)`

Conditional rendering:

```tsx
{
  when(page.toc.length > 0, <aside class="toc">...</aside>);
}
```

## Type Generation

ox-content auto-generates TypeScript types based on your pages' frontmatter. The generated types are saved to your output directory.

```ts
// Generated: page-props.d.ts
export interface PageFrontmatter {
  title: string;
  description?: string;
  layout?: string;
  // ... other fields from your frontmatter
}

export type PageProps = import("@ox-content/vite-plugin").PageProps<PageFrontmatter>;
```

Use the generated types:

```tsx
import type { PageProps } from "./page-props";

function Layout() {
  const page = usePageProps<PageProps["frontmatter"]>();
  // page.frontmatter is now fully typed
}
```

## Layout Switching

Support multiple layouts based on frontmatter:

```tsx
// theme/index.tsx
import { createTheme } from "@ox-content/vite-plugin";
import { DefaultLayout } from "./layouts/Default";
import { EntryLayout } from "./layouts/Entry";
import { BlogLayout } from "./layouts/Blog";

export default createTheme({
  layouts: {
    default: DefaultLayout,
    entry: EntryLayout,
    blog: BlogLayout,
  },
});
```

In your markdown:

```md
---
layout: entry
title: Welcome
---

# Welcome to My Docs
```

## Social Links

Add social links to the header. The shorthand form covers the common networks:

```ts
defineTheme({
  extends: defaultTheme,
  socialLinks: {
    github: "https://github.com/your/repo",
    twitter: "https://twitter.com/yourhandle",
    discord: "https://discord.gg/yourserver",
  },
});
```

For anything else, pass an array of `{ icon, link, label? }` entries. The
`icon` field accepts several formats:

| Format                | Example                       | Renders as                               |
| --------------------- | ----------------------------- | ---------------------------------------- |
| Iconify `prefix:name` | `"mdi:mastodon"`              | Iconify icon (any set), color-aware      |
| Lucide                | `"lucide:rss"`                | Lucide icon via Iconify                  |
| Image URL             | `"https://example.com/x.svg"` | `<img>` with that source                 |
| Local path            | `"/icons/x.svg"`              | `<img>` resolved against the site `base` |
| Emoji / text          | `"📡"`                        | Rendered inline as-is                    |

```ts
defineTheme({
  extends: defaultTheme,
  socialLinks: [
    { icon: "mdi:mastodon", link: "https://mastodon.social/@you", label: "Mastodon" },
    { icon: "lucide:rss", link: "/feed.xml", label: "RSS" },
  ],
});
```

Inline SVG passed as an icon is sanitized — `<script>` is stripped — so an icon
string can never inject executable markup.

Enable [`icons`](./built-in/icons.md) to emit local CSS masks for these Iconify
names (and for entry-page feature icons) instead of `api.iconify.design`.

## Embedded HTML (Slots)

The `embed` option injects raw HTML at fixed points in the page layout. All nine
positions are optional:

| Field           | Renders…                                                   |
| --------------- | ---------------------------------------------------------- |
| `head`          | inside `<head>` (analytics, `preconnect`, custom `<meta>`) |
| `headerBefore`  | immediately before the header bar                          |
| `headerAfter`   | immediately after the header bar                           |
| `sidebarBefore` | at the top of the sidebar, before the navigation           |
| `sidebarAfter`  | at the bottom of the sidebar, after the navigation         |
| `contentBefore` | before the main content (above the article)                |
| `contentAfter`  | after the main content (below the article)                 |
| `footerBefore`  | immediately before the footer                              |
| `footer`        | replaces the default footer entirely                       |

```ts
defineTheme({
  extends: defaultTheme,
  embed: {
    head: '<link rel="preconnect" href="https://fonts.googleapis.com">',
    headerBefore: '<div class="announcement">New version!</div>',
    contentAfter: '<div class="feedback">Was this helpful?</div>',
    footer: '<footer class="custom">© My Project</footer>',
  },
});
```

Embedded HTML is inserted verbatim, so only pass trusted markup.

## Custom CSS and JavaScript

`css` is appended **after** the generated `--octc-*` variable overrides, so your
rules win on specificity ties and you can freely read or redefine the variables.
`js` is injected as an inline script on every page.

```ts
defineTheme({
  extends: defaultTheme,
  css: `
    /* Override a generated variable for every page… */
    :root {
      --octc-max-content-width: 1100px;
    }
    /* …or target the rendered markup directly. */
    .content h1 {
      color: var(--octc-color-primary);
      letter-spacing: -0.04em;
    }
  `,
  js: `
    console.log('Page loaded');
  `,
});
```

For one-off tweaks you can also pass `css` straight to the `ssg` plugin option
without defining a full theme — it is merged the same way:

```ts
oxContent({
  ssg: {
    theme: { css: ".hero-name { letter-spacing: -0.04em; }" },
  },
});
```

## Default Theme Values

```ts
const defaultTheme = {
  name: "default",
  aside: false,
  headingPermalink: "hover",
  colors: {
    primary: "#3b82f6",
    primaryHover: "#2563eb",
    background: "#ffffff",
    backgroundAlt: "#f5f7fb",
    text: "#131a30",
    textMuted: "#4f607b",
    border: "#d2dbea",
    codeBackground: "#0b1328",
    codeText: "#eaf2ff",
  },
  darkColors: {
    primary: "#60a5fa",
    primaryHover: "#93c5fd",
    background: "#060816",
    backgroundAlt: "#0d1528",
    text: "#ebf2ff",
    textMuted: "#8ea0bf",
    border: "#223252",
    codeBackground: "#0a1020",
    codeText: "#e7f0ff",
  },
  fonts: {
    sans: '"IBM Plex Sans", "Avenir Next", "Segoe UI Variable", "Segoe UI", sans-serif',
    mono: '"IBM Plex Mono", "SFMono-Regular", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "260px",
    headerHeight: "60px",
    maxContentWidth: "960px",
  },
  socialLinks: {},
};
```

## TypeScript Support

All types are exported:

```ts
import type {
  ThemeConfig,
  ThemeColors,
  ThemeLayout,
  ThemeFonts,
  ThemeHeader,
  ThemeFooter,
  SocialLinks,
  ThemeEmbed,
  ResolvedThemeConfig,
  PageProps,
  BasePageProps,
  SiteConfig,
  NavGroup,
  NavItem,
  ThemeComponent,
  ThemeProps,
} from "@ox-content/vite-plugin";
```
