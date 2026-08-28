# @ox-content/theme-color-github

GitHub — GitHub Light and GitHub Dark — for [Ox Content](https://github.com/ubugeeei-prod/ox-content).

**Color only.** Light and dark `--octc-*` tokens and nothing else: no layout, no
texture, no typography. That is what lets it drop under any skin package.

```bash
npm install @ox-content/theme-color-github
```

```ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import github from "@ox-content/theme-color-github";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      ssg: { siteName: "My Docs", theme: github },
    }),
  ],
});
```

Stack a skin on top to change the form as well — layers compose left to right,
so anything you append wins:

```ts
import pixel from "@ox-content/theme-pixel";

theme: [pixel, github, { colors: { primary: "#ff5f56" } }];
```

## Variants

The default export stays `github` (GitHub Light and GitHub Dark). Named variants
are exported from the same package:

```ts
import {
  githubClassic,
  githubDefault,
  githubDimmed,
  githubHighContrast,
} from "@ox-content/theme-color-github";
```

- `githubClassic` — GitHub Light and Dark from @shikijs/themes
- `githubDefault` — GitHub Light Default and Dark Default
- `githubDimmed` — GitHub Light Default and Dark Dimmed
- `githubHighContrast` — GitHub Light High Contrast and Dark High Contrast

## License

MIT
