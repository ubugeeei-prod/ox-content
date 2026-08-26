/**
 * Opt-in web-font objects for `theme.fonts`, plus SSG self-host emission.
 *
 * NAPI still receives flattened CSS stacks (`JsThemeFonts`). File acquisition
 * and `@font-face` generation stay in TypeScript so other PRs can keep landing
 * NAPI theme-type changes independently.
 */

import { mkdir, writeFile } from "node:fs/promises";
import { join } from "node:path";
import {
  acquireSelfHostedFaces,
  fontMime,
  renderFontFaceCss,
  type FontFetch,
} from "./theme-fonts-acquire";

export const FONT_ASSET_DIR = "__ox_fonts__";
export const FONT_CSS_NAME = "fonts.css";

export type ThemeFontProvider = "google" | "local";
export type ThemeFontStyle = "normal" | "italic";
export type ThemeFontDisplay = "auto" | "block" | "swap" | "fallback" | "optional";

/** UnoCSS-inspired family descriptor. The string stack form remains valid. */
export interface ThemeWebFont {
  /** Family name, e.g. `"Inter"` or `"DM Mono"`. */
  family: string;
  /** Defaults to `"local"` when `path` is set, otherwise `"google"`. */
  provider?: ThemeFontProvider;
  /** File, directory, or `@fontsource/*` package. Required for `local`. */
  path?: string;
  weights?: number[];
  styles?: ThemeFontStyle[];
  subsets?: string[];
  display?: ThemeFontDisplay;
  /** Copy files into the SSG output and emit `@font-face`. */
  selfHost?: boolean;
  /** Extra families after `family` in the emitted CSS stack. */
  fallbacks?: string[];
  /** Preload every self-hosted face, or only these weights. */
  preload?: boolean | number[];
  /** Optional `unicode-range` for local faces. */
  unicodeRange?: string;
}

export type ThemeFontValue = string | ThemeWebFont;

export interface ThemeFontsLike {
  sans?: ThemeFontValue;
  mono?: ThemeFontValue;
  named?: Record<string, ThemeFontValue>;
}

export interface WriteThemeFontsOptions {
  fonts: ThemeFontsLike;
  outDir: string;
  root: string;
  cacheDir?: string;
  fetch?: FontFetch;
}

const NAMED_FONT_PATTERN = /^[a-z][a-z0-9-]*$/;
const GENERIC_FOR = { sans: "sans-serif", mono: "monospace", named: "sans-serif" } as const;

export function isThemeWebFont(value: ThemeFontValue | undefined): value is ThemeWebFont {
  return typeof value === "object" && value !== null && typeof value.family === "string";
}

