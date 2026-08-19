import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Manuscript — A codex page — narrow measure, rubricated heads, and a table of contents that becomes marginalia.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [manuscript, tokyoNight]
 * ```
 */
export const manuscript: ThemeConfig = {
  name: "manuscript",
  fonts: {
    sans: 'ui-serif, "Iowan Old Style", "Hoefler Text", Georgia, "Times New Roman", serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "238px",
    headerHeight: "62px",
    maxContentWidth: "720px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "200ms",
    "motion-base": "400ms",
    "motion-slow": "860ms",
    "motion-ease": "cubic-bezier(0.33,1,0.68,1)",
    "motion-spring": "cubic-bezier(0.33,1,0.68,1)",
    "motion-rise": "0.5rem",
  },
  css,
};

export default manuscript;
