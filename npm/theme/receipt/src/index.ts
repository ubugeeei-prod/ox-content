import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Receipt — A thermal roll — one narrow centred column, dotted leaders and a torn edge.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [receipt, tokyoNight]
 * ```
 */
export const receipt: ThemeConfig = {
  name: "receipt",
  fonts: {
    sans: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "0px",
    headerHeight: "0px",
    maxContentWidth: "620px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "120ms",
    "motion-base": "220ms",
    "motion-slow": "440ms",
    "motion-ease": "steps(8, end)",
    "motion-spring": "linear",
    "motion-rise": "0.3rem",
  },
  css,
};

export default receipt;
