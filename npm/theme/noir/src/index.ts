import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Noir — One hard key light and a steep falloff — shapes picked out of the dark.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [noir, tokyoNight]
 * ```
 */
export const noir: ThemeConfig = {
  name: "noir",
  fonts: {
    sans: 'ui-serif, "Iowan Old Style", "Hoefler Text", Georgia, "Times New Roman", serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "252px",
    headerHeight: "66px",
    maxContentWidth: "840px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "200ms",
    "motion-base": "420ms",
    "motion-slow": "900ms",
    "motion-ease": "cubic-bezier(0.33,1,0.68,1)",
    "motion-spring": "cubic-bezier(0.33,1,0.68,1)",
    "motion-rise": "0.5rem",
  },
  css,
};

export default noir;
