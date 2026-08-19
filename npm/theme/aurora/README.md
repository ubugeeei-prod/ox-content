# @ox-content/theme-aurora

Aurora — Slow conic light curtains drifting behind translucent panels — for [Ox Content](https://github.com/ubugeeei-prod/ox-content).

**Form only.** Geometry, texture, typography and motion, written entirely
against `--octc-*` custom properties. It names no colors, so it pairs with any
`@ox-content/theme-color-*` scheme. About 9.7 kB of CSS, zero JavaScript, zero
runtime dependencies.

```bash
npm install @ox-content/theme-aurora @ox-content/theme-color-tokyo-night
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import aurora from "@ox-content/theme-aurora";
import tokyoNight from "@ox-content/theme-color-tokyo-night";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      ssg: { siteName: "My Docs", theme: [aurora, tokyoNight] },
    }),
  ],
});
```

Layers compose left to right, so anything you append wins:

```ts
theme: [aurora, tokyoNight, { colors: { primary: "#ff5f56" } }];
```

## Motion

Transitions, scroll-driven reveals and cross-document page transitions are all
declarative CSS — no router, no observer, no client bundle. Everything sits
behind `@supports` and is switched off under `prefers-reduced-motion: reduce`.

Retune the choreography without touching the stylesheet:

```ts
theme: [aurora, tokyoNight, { tokens: { "motion-base": "200ms", "motion-ease": "linear" } }];
```

## License

MIT
