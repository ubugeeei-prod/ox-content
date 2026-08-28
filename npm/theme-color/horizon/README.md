# @ox-content/theme-color-horizon

Horizon — Warm coral and plum at dusk — for [Ox Content](https://github.com/ubugeeei-prod/ox-content).

**Color only.** Light and dark `--octc-*` tokens and nothing else: no layout, no
texture, no typography. That is what lets it drop under any skin package.

```bash
npm install @ox-content/theme-color-horizon
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import horizon from "@ox-content/theme-color-horizon";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      ssg: { siteName: "My Docs", theme: horizon },
    }),
  ],
});
```

Stack a skin on top to change the form as well — layers compose left to right,
so anything you append wins:

```ts
import pixel from "@ox-content/theme-pixel";

theme: [pixel, horizon, { colors: { primary: "#ff5f56" } }];
```

## Variants

The default export stays `horizon` (Warm coral and plum at dusk). Named variants
are exported from the same package:

```ts
import { horizonBright } from "@ox-content/theme-color-horizon";
```

- `horizonBright` — Horizon Bright and Horizon

## License

MIT
