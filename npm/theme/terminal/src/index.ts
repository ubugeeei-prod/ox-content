import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Terminal — CRT phosphor with scanlines, a blinking block caret and prompt gutters.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [terminal, tokyoNight]
 * ```
 */
export const terminal: ThemeConfig = {
  name: "terminal",
  fonts: {
    sans: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "248px",
    headerHeight: "56px",
    maxContentWidth: "880px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "96ms",
    "motion-base": "204ms",
    "motion-slow": "405ms",
    "motion-ease": "steps(6, end)",
    "motion-spring": "linear",
    "motion-rise": "0.25rem",
  },
  css,
};

export default terminal;
