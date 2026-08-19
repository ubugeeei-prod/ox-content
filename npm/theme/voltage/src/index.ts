import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";
import { js } from "./gl";

/**
 * Voltage — Oversized display type with charged gradient edges and a live electric field.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [voltage, tokyoNight]
 * ```
 */
export const voltage: ThemeConfig = {
  name: "voltage",
  fonts: {
    sans: 'ui-sans-serif, system-ui, -apple-system, "Segoe UI Variable", "Segoe UI", Roboto, "Helvetica Neue", sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "262px",
    headerHeight: "66px",
    maxContentWidth: "960px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "180ms",
    "motion-base": "360ms",
    "motion-slow": "900ms",
    "motion-ease": "cubic-bezier(0.2, 0.9, 0.2, 1)",
    "motion-spring": "cubic-bezier(0.34, 1.5, 0.64, 1)",
    "motion-rise": "0.7rem",
  },
  css,
  js,
};

export default voltage;
