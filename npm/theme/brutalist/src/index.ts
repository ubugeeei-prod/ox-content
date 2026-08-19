import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Brutalist — Raw structure, oversized type and shadows that slam into place.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [brutalist, tokyoNight]
 * ```
 */
export const brutalist: ThemeConfig = {
  name: "brutalist",
  fonts: {
    sans: '"Helvetica Neue", Helvetica, Inter, "Segoe UI", Arial, sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "256px",
    headerHeight: "72px",
    maxContentWidth: "980px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "112ms",
    "motion-base": "204ms",
    "motion-slow": "351ms",
    "motion-ease": "cubic-bezier(0.9, 0, 0.1, 1)",
    "motion-spring": "cubic-bezier(0.9, 0, 0.1, 1)",
    "motion-rise": "0.3rem",
  },
  css,
};

export default brutalist;