/** CSS `font-family` identifier; quotes names that are not a single ident. */
export function cssFamilyName(family: string): string {
  const trimmed = family.trim();
  if (
    (trimmed.startsWith('"') && trimmed.endsWith('"')) ||
    (trimmed.startsWith("'") && trimmed.endsWith("'"))
  ) {
    return trimmed;
  }
  if (/^[a-zA-Z_-][\w-]*$/.test(trimmed)) {
    return trimmed;
  }
  return `"${trimmed.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
}

export function flattenThemeFont(
  value: ThemeFontValue | undefined,
  generic: string,
): string | undefined {
  if (value === undefined) {
    return undefined;
  }
  if (typeof value === "string") {
    return value;
  }
  const fallbacks = value.fallbacks?.length ? value.fallbacks.join(", ") : generic;
  return `${cssFamilyName(value.family)}, ${fallbacks}`;
}

/** Flatten object fonts so `JsThemeFonts` stays `{ sans?: string; mono?: string }`. */
export function flattenThemeFonts(
  fonts: ThemeFontsLike,
): { sans?: string; mono?: string } | undefined {
  const sans = flattenThemeFont(fonts.sans, GENERIC_FOR.sans);
  const mono = flattenThemeFont(fonts.mono, GENERIC_FOR.mono);
  if (!sans && !mono) {
    return undefined;
  }
  return { sans, mono };
}

export function namedFontToken(name: string): string {
  if (!NAMED_FONT_PATTERN.test(name)) {
    throw new Error(
      `Invalid theme font name: ${JSON.stringify(name)}. ` +
        `Named fonts are lowercase kebab-case (e.g. "code").`,
    );
  }
  return name;
}

/** Extra `--octc-font-*` variables for `fonts.named`. Roles stay in Rust theme CSS. */
export function namedFontVarsCss(fonts: ThemeFontsLike): string {
  const entries = Object.entries(fonts.named ?? {});
  if (entries.length === 0) {
    return "";
  }
  const lines = entries.map(([name, value]) => {
    const stack = flattenThemeFont(value, GENERIC_FOR.named);
    return `  --octc-font-${namedFontToken(name)}: ${stack};`;
  });
  return `:root {\n${lines.join("\n")}\n}`;
}

export function normalizeBasePath(base: string | undefined): string {
  if (!base || base === "/") {
    return "/";
  }
  return base.endsWith("/") ? base : `${base}/`;
}

export function plannedFontFileName(
  family: string,
  weight: number,
  style: ThemeFontStyle,
  subset: string,
  extension: string,
): string {
  const ext = extension.startsWith(".") ? extension : `.${extension}`;
  return `${slugify(family)}-${weight}-${style}-${slugify(subset)}${ext}`;
}

export function plannedFontExtension(font: ThemeWebFont): string {
  if (
    font.provider === "local" &&
    font.path &&
    /\.\w+$/.test(font.path) &&
    !font.path.endsWith("/")
  ) {
    const match = font.path.match(/(\.\w+)$/);
    return match?.[1] ?? ".woff2";
  }
  return ".woff2";
}

export interface PlannedSelfHostFace {
  family: string;
  weight: number;
  style: ThemeFontStyle;
  subset: string;
  display: ThemeFontDisplay;
  preload: boolean;
  provider: ThemeFontProvider;
  path?: string;
  fileName: string;
  unicodeRange?: string;
}

export function planSelfHostedFaces(fonts: ThemeFontsLike): PlannedSelfHostFace[] {
  const faces: PlannedSelfHostFace[] = [];
  for (const value of themeFontValues(fonts)) {
    if (!isThemeWebFont(value) || !value.selfHost) {
      continue;
    }
    const font = normalizeWebFont(value);
    const extension = plannedFontExtension(font);
    for (const weight of font.weights) {
      for (const style of font.styles) {
        for (const subset of font.subsets) {
          faces.push({
            family: font.family,
            weight,
            style,
            subset,
            display: font.display,
            preload: shouldPreload(font.preload, weight),
            provider: font.provider,
            path: font.path,
            fileName: plannedFontFileName(font.family, weight, style, subset, extension),
            unicodeRange: font.unicodeRange,
          });
        }
      }
    }
  }
  const unique = new Map<string, PlannedSelfHostFace>();
  for (const face of faces) {
    const existing = unique.get(face.fileName);
    if (existing) {
      existing.preload ||= face.preload;
    } else {
      unique.set(face.fileName, face);
    }
  }
  return [...unique.values()];
}

export function themeFontHeadHtml(fonts: ThemeFontsLike, base?: string): string {
  const faces = planSelfHostedFaces(fonts);
  if (faces.length === 0) {
    return "";
  }
  const root = normalizeBasePath(base);
  const tags = [`<link rel="stylesheet" href="${root}${FONT_ASSET_DIR}/${FONT_CSS_NAME}">`];
  for (const face of faces) {
    if (!face.preload) {
      continue;
    }
    tags.push(
      `<link rel="preload" href="${root}${FONT_ASSET_DIR}/${face.fileName}" as="font" type="${fontMime(face.fileName)}" crossorigin>`,
    );
  }
  return tags.join("\n");
}

export function withSelfHostedFontHead<T extends { head?: string }>(
  embed: T,
  fonts: ThemeFontsLike,
  base?: string,
): T | undefined {
  const extra = themeFontHeadHtml(fonts, base);
  const keys = Object.keys(embed);
  if (!extra && keys.length === 0) {
    return undefined;
  }
  if (!extra) {
    return embed;
  }
  return { ...embed, head: embed.head ? `${extra}\n${embed.head}` : extra };
}

/** Copy self-hosted faces into `outDir` and write `@font-face` CSS. */
export async function writeSelfHostedThemeFonts(
  options: WriteThemeFontsOptions,
): Promise<string[]> {
  const faces = planSelfHostedFaces(options.fonts);
  if (faces.length === 0) {
    return [];
  }
  const acquired = await acquireSelfHostedFaces(faces, options);
  const destDir = join(options.outDir, FONT_ASSET_DIR);
  await mkdir(destDir, { recursive: true });
  const written: string[] = [];
  for (const face of acquired) {
    const dest = join(destDir, face.fileName);
    await writeFile(dest, face.bytes);
    written.push(dest);
  }
  const cssPath = join(destDir, FONT_CSS_NAME);
  await writeFile(cssPath, renderFontFaceCss(acquired), "utf8");
  written.push(cssPath);
  return written;
}

function themeFontValues(fonts: ThemeFontsLike): ThemeFontValue[] {
  return [fonts.sans, fonts.mono, ...Object.values(fonts.named ?? {})].filter(
    (value): value is ThemeFontValue => value !== undefined,
  );
}

function normalizeWebFont(
  font: ThemeWebFont,
): Required<
  Pick<ThemeWebFont, "family" | "provider" | "weights" | "styles" | "subsets" | "display">
> &
  ThemeWebFont {
  const provider = font.provider ?? (font.path ? "local" : "google");
  if (provider === "local" && !font.path) {
    throw new Error(`Theme font "${font.family}" uses provider "local" but has no path.`);
  }
  return {
    ...font,
    family: font.family.trim(),
    provider,
    weights: font.weights?.length ? font.weights : [400],
    styles: font.styles?.length ? font.styles : ["normal"],
    subsets: font.subsets?.length ? font.subsets : ["latin"],
    display: font.display ?? "swap",
  };
}

function shouldPreload(preload: ThemeWebFont["preload"], weight: number): boolean {
  if (preload === true) {
    return true;
  }
  return Array.isArray(preload) && preload.includes(weight);
}

function slugify(value: string): string {
  const slug = value
    .trim()
    .toLowerCase()
    .replace(/['"]/g, "")
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-|-$/g, "");
  return slug || "font";
}
