# @ox-content/theme-color-rose-pine

Rosé Pine — Rosé Pine Dawn and Rosé Pine — for [Ox Content](https://github.com/ubugeeei-prod/ox-content).

**Color only.** Light and dark `--octc-*` tokens and nothing else: no layout, no
texture, no typography. That is what lets it drop under any skin package.

```bash
npm install @ox-content/theme-color-rose-pine
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import rosePine from "@ox-content/theme-color-rose-pine";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      ssg: { siteName: "My Docs", theme: rosePine },
    }),
  ],
});
```

Stack a skin on top to change the form as well — layers compose left to right,
so anything you append wins:

```ts
import pixel from "@ox-content/theme-pixel";

theme: [pixel, rosePine, { colors: { primary: "#ff5f56" } }];
```

## Variants

The default export stays `rosePine` (Rosé Pine Dawn and Rosé Pine). Named variants
are exported from the same package:

```ts
import { rosePineMoon } from "@ox-content/theme-color-rose-pine";
```

- `rosePineMoon` — Rosé Pine Dawn and Rosé Pine Moon

## License

MIT
