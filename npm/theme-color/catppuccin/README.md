# @ox-content/theme-color-catppuccin

Catppuccin — Catppuccin Latte and Mocha — for [Ox Content](https://github.com/ubugeeei-prod/ox-content).

**Color only.** Light and dark `--octc-*` tokens and nothing else: no layout, no
texture, no typography. That is what lets it drop under any skin package.

```bash
npm install @ox-content/theme-color-catppuccin
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import catppuccin from "@ox-content/theme-color-catppuccin";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      ssg: { siteName: "My Docs", theme: catppuccin },
    }),
  ],
});
```

Stack a skin on top to change the form as well — layers compose left to right,
so anything you append wins:

```ts
import pixel from "@ox-content/theme-pixel";

theme: [pixel, catppuccin, { colors: { primary: "#ff5f56" } }];
```

## Variants

The default export stays `catppuccin` (Catppuccin Latte and Mocha). Named variants
are exported from the same package:

```ts
import { catppuccinFrappe, catppuccinMacchiato } from "@ox-content/theme-color-catppuccin";
```

- `catppuccinFrappe` — Catppuccin Latte and Frappé
- `catppuccinMacchiato` — Catppuccin Latte and Macchiato

## License

MIT
