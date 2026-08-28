# @ox-content/theme-color-material

Material — Material Theme Lighter and Material Theme — for [Ox Content](https://github.com/ubugeeei-prod/ox-content).

**Color only.** Light and dark `--octc-*` tokens and nothing else: no layout, no
texture, no typography. That is what lets it drop under any skin package.

```bash
npm install @ox-content/theme-color-material
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import material from "@ox-content/theme-color-material";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      ssg: { siteName: "My Docs", theme: material },
    }),
  ],
});
```

Stack a skin on top to change the form as well — layers compose left to right,
so anything you append wins:

```ts
import pixel from "@ox-content/theme-pixel";

theme: [pixel, material, { colors: { primary: "#ff5f56" } }];
```

## Variants

The default export stays `material` (Material Theme Lighter and Material Theme). Named variants
are exported from the same package:

```ts
import { materialDarker, materialOcean, materialPalenight } from "@ox-content/theme-color-material";
```

- `materialDarker` — Material Theme Lighter and Material Theme Darker
- `materialOcean` — Material Theme Lighter and Material Theme Ocean
- `materialPalenight` — Material Theme Lighter and Material Theme Palenight

## License

MIT
