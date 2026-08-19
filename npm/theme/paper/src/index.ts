import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Paper — Letterpress impressions on soft stock with a deckled page edge.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [paper, tokyoNight]
 * ```
 */
export const paper: ThemeConfig = {
  name: "paper",
  fonts: {
    sans: 'ui-serif, "Iowan Old Style", "Hoefler Text", Georgia, "Times New Roman", serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "258px",
    headerHeight: "66px",
    maxContentWidth: "820px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "220ms",
    "motion-base": "420ms",
    "motion-slow": "918ms",
    "motion-ease": "cubic-bezier(0.33, 1, 0.68, 1)",
    "motion-spring": "cubic-bezier(0.34, 1.3, 0.64, 1)",
    "motion-rise": "0.45rem",
  },
  css,
};

export default paper;
