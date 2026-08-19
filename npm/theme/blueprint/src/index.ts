import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Blueprint — Technical drawing grid with dashed callouts and strokes that draw themselves.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [blueprint, tokyoNight]
 * ```
 */
export const blueprint: ThemeConfig = {
  name: "blueprint",
  fonts: {
    sans: '"Arial Narrow", "Helvetica Neue Condensed", "Roboto Condensed", "Segoe UI", sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "260px",
    headerHeight: "60px",
    maxContentWidth: "920px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "192ms",
    "motion-base": "408ms",
    "motion-slow": "837ms",
    "motion-ease": "cubic-bezier(0.25, 0.8, 0.25, 1)",
    "motion-spring": "cubic-bezier(0.25, 0.8, 0.25, 1)",
    "motion-rise": "0.42rem",
  },
  css,
};

export default blueprint;
