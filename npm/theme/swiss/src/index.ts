import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Swiss — International Typographic Style — a hard grid, rules, and precise slides.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [swiss, tokyoNight]
 * ```
 */
export const swiss: ThemeConfig = {
  name: "swiss",
  fonts: {
    sans: '"Helvetica Neue", Helvetica, Inter, "Segoe UI", Arial, sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "244px",
    headerHeight: "60px",
    maxContentWidth: "860px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "208ms",
    "motion-base": "420ms",
    "motion-slow": "810ms",
    "motion-ease": "cubic-bezier(0.2, 0, 0, 1)",
    "motion-spring": "cubic-bezier(0.2, 0, 0, 1)",
    "motion-rise": "0.35rem",
  },
  css,
};

export default swiss;
