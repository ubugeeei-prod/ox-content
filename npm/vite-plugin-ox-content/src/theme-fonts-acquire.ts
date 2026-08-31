/**
 * Resolve self-hosted faces from a local file / `@fontsource` directory or
 * Google Fonts. Downloads are cached; tests inject `fetch` so CI never hits
 * the network.
 */

import { createHash } from "node:crypto";
import { existsSync } from "node:fs";
import { mkdir, readdir, readFile, stat, writeFile } from "node:fs/promises";
import { isAbsolute, join, resolve } from "node:path";
import type { PlannedSelfHostFace, WriteThemeFontsOptions } from "./theme-fonts";

export type FontFetch = (input: string, init?: RequestInit) => Promise<Response>;

const GOOGLE_CSS = "https://fonts.googleapis.com/css2";
const GOOGLE_UA =
  "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";
const ALLOWED_HOSTS = new Set(["fonts.googleapis.com", "fonts.gstatic.com"]);

export interface AcquiredSelfHostFace extends PlannedSelfHostFace {
  bytes: Uint8Array;
}

export function fontMime(fileName: string): string {
  if (fileName.endsWith(".woff")) {
    return "font/woff";
  }
  if (fileName.endsWith(".ttf")) {
    return "font/ttf";
  }
  if (fileName.endsWith(".otf")) {
    return "font/otf";
  }
  return "font/woff2";
}

export function renderFontFaceCss(
  faces: AcquiredSelfHostFace[],
  options: { urlPrefix?: string } = {},
): string {
  return faces
    .map((face) => {
      const range = face.unicodeRange ? `\n  unicode-range: ${face.unicodeRange};` : "";
      const fileName = face.fileName;
      const format = fileName.endsWith(".woff")
        ? "woff"
        : fileName.endsWith(".ttf")
          ? "truetype"
          : fileName.endsWith(".otf")
            ? "opentype"
            : "woff2";
      const family = /^[a-zA-Z_-][\w-]*$/.test(face.family)
        ? face.family
        : `"${face.family.replace(/\\/g, "\\\\").replace(/"/g, '\\"')}"`;
      return `@font-face {
  font-family: ${family};
  font-style: ${face.style};
  font-weight: ${face.weight};
  font-display: ${face.display};
  src: url(${options.urlPrefix ?? "./"}${fileName}) format("${format}");${range}
}`;
    })
    .join("\n\n");
}

export function resolveFontCacheDir(root: string, cacheDir?: string): string {
  return cacheDir ?? join(root, "node_modules", ".cache", "ox-content", "fonts");
}

export async function acquireSelfHostedFaces(
  faces: PlannedSelfHostFace[],
  options: WriteThemeFontsOptions,
): Promise<AcquiredSelfHostFace[]> {
  const cacheDir = resolveFontCacheDir(options.root, options.cacheDir);
  const acquired: AcquiredSelfHostFace[] = [];
  for (const face of faces) {
    acquired.push(
      face.provider === "local"
        ? await acquireLocalFace(face, options.root)
        : await acquireGoogleFace(face, cacheDir, options.fetch ?? fetch),
    );
  }
  return acquired;
}

async function acquireLocalFace(
  face: PlannedSelfHostFace,
  root: string,
): Promise<AcquiredSelfHostFace> {
  if (!face.path) {
    throw new Error(`Theme font "${face.family}" uses provider "local" but has no path.`);
  }
  if (face.path.includes("\0")) {
    throw new Error(`Theme font "${face.family}" path must not contain NUL.`);
  }
  const resolved = resolveLocalPath(root, face.path);
  const info = await stat(resolved).catch(() => undefined);
  if (!info) {
    throw new Error(`Theme font "${face.family}" was not found at ${resolved}.`);
  }
  const file = info.isDirectory() ? await findDirectoryFont(resolved, face) : resolved;
  return { ...face, bytes: await readFile(file) };
}

function resolveLocalPath(root: string, spec: string): string {
  if (isAbsolute(spec)) {
    return spec;
  }
  if (spec.startsWith("@") || !spec.startsWith(".")) {
    return resolve(root, "node_modules", spec);
  }
  return resolve(root, spec);
}

async function findDirectoryFont(dir: string, face: PlannedSelfHostFace): Promise<string> {
  const filesDir = existsSync(join(dir, "files")) ? join(dir, "files") : dir;
  const names = (await readdir(filesDir)).filter((name) => /\.(woff2|woff|ttf|otf)$/i.test(name));
  const weight = String(face.weight);
  const wantItalic = face.style === "italic";
  const match = names.find((name) => {
    const lower = name.toLowerCase();
    const hasWeight = lower.includes(weight);
    const italic = lower.includes("italic");
    const subset = face.subset === "all" || lower.includes(face.subset.toLowerCase());
    return hasWeight && subset && italic === wantItalic;
  });
  const fallback = names[0];
  const chosen = match ?? (names.length === 1 ? fallback : undefined);
  if (!chosen) {
    throw new Error(
      `Theme font "${face.family}" has no ${face.weight} ${face.style} ${face.subset} file in ${filesDir}.`,
    );
  }
  return join(filesDir, chosen);
}

