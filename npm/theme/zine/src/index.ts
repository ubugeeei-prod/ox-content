import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Zine — Photocopied and taped together — nothing square to the page.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [zine, tokyoNight]
 * ```
 */
export const zine: ThemeConfig = {
  name: "zine",
  fonts: {
    sans: '"Helvetica Neue", Helvetica, Inter, "Segoe UI", Arial, sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "250px",
    headerHeight: "62px",
    maxContentWidth: "880px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "90ms",
    "motion-base": "170ms",
    "motion-slow": "380ms",
    "motion-ease": "cubic-bezier(0.9,0,0.1,1)",
    "motion-spring": "cubic-bezier(0.34,1.6,0.64,1)",
    "motion-rise": "0.4rem",
  },
  css,
};

export default zine;
