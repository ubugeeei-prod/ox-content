# @ox-content/theme-color-dracula

Dracula — Dracula with the Alucard light counterpart — for [Ox Content](https://github.com/ubugeeei-prod/ox-content).

**Color only.** Light and dark `--octc-*` tokens and nothing else: no layout, no
texture, no typography. That is what lets it drop under any skin package.

```bash
npm install @ox-content/theme-color-dracula
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import dracula from "@ox-content/theme-color-dracula";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      ssg: { siteName: "My Docs", theme: dracula },
    }),
  ],
});
```

Stack a skin on top to change the form as well — layers compose left to right,
so anything you append wins:

```ts
import pixel from "@ox-content/theme-pixel";

theme: [pixel, dracula, { colors: { primary: "#ff5f56" } }];
```

## Variants

The default export stays `dracula` (Dracula with the Alucard light counterpart). Named variants
are exported from the same package:

```ts
import { draculaSoft } from "@ox-content/theme-color-dracula";
```

- `draculaSoft` — Dracula Alucard light companion and Dracula Soft

## License

MIT
