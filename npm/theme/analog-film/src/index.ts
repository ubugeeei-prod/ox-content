import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Analog Film — Grain, halation and sprocket rails with a gentle gate weave.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [analogFilm, tokyoNight]
 * ```
 */
export const analogFilm: ThemeConfig = {
  name: "analog-film",
  fonts: {
    sans: '"Arial Narrow", "Helvetica Neue Condensed", "Roboto Condensed", "Segoe UI", sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "262px",
    headerHeight: "64px",
    maxContentWidth: "900px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "220ms",
    "motion-base": "420ms",
    "motion-slow": "945ms",
    "motion-ease": "cubic-bezier(0.4, 0, 0.2, 1)",
    "motion-spring": "cubic-bezier(0.3, 1.3, 0.6, 1)",
    "motion-rise": "0.5rem",
  },
  css,
};

export default analogFilm;
