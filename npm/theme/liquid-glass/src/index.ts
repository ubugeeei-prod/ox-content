import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";

/**
 * Liquid Glass — Refractive glass panels with specular edges and a light sweep on hover.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any `@ox-content/theme-color-*` package:
 *
 * ```ts
 * theme: [liquidGlass, tokyoNight]
 * ```
 */
export const liquidGlass: ThemeConfig = {
  name: "liquid-glass",
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
  embed: {
    head: '<svg aria-hidden="true" width="0" height="0" style="position:absolute;width:0;height:0;overflow:hidden"><filter id="octc-lg-lens" x="0%" y="0%" width="100%" height="100%" color-interpolation-filters="sRGB"><feImage href="data:image/svg+xml,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%22100%22%20height%3D%22100%22%3E%3Cdefs%3E%3ClinearGradient%20id%3D%22x%22%20x1%3D%220%22%20y1%3D%220%22%20x2%3D%221%22%20y2%3D%220%22%3E%3Cstop%20offset%3D%220%22%20stop-color%3D%22rgb%280%2C0%2C0%29%22%2F%3E%3Cstop%20offset%3D%220.17%22%20stop-color%3D%22rgb%28128%2C0%2C0%29%22%2F%3E%3Cstop%20offset%3D%220.83%22%20stop-color%3D%22rgb%28128%2C0%2C0%29%22%2F%3E%3Cstop%20offset%3D%221%22%20stop-color%3D%22rgb%28255%2C0%2C0%29%22%2F%3E%3C%2FlinearGradient%3E%3ClinearGradient%20id%3D%22y%22%20x1%3D%220%22%20y1%3D%220%22%20x2%3D%220%22%20y2%3D%221%22%3E%3Cstop%20offset%3D%220%22%20stop-color%3D%22rgb%280%2C0%2C0%29%22%2F%3E%3Cstop%20offset%3D%220.17%22%20stop-color%3D%22rgb%280%2C128%2C0%29%22%2F%3E%3Cstop%20offset%3D%220.83%22%20stop-color%3D%22rgb%280%2C128%2C0%29%22%2F%3E%3Cstop%20offset%3D%221%22%20stop-color%3D%22rgb%280%2C255%2C0%29%22%2F%3E%3C%2FlinearGradient%3E%3Cfilter%20id%3D%22mix%22%3E%3CfeBlend%20mode%3D%22screen%22%2F%3E%3C%2Ffilter%3E%3C%2Fdefs%3E%3Crect%20width%3D%22100%22%20height%3D%22100%22%20fill%3D%22url%28%23x%29%22%2F%3E%3Crect%20width%3D%22100%22%20height%3D%22100%22%20fill%3D%22url%28%23y%29%22%20style%3D%22mix-blend-mode%3Ascreen%22%2F%3E%3C%2Fsvg%3E" result="map" preserveAspectRatio="none"/><feDisplacementMap in="SourceGraphic" in2="map" scale="26" xChannelSelector="R" yChannelSelector="G"/></filter></svg>',
  },
  tokens: {
    "motion-fast": "220ms",
    "motion-base": "420ms",
    "motion-slow": "1000ms",
    "motion-ease": "cubic-bezier(0.22, 1, 0.36, 1)",
    "motion-spring": "cubic-bezier(0.34, 1.56, 0.64, 1)",
    "motion-rise": "0.75rem",
  },
  css,
};

export default liquidGlass;
