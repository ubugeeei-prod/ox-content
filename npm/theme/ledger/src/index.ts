import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Ledger — A bound accounting book — text set on real ruled lines, tabular figures throughout.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [ledger, tokyoNight]
 * ```
 */
export const ledger: ThemeConfig = {
  name: "ledger",
  fonts: {
    sans: '"Optima", "Gill Sans", "Gill Sans MT", "Segoe UI", ui-sans-serif, sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "252px",
    headerHeight: "60px",
    maxContentWidth: "880px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "160ms",
    "motion-base": "300ms",
    "motion-slow": "640ms",
    "motion-ease": "cubic-bezier(0.2,0,0,1)",
    "motion-spring": "cubic-bezier(0.2,0,0,1)",
    "motion-rise": "0.4rem",
  },
  css,
};

export default ledger;
