---
title: Theme Presets
description: Ready-made Ox Content themes — pick a skin for the form, a color scheme for the palette, and compose them.
---

# Theme Presets

Ox Content splits a theme into two independent axes, published as two families
of packages:

- **Skins** (`@ox-content/theme-*`) own **form** — geometry, texture,
  typography and motion. A skin is written entirely against `--octc-*` custom
  properties and never names a color.
- **Color schemes** (`@ox-content/theme-color-*`) own **color** — a light and a
  dark palette, and nothing else. No layout, no texture, no fonts.

Because neither half knows about the other, any skin pairs with any scheme:
**18 skins × 22 schemes = 396 combinations**.

## Gallery

**[Browse every combination →](/theme-gallery.html)**

The gallery renders each pairing with the real SSG stylesheet inside an iframe,
so what you see is what a built site looks like — including light and dark, the
landing page and an article page, and the live WebGL backdrops.

## Quick start

Install one of each:

```bash
npm install @ox-content/theme-liquid-glass @ox-content/theme-color-tokyo-night
```

Then list them as layers. `theme` accepts an array, and layers compose left to
right:

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import liquidGlass from "@ox-content/theme-liquid-glass";
import tokyoNight from "@ox-content/theme-color-tokyo-night";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      ssg: {
        siteName: "My Docs",
        theme: [liquidGlass, tokyoNight],
      },
    }),
  ],
});
```

Anything you append wins, so your own overrides go last:

```ts
theme: [
  liquidGlass,
  tokyoNight,
  {
    colors: { primary: "#ff5f56" },
    footer: { copyright: "Copyright © 2026 My Company" },
  },
];
```

Either axis works on its own. A scheme with no skin restyles the default theme;
a skin with no scheme reshapes the default palette.

## Skins

Each skin is roughly 6–7 kB of CSS with **zero JavaScript** and no runtime
dependencies.

| Package                                                                                      | Name         | Description                                                                  |
| -------------------------------------------------------------------------------------------- | ------------ | ---------------------------------------------------------------------------- |
| [`@ox-content/theme-pixel`](https://npmjs.com/package/@ox-content/theme-pixel)               | Pixel        | Chunky 8-bit surfaces, hard offset shadows and stepped motion                |
| [`@ox-content/theme-liquid-glass`](https://npmjs.com/package/@ox-content/theme-liquid-glass) | Liquid Glass | Refractive glass panels with specular edges and a light sweep on hover       |
| [`@ox-content/theme-blur-glass`](https://npmjs.com/package/@ox-content/theme-blur-glass)     | Blur Glass   | Frosted backdrop-blur layers that resolve out of a soft haze                 |
| [`@ox-content/theme-analog-film`](https://npmjs.com/package/@ox-content/theme-analog-film)   | Analog Film  | Grain, halation and sprocket rails with a gentle gate weave                  |
| [`@ox-content/theme-fluid`](https://npmjs.com/package/@ox-content/theme-fluid)               | Fluid        | Organic blob gradients that drift and morph behind the content               |
| [`@ox-content/theme-fabric`](https://npmjs.com/package/@ox-content/theme-fabric)             | Fabric       | Woven texture, stitched seams and soft cloth-fold reveals                    |
| [`@ox-content/theme-leather`](https://npmjs.com/package/@ox-content/theme-leather)           | Leather      | Embossed grain and saddle stitching that presses in when touched             |
| [`@ox-content/theme-brutalist`](https://npmjs.com/package/@ox-content/theme-brutalist)       | Brutalist    | Raw structure, oversized type and shadows that slam into place               |
| [`@ox-content/theme-terminal`](https://npmjs.com/package/@ox-content/theme-terminal)         | Terminal     | CRT phosphor with scanlines, a blinking block caret and prompt gutters       |
| [`@ox-content/theme-blueprint`](https://npmjs.com/package/@ox-content/theme-blueprint)       | Blueprint    | Technical drawing grid with dashed callouts and strokes that draw themselves |
| [`@ox-content/theme-risograph`](https://npmjs.com/package/@ox-content/theme-risograph)       | Risograph    | Misregistered duotone print where the ink channels separate on hover         |
| [`@ox-content/theme-swiss`](https://npmjs.com/package/@ox-content/theme-swiss)               | Swiss        | International Typographic Style — a hard grid, rules, and precise slides     |
| [`@ox-content/theme-neon`](https://npmjs.com/package/@ox-content/theme-neon)                 | Neon         | Sunset-grid glow with humming tube outlines and a scanline sweep             |
| [`@ox-content/theme-clay`](https://npmjs.com/package/@ox-content/theme-clay)                 | Clay         | Soft extruded clay that squishes under the pointer                           |
| [`@ox-content/theme-editorial`](https://npmjs.com/package/@ox-content/theme-editorial)       | Editorial    | Magazine typography with drop caps, hairline rules and column reveals        |
| [`@ox-content/theme-aurora`](https://npmjs.com/package/@ox-content/theme-aurora)             | Aurora       | Slow conic light curtains drifting behind translucent panels                 |
| [`@ox-content/theme-holo`](https://npmjs.com/package/@ox-content/theme-holo)                 | Holo         | Iridescent foil that shifts hue as panels tilt and slide                     |
| [`@ox-content/theme-paper`](https://npmjs.com/package/@ox-content/theme-paper)               | Paper        | Letterpress impressions on soft stock with a deckled page edge               |

## Color schemes

Every scheme ships a matched light **and** dark palette; the built-in header
toggle switches between them with no extra configuration.

| Package                                                                                                    | Name          | Description                                         |
| ---------------------------------------------------------------------------------------------------------- | ------------- | --------------------------------------------------- |
| [`@ox-content/theme-color-github`](https://npmjs.com/package/@ox-content/theme-color-github)               | GitHub        | GitHub Light and GitHub Dark                        |
| [`@ox-content/theme-color-tokyo-night`](https://npmjs.com/package/@ox-content/theme-color-tokyo-night)     | Tokyo Night   | Tokyo Night Day and Tokyo Night Storm               |
| [`@ox-content/theme-color-mono`](https://npmjs.com/package/@ox-content/theme-color-mono)                   | Mono          | Pure monochrome, zero hue                           |
| [`@ox-content/theme-color-dracula`](https://npmjs.com/package/@ox-content/theme-color-dracula)             | Dracula       | Dracula with the Alucard light counterpart          |
| [`@ox-content/theme-color-one-dark`](https://npmjs.com/package/@ox-content/theme-color-one-dark)           | One Dark      | Atom One Light and One Dark                         |
| [`@ox-content/theme-color-retro`](https://npmjs.com/package/@ox-content/theme-color-retro)                 | Retro         | Warm amber phosphor terminal                        |
| [`@ox-content/theme-color-snow`](https://npmjs.com/package/@ox-content/theme-color-snow)                   | Snow          | Crisp snow white over deep slate                    |
| [`@ox-content/theme-color-catppuccin`](https://npmjs.com/package/@ox-content/theme-color-catppuccin)       | Catppuccin    | Catppuccin Latte and Mocha                          |
| [`@ox-content/theme-color-nord`](https://npmjs.com/package/@ox-content/theme-color-nord)                   | Nord          | Nord Snow Storm and Polar Night                     |
| [`@ox-content/theme-color-gruvbox`](https://npmjs.com/package/@ox-content/theme-color-gruvbox)             | Gruvbox       | Gruvbox light and dark, medium contrast             |
| [`@ox-content/theme-color-rose-pine`](https://npmjs.com/package/@ox-content/theme-color-rose-pine)         | Rosé Pine     | Rosé Pine Dawn and Rosé Pine                        |
| [`@ox-content/theme-color-solarized`](https://npmjs.com/package/@ox-content/theme-color-solarized)         | Solarized     | Ethan Schoonover's Solarized Light and Dark         |
| [`@ox-content/theme-color-everforest`](https://npmjs.com/package/@ox-content/theme-color-everforest)       | Everforest    | Everforest light and dark, medium contrast          |
| [`@ox-content/theme-color-ayu`](https://npmjs.com/package/@ox-content/theme-color-ayu)                     | Ayu           | Ayu Light and Ayu Mirage                            |
| [`@ox-content/theme-color-vitesse`](https://npmjs.com/package/@ox-content/theme-color-vitesse)             | Vitesse       | Anthony Fu's Vitesse light and dark                 |
| [`@ox-content/theme-color-night-owl`](https://npmjs.com/package/@ox-content/theme-color-night-owl)         | Night Owl     | Sarah Drasner's Light Owl and Night Owl             |
| [`@ox-content/theme-color-monokai`](https://npmjs.com/package/@ox-content/theme-color-monokai)             | Monokai       | Monokai Pro with a light companion                  |
| [`@ox-content/theme-color-kanagawa`](https://npmjs.com/package/@ox-content/theme-color-kanagawa)           | Kanagawa      | Kanagawa Lotus and Wave                             |
| [`@ox-content/theme-color-poimandres`](https://npmjs.com/package/@ox-content/theme-color-poimandres)       | Poimandres    | Poimandres, cool teal on deep navy                  |
| [`@ox-content/theme-color-sepia`](https://npmjs.com/package/@ox-content/theme-color-sepia)                 | Sepia         | Low-glare sepia tuned for long reading sessions     |
| [`@ox-content/theme-color-high-contrast`](https://npmjs.com/package/@ox-content/theme-color-high-contrast) | High Contrast | Maximum-contrast scheme aimed at WCAG AAA body text |
| [`@ox-content/theme-color-synthwave`](https://npmjs.com/package/@ox-content/theme-color-synthwave)         | Synthwave     | Sunset-grid synthwave with hot magenta accents      |

## Syntax highlighting

Highlighting follows the color scheme in both modes, with no extra
configuration. `highlightTheme` defaults to `'css-variables'`, so Shiki emits
token colors as `--octc-shiki-*` custom properties and each scheme defines them
per mode — one build, two palettes.

Schemes pick syntax colors against the **code background**, not the page
background. Several light schemes (`mono`, `snow`, `nord`, `retro`, `monokai`,
`poimandres`, `synthwave`) deliberately keep a dark code block in light mode; a
fixed theme would put dark token colors on it and make it unreadable.

With no scheme installed the properties fall back to GitHub Dark, which is what
the default used to be. To bake fixed colors in instead, name any bundled Shiki
theme:

```ts
oxContent({ highlightTheme: "vitesse-dark" });
```

Override individual tokens like any other token:

```ts
theme: [pixel, tokyoNight, { darkTokens: { "shiki-token-comment": "#5a6a8a" } }];
```

## Motion

Presets are motion-first, and all of it is declarative CSS:

- **Cross-document page transitions** via `@view-transition { navigation: auto }`.
  The header and sidebar are given `view-transition-name`s so they hold still
  while the article swaps — an app-like feel with no router and no client bundle.
- **Scroll-driven reveals** via `animation-timeline: view()`. Headings, code
  blocks, tables and feature cards rise as they enter, with the feature grid
  staggered by column.
- **Dialog entry** via `@starting-style`, so the search modal animates open
  without a JavaScript-toggled class.

Everything sits behind `@supports`, so an engine that lacks a feature simply
renders the finished, static state — the un-animated state is never a hidden
one. All of it is disabled under `prefers-reduced-motion: reduce`.

Retune the choreography without touching CSS. Every skin exposes its timing as
tokens:

| Token                  | Purpose                               |
| ---------------------- | ------------------------------------- |
| `--octc-motion-fast`   | Color and background transitions      |
| `--octc-motion-base`   | Elevation, transform, page swap       |
| `--octc-motion-slow`   | Hero entrance, specular sweeps        |
| `--octc-motion-ease`   | The skin's signature easing curve     |
| `--octc-motion-spring` | Overshoot curve for lifts and presses |
| `--octc-motion-rise`   | Distance a revealed block travels     |

```ts
theme: [liquidGlass, tokyoNight, { tokens: { "motion-base": "200ms", "motion-ease": "linear" } }];
```

## Live backdrops

Three skins render their hero backdrop on the GPU instead of faking it in CSS:

| Skin           | What it renders                                                                                                                  |
| -------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| `liquid-glass` | True refraction — a domain-warped height field becomes a normal map that bends the lookup into the backdrop, with a Fresnel edge |
| `fluid`        | Dye advected through a curl-noise velocity field, which is divergence-free and so reads as an incompressible liquid              |
| `fabric`       | A woven height field lit by a drifting key light, so threads catch the light along their own direction                           |

All three are hand-written WebGL2 with **no dependencies** — no Three.js, no
bundle, nothing to install. They ship inside the theme's own `js`, which the SSG
emits as one inline script.

They are strictly progressive enhancement, and every one of these leaves the
page exactly as it renders without JavaScript:

- `prefers-reduced-motion: reduce` — never starts
- No WebGL2 context, or a shader that fails to compile — never starts
- Hero scrolled out of view, or tab backgrounded — stops drawing
- Any error at all — swallowed, since the CSS backdrop underneath is the design

Colors are read from the live palette through the same `--octc-*` properties, so
a backdrop follows its color scheme and re-reads on every theme toggle.

## Custom tokens

`tokens` and `darkTokens` set any `--octc-*` custom property, keyed without the
prefix. They merge key-by-key across layers, so overriding one value never means
redeclaring the rest:

```ts
theme: [
  liquidGlass,
  tokyoNight,
  {
    tokens: { "surface-glass": "#f8fafc", "code-line-add": "rgba(0,200,120,.18)" },
    darkTokens: { "surface-glass": "#0c1324" },
  },
];
```

Token names are validated at build time — a name that could break out of its
declaration block fails the build rather than shipping a corrupt stylesheet.

## How composition works

`resolveTheme` flattens every layer's `extends` chain, then merges the result:

- Object fields (`colors`, `tokens`, `layout`, `fonts`, …) merge **key-by-key**,
  last layer winning.
- `css` and `js` **concatenate** in layer order — overwriting them would discard
  one half of a skin + scheme stack. Identical fragments are joined once, so a
  layer reached through both an array and an `extends` chain is not emitted
  twice.

That ordering is why a skin declares its surface variables through `tokens`
rather than raw `css`: tokens resolve declaratively before any stylesheet runs,
which keeps the outcome independent of the order you list the packages in.

## Writing your own

A preset is a plain [`ThemeConfig`](/theming.md) — there is no plugin API to
implement. A skin sets `css`, `fonts`, `layout` and motion `tokens`; a scheme
sets `colors`, `darkColors`, `tokens` and `darkTokens`.

```ts
import { defineTheme } from "@ox-content/vite-plugin";

export default defineTheme({
  name: "my-skin",
  fonts: { sans: "Inter, system-ui, sans-serif" },
  tokens: { "motion-base": "260ms", "motion-ease": "cubic-bezier(.2,0,0,1)" },
  css: `.header { border-bottom: 2px solid var(--octc-color-primary); }`,
});
```

The one rule that keeps a skin composable: **never name a color.** Reference
`--octc-color-*` and `--octc-accent-*`, and reach for `color-mix()` when you
need a tint. Neutral black/white alpha for depth is fine, since it reads
correctly over any palette.
