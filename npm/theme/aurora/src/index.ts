import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Aurora — Slow conic light curtains drifting behind translucent panels.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [aurora, tokyoNight]
 * ```
 */
export const aurora: ThemeConfig = {
  name: "aurora",
  fonts: {
    sans: 'ui-sans-serif, system-ui, -apple-system, "Segoe UI Variable", "Segoe UI", Roboto, "Helvetica Neue", sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "268px",
    headerHeight: "66px",
    maxContentWidth: "940px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "220ms",
    "motion-base": "420ms",
    "motion-slow": "1000ms",
    "motion-ease": "cubic-bezier(0.16, 1, 0.3, 1)",
    "motion-spring": "cubic-bezier(0.34, 1.4, 0.64, 1)",
    "motion-rise": "0.88rem",
  },
  css,
};

export default aurora;
