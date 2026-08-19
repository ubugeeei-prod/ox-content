import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Pixel — Chunky 8-bit surfaces, hard offset shadows and stepped motion.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [pixel, tokyoNight]
 * ```
 */
export const pixel: ThemeConfig = {
  name: "pixel",
  fonts: {
    sans: '"DotGothic16", "Silkscreen", ui-monospace, SFMono-Regular, Menlo, monospace',
    mono: '"Silkscreen", "DotGothic16", ui-monospace, SFMono-Regular, Menlo, monospace',
  },
  layout: {
    sidebarWidth: "252px",
    headerHeight: "58px",
    maxContentWidth: "920px",
  },
  entryPage: { mode: "default" },
  embed: {
    head: '<link rel="preconnect" href="https://fonts.googleapis.com"><link rel="preconnect" href="https://fonts.gstatic.com" crossorigin><link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Silkscreen:wght@400;700&amp;family=DotGothic16&amp;display=swap">',
  },
  tokens: {
    "motion-fast": "128ms",
    "motion-base": "238ms",
    "motion-slow": "486ms",
    "motion-ease": "steps(4, end)",
    "motion-spring": "steps(5, end)",
    "motion-rise": "0.38rem",
  },
  css,
};

export default pixel;
