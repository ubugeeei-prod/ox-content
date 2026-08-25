import * as fs from "fs/promises";
import * as path from "path";
import type { Font as SatoriFont } from "satori";
import type { OgImageSatoriFont, ResolvedOgImageOptions } from "./types";

interface SystemFontCandidate {
  path: string;
  name?: string;
}

const FALLBACK_FONT_FAMILY = "OxContentSans";

const SYSTEM_FONT_CANDIDATES: SystemFontCandidate[] = [
  { path: "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf" },
  { path: "/usr/share/fonts/truetype/dejavu/DejaVuSansCondensed.ttf" },
  { path: "/usr/share/fonts/truetype/liberation2/LiberationSans-Regular.ttf" },
  { path: "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf" },
  { path: "/System/Library/Fonts/Supplemental/Arial.ttf" },
  { path: "/System/Library/Fonts/Supplemental/Helvetica.ttf" },
  { path: "/Library/Fonts/Arial.ttf" },
  { path: "C:\\Windows\\Fonts\\arial.ttf" },
  { path: "C:\\Windows\\Fonts\\segoeui.ttf" },
];

/**
 * Render an HTML string to PNG using Satori and resvg.
 */
export async function renderHtmlToPngWithSatori(
  source: string,
  options: ResolvedOgImageOptions,
  root: string,
): Promise<Buffer> {
  const fonts = await loadSatoriFonts(options, root);
  const [{ Resvg }, { default: satori }, { html }] = await Promise.all([
    import("@resvg/resvg-js"),
    import("satori"),
    import("satori-html"),
  ]);
  const vnode = html(source) as unknown as Parameters<typeof satori>[0];
  const svg = await satori(vnode, {
    width: options.width,
    height: options.height,
    fonts,
  });

  return new Resvg(svg, {
    fitTo: { mode: "original" },
    font: { loadSystemFonts: false },
  })
    .render()
    .asPng();
}

async function loadSatoriFonts(
  options: ResolvedOgImageOptions,
  root: string,
): Promise<SatoriFont[]> {
  const explicitFonts = options.satori.fonts;
  if (explicitFonts.length > 0) {
    return Promise.all(explicitFonts.map((font) => loadFont(font, root)));
  }

  if (!options.satori.systemFontFallback) {
    throw new Error(
      "[ox-content:og-image] The Satori renderer requires at least one font. " +
        "Set ogImageOptions.satori.fonts or enable satori.systemFontFallback.",
    );
  }

  const fallback = await findFirstExistingSystemFont();
  if (!fallback) {
    throw new Error(
      "[ox-content:og-image] The Satori renderer could not find a system font. " +
        "Set ogImageOptions.satori.fonts to a .ttf, .otf, or .woff file.",
    );
  }

  return [
    await loadFont(
      {
        path: fallback.path,
        name: fallback.name ?? FALLBACK_FONT_FAMILY,
      },
      root,
    ),
  ];
}

async function loadFont(font: OgImageSatoriFont, root: string): Promise<SatoriFont> {
  const fontPath = path.isAbsolute(font.path) ? font.path : path.resolve(root, font.path);
  const data = await fs.readFile(fontPath);
  return {
    data,
    name: font.name ?? fontNameFromPath(fontPath),
    weight: font.weight ?? 400,
    style: font.style ?? "normal",
  };
}

async function findFirstExistingSystemFont(): Promise<SystemFontCandidate | null> {
  for (const candidate of SYSTEM_FONT_CANDIDATES) {
    try {
      await fs.access(candidate.path);
      return { ...candidate, name: candidate.name ?? FALLBACK_FONT_FAMILY };
    } catch {
      // Keep looking through platform-specific fallbacks.
    }
  }
  return null;
}

function fontNameFromPath(fontPath: string): string {
  const basename = path.basename(fontPath).replace(/\.(otf|ttf|woff)$/i, "");
  return basename || FALLBACK_FONT_FAMILY;
}
