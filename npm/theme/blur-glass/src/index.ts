import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Blur Glass — Frosted backdrop-blur layers that resolve out of a soft haze.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [blurGlass, tokyoNight]
 * ```
 */
export const blurGlass: ThemeConfig = {
  name: "blur-glass",
  fonts: {
    sans: 'ui-sans-serif, system-ui, -apple-system, "Segoe UI Variable", "Segoe UI", Roboto, "Helvetica Neue", sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "268px",
    headerHeight: "64px",
    maxContentWidth: "940px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "220ms",
    "motion-base": "420ms",
    "motion-slow": "1000ms",
    "motion-ease": "cubic-bezier(0.16, 1, 0.3, 1)",
    "motion-spring": "cubic-bezier(0.16, 1, 0.3, 1)",
    "motion-rise": "0.62rem",
  },
  css,
};

export default blurGlass;
