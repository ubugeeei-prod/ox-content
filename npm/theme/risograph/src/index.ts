import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Risograph — Misregistered duotone print where the ink channels separate on hover.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [risograph, tokyoNight]
 * ```
 */
export const risograph: ThemeConfig = {
  name: "risograph",
  fonts: {
    sans: '"Helvetica Neue", Helvetica, Inter, "Segoe UI", Arial, sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "258px",
    headerHeight: "62px",
    maxContentWidth: "900px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "176ms",
    "motion-base": "391ms",
    "motion-slow": "756ms",
    "motion-ease": "cubic-bezier(0.2, 0.9, 0.3, 1)",
    "motion-spring": "cubic-bezier(0.34, 1.5, 0.64, 1)",
    "motion-rise": "0.5rem",
  },
  css,
};

export default risograph;
