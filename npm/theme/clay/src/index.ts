import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Clay — Soft extruded clay that squishes under the pointer.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [clay, tokyoNight]
 * ```
 */
export const clay: ThemeConfig = {
  name: "clay",
  fonts: {
    sans: '"SF Pro Rounded", ui-rounded, "Hiragino Maru Gothic ProN", "Varela Round", system-ui, sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "272px",
    headerHeight: "70px",
    maxContentWidth: "900px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "220ms",
    "motion-base": "420ms",
    "motion-slow": "1000ms",
    "motion-ease": "cubic-bezier(0.34, 1.56, 0.64, 1)",
    "motion-spring": "cubic-bezier(0.34, 1.8, 0.64, 1)",
    "motion-rise": "0.75rem",
  },
  css,
};

export default clay;
