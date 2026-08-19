import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Atlas — A survey sheet — contour bands, a keyed legend sidebar and registration marks.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [atlas, tokyoNight]
 * ```
 */
export const atlas: ThemeConfig = {
  name: "atlas",
  fonts: {
    sans: '"Optima", "Gill Sans", "Gill Sans MT", "Segoe UI", ui-sans-serif, sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "250px",
    headerHeight: "60px",
    maxContentWidth: "860px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "200ms",
    "motion-base": "380ms",
    "motion-slow": "820ms",
    "motion-ease": "cubic-bezier(0.25,0.8,0.25,1)",
    "motion-spring": "cubic-bezier(0.25,0.8,0.25,1)",
    "motion-rise": "0.4rem",
  },
  css,
};

export default atlas;
