# @ox-content/theme-paper

Paper — Letterpress impressions on soft stock with a deckled page edge — for [Ox Content](https://github.com/ubugeeei-prod/ox-content).

**Form only.** Geometry, texture, typography and motion, written entirely
against `--octc-*` custom properties. It names no colors, so it pairs with any
`@ox-content/theme-color-*` scheme. About 10.2 kB of CSS, zero JavaScript, zero
runtime dependencies.

```bash
npm install @ox-content/theme-paper @ox-content/theme-color-tokyo-night
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import paper from "@ox-content/theme-paper";
import tokyoNight from "@ox-content/theme-color-tokyo-night";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      ssg: { siteName: "My Docs", theme: [paper, tokyoNight] },
    }),
  ],
});
```

Layers compose left to right, so anything you append wins:

```ts
theme: [paper, tokyoNight, { colors: { primary: "#ff5f56" } }];
```

## Motion

Transitions, scroll-driven reveals and cross-document page transitions are all
declarative CSS — no router, no observer, no client bundle. Everything sits
behind `@supports` and is switched off under `prefers-reduced-motion: reduce`.

Retune the choreography without touching the stylesheet:

```ts
theme: [paper, tokyoNight, { tokens: { "motion-base": "200ms", "motion-ease": "linear" } }];
```

## License

MIT
