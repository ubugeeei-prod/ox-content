import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Holo — Iridescent foil that shifts hue as panels tilt and slide.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [holo, tokyoNight]
 * ```
 */
export const holo: ThemeConfig = {
  name: "holo",
  fonts: {
    sans: 'ui-sans-serif, system-ui, -apple-system, "Segoe UI Variable", "Segoe UI", Roboto, "Helvetica Neue", sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "266px",
    headerHeight: "66px",
    maxContentWidth: "920px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "220ms",
    "motion-base": "420ms",
    "motion-slow": "1000ms",
    "motion-ease": "cubic-bezier(0.22, 1, 0.36, 1)",
    "motion-spring": "cubic-bezier(0.34, 1.56, 0.64, 1)",
    "motion-rise": "0.7rem",
  },
  css,
};

export default holo;
