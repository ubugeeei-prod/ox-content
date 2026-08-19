import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";
import { js } from "./gl";

/**
 * Fabric — Woven texture, stitched seams and soft cloth-fold reveals.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [fabric, tokyoNight]
 * ```
 */
export const fabric: ThemeConfig = {
  name: "fabric",
  fonts: {
    sans: 'ui-sans-serif, system-ui, -apple-system, "Segoe UI Variable", "Segoe UI", Roboto, "Helvetica Neue", sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "264px",
    headerHeight: "64px",
    maxContentWidth: "900px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "220ms",
    "motion-base": "420ms",
    "motion-slow": "1000ms",
    "motion-ease": "cubic-bezier(0.33, 1, 0.68, 1)",
    "motion-spring": "cubic-bezier(0.34, 1.4, 0.64, 1)",
    "motion-rise": "0.62rem",
  },
  css,
  js,
};

export default fabric;