async function acquireGoogleFace(
  face: PlannedSelfHostFace,
  cacheDir: string,
  fetchFn: FontFetch,
): Promise<AcquiredSelfHostFace> {
  const css = await cachedText(googleCssUrl(face), cacheDir, fetchFn, ".css");
  const parsed = parseGoogleCss(css).find(
    (entry) =>
      entry.weight === face.weight &&
      entry.style === face.style &&
      (entry.subset === face.subset || !entry.subset),
  );
  if (!parsed) {
    throw new Error(
      `Google Fonts CSS for "${face.family}" has no ${face.weight} ${face.style} ${face.subset} face.`,
    );
  }
  const bytes = await cachedBytes(parsed.url, cacheDir, fetchFn, ".woff2");
  return { ...face, bytes, unicodeRange: face.unicodeRange ?? parsed.unicodeRange };
}

export function googleCssUrl(face: PlannedSelfHostFace): string {
  const italic = face.style === "italic";
  const axis = italic ? "ital,wght" : "wght";
  const spec = italic ? `1,${face.weight}` : `${face.weight}`;
  const family = `${face.family.replace(/ /g, "+")}:${axis}@${spec}`;
  return `${GOOGLE_CSS}?family=${family}&display=${encodeURIComponent(face.display)}`;
}

interface ParsedGoogleFace {
  subset: string;
  weight: number;
  style: "normal" | "italic";
  url: string;
  unicodeRange?: string;
}

export function parseGoogleCss(css: string): ParsedGoogleFace[] {
  const faces: ParsedGoogleFace[] = [];
  const blocks = css.matchAll(/\/\*\s*([a-z0-9-]+)\s*\*\/\s*@font-face\s*\{([^}]+)\}/gi);
  for (const match of blocks) {
    const parsed = parseGoogleBlock(match[2] ?? "", match[1]?.toLowerCase() ?? "");
    if (parsed) {
      faces.push(parsed);
    }
  }
  if (faces.length === 0) {
    for (const match of css.matchAll(/@font-face\s*\{([^}]+)\}/gi)) {
      const parsed = parseGoogleBlock(match[1] ?? "", "");
      if (parsed) {
        faces.push(parsed);
      }
    }
  }
  return faces;
}

function parseGoogleBlock(body: string, subset: string): ParsedGoogleFace | undefined {
  const url = body.match(/url\((['"]?)(https?:\/\/[^'")]+)\1\)/)?.[2];
  if (!url || !isAllowedFontUrl(url)) {
    return undefined;
  }
  const weight = Number(body.match(/font-weight:\s*(\d+)/i)?.[1] ?? 400);
  const style = /font-style:\s*italic/i.test(body) ? "italic" : "normal";
  return {
    subset,
    weight,
    style,
    url,
    unicodeRange: body.match(/unicode-range:\s*([^;]+)/i)?.[1]?.trim(),
  };
}

function isAllowedFontUrl(url: string): boolean {
  try {
    const parsed = new URL(url);
    return parsed.protocol === "https:" && ALLOWED_HOSTS.has(parsed.hostname);
  } catch {
    return false;
  }
}

async function cachedText(
  url: string,
  cacheDir: string,
  fetchFn: FontFetch,
  ext: string,
): Promise<string> {
  const bytes = await cachedBytes(url, cacheDir, fetchFn, ext);
  return new TextDecoder().decode(bytes);
}

async function cachedBytes(
  url: string,
  cacheDir: string,
  fetchFn: FontFetch,
  ext: string,
): Promise<Uint8Array> {
  if (!isAllowedFontUrl(url)) {
    throw new Error(`Refusing to download font from ${url}.`);
  }
  await mkdir(cacheDir, { recursive: true });
  const dest = join(
    cacheDir,
    `${createHash("sha256").update(url).digest("hex").slice(0, 16)}${ext}`,
  );
  if (existsSync(dest)) {
    return readFile(dest);
  }
  const response = await fetchFn(url, { headers: { "User-Agent": GOOGLE_UA } });
  if (!response.ok) {
    throw new Error(`Failed to download ${url}: ${response.status}`);
  }
  const bytes = new Uint8Array(await response.arrayBuffer());
  await writeFile(dest, bytes);
  return bytes;
}
