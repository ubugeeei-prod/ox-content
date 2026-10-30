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

export async function loadOgFonts(
  options: ResolvedOgImageOptions,
  root: string,
): Promise<SatoriFont[]> {
  const config = options.renderer === "takumi" ? options.takumi : options.satori;
  const renderer = options.renderer === "takumi" ? "Takumi" : "Satori";
  const explicitFonts = config?.fonts ?? [];
  if (explicitFonts.length > 0) {
    return Promise.all(explicitFonts.map((font) => loadFont(font, root)));
  }

  if (config?.systemFontFallback === false) {
    throw new Error(
      `[ox-content:og-image] The ${renderer} renderer requires at least one font. Set ogImageOptions.${options.renderer}.fonts or enable systemFontFallback.`,
    );
  }

  const fallback = await findFirstExistingSystemFont();
  if (!fallback) {
    throw new Error(
      `[ox-content:og-image] The ${renderer} renderer could not find a system font. Set ogImageOptions.${options.renderer}.fonts to a .ttf, .otf, or .woff file.`,
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
