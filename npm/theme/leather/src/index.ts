import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Leather — Embossed grain and saddle stitching that presses in when touched.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [leather, tokyoNight]
 * ```
 */
export const leather: ThemeConfig = {
  name: "leather",
  fonts: {
    sans: 'ui-serif, "Iowan Old Style", "Hoefler Text", Georgia, "Times New Roman", serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "266px",
    headerHeight: "68px",
    maxContentWidth: "880px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "220ms",
    "motion-base": "420ms",
    "motion-slow": "864ms",
    "motion-ease": "cubic-bezier(0.36, 0.66, 0.04, 1)",
    "motion-spring": "cubic-bezier(0.36, 0.66, 0.04, 1)",
    "motion-rise": "0.45rem",
  },
  css,
};

export default leather;
