import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";
import { js } from "./gl";

/**
 * Fluid — Organic blob gradients that drift and morph behind the content.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [fluid, tokyoNight]
 * ```
 */
export const fluid: ThemeConfig = {
  name: "fluid",
  fonts: {
    sans: '"SF Pro Rounded", ui-rounded, "Hiragino Maru Gothic ProN", "Varela Round", system-ui, sans-serif',
    mono: 'ui-monospace, SFMono-Regular, "SF Mono", Menlo, "Cascadia Mono", Consolas, monospace',
  },
  layout: {
    sidebarWidth: "270px",
    headerHeight: "68px",
    maxContentWidth: "920px",
  },
  entryPage: { mode: "default" },
  tokens: {
    "motion-fast": "220ms",
    "motion-base": "420ms",
    "motion-slow": "1000ms",
    "motion-ease": "cubic-bezier(0.65, 0, 0.35, 1)",
    "motion-spring": "cubic-bezier(0.34, 1.56, 0.64, 1)",
    "motion-rise": "0.88rem",
  },
  css,
  js,
};

export default fluid;
