import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Editorial — Magazine typography with drop caps, hairline rules and column reveals.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [editorial, tokyoNight]
 * ```
 */
export const editorial: ThemeConfig = {
  name: "editorial",
  fonts: {
    sans: 'ui-serif, "Iowan Old Style", "Hoefler Text", Georgia, "Times New Roman", serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "250px",
    headerHeight: "72px",
    maxContentWidth: "780px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "220ms",
    "motion-base": "420ms",
    "motion-slow": "945ms",
    "motion-ease": "cubic-bezier(0.33, 1, 0.68, 1)",
    "motion-spring": "cubic-bezier(0.33, 1, 0.68, 1)",
    "motion-rise": "0.5rem",
  },
  css,
};

export default editorial;
