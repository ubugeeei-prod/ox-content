import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Neon — Sunset-grid glow with humming tube outlines and a scanline sweep.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [neon, tokyoNight]
 * ```
 */
export const neon: ThemeConfig = {
  name: "neon",
  fonts: {
    sans: 'ui-sans-serif, system-ui, -apple-system, "Segoe UI Variable", "Segoe UI", Roboto, "Helvetica Neue", sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "264px",
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
    "motion-rise": "0.75rem",
  },
  css,
};

export default neon;
