import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Kiosk — Platform signage — a departure-board sidebar, banded heads and arrows.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [kiosk, tokyoNight]
 * ```
 */
export const kiosk: ThemeConfig = {
  name: "kiosk",
  fonts: {
    sans: '"Helvetica Neue", Helvetica, Inter, "Segoe UI", Arial, sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "248px",
    headerHeight: "64px",
    maxContentWidth: "920px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "120ms",
    "motion-base": "220ms",
    "motion-slow": "460ms",
    "motion-ease": "cubic-bezier(0.3,0,0,1)",
    "motion-spring": "cubic-bezier(0.3,0,0,1)",
    "motion-rise": "0.4rem",
  },
  css,
};

export default kiosk;
