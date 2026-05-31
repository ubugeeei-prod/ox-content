import type { NapiModule, NapiSlideTheme } from "./internal-types";
import type { SlideThemeConfig } from "./public-types";

let napiModulePromise: Promise<NapiModule> | undefined;

/**
 * Loads the slide-aware N-API bindings on demand.
 */
export async function loadNapiModule(): Promise<NapiModule> {
  napiModulePromise ??= import("@ox-content/napi").then(
    (module) => module as unknown as NapiModule,
  );
  return napiModulePromise;
}

/**
 * Converts plugin theme config into the N-API shape.
 */
export function toNapiTheme(theme: SlideThemeConfig, animations = true): NapiSlideTheme {
  return {
    aspectRatio: theme.aspectRatio,
    maxWidth: theme.maxWidth,
    maxHeight: theme.maxHeight,
    padding: theme.padding,
    surfaceRadius: theme.surfaceRadius,
    codeBackground: theme.codeBackground,
    builtinAnimations: animations,
    canvasBackground: theme.canvasBackground,
    surfaceBackground: theme.surfaceBackground,
    surfaceBorder: theme.surfaceBorder,
    surfaceShadow: theme.surfaceShadow,
    presenterSidebarBackground: theme.presenterSidebarBackground,
    fontSans: theme.fontSans,
    fontMono: theme.fontMono,
    colorText: theme.colorText,
    colorTextMuted: theme.colorTextMuted,
    colorPrimary: theme.colorPrimary,
    colorBorder: theme.colorBorder,
    darkCanvasBackground: theme.darkCanvasBackground,
    darkSurfaceBackground: theme.darkSurfaceBackground,
    darkSurfaceBorder: theme.darkSurfaceBorder,
    darkPresenterSidebarBackground: theme.darkPresenterSidebarBackground,
    darkCodeBackground: theme.darkCodeBackground,
    darkColorText: theme.darkColorText,
    darkColorTextMuted: theme.darkColorTextMuted,
    darkColorPrimary: theme.darkColorPrimary,
    darkColorBorder: theme.darkColorBorder,
  };
}
