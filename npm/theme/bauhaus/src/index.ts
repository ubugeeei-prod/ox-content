import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Bauhaus — Circle, square, triangle and a diagonal that refuses the grid.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [bauhaus, tokyoNight]
 * ```
 */
export const bauhaus: ThemeConfig = {
  name: "bauhaus",
  fonts: {
    sans: '"Helvetica Neue", Helvetica, Inter, "Segoe UI", Arial, sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "256px",
    headerHeight: "64px",
    maxContentWidth: "900px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "140ms",
    "motion-base": "260ms",
    "motion-slow": "560ms",
    "motion-ease": "cubic-bezier(0.65,0,0.35,1)",
    "motion-spring": "cubic-bezier(0.34,1.5,0.64,1)",
    "motion-rise": "0.5rem",
  },
  css,
};

export default bauhaus;
