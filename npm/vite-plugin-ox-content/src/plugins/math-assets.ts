/**
 * Serve and copy KaTeX CSS/fonts only when the optional `katex` package exists.
 */

import { createReadStream } from "node:fs";
import { copyFile, cp, mkdir, stat } from "node:fs/promises";
import { extname, join, relative, resolve, sep } from "node:path";
import type { Plugin } from "vite";
import { KATEX_ASSET_DIR, resolveKatexDist } from "./math";

/** Font formats KaTeX ships, newest first. */
const FONT_FORMATS = { woff2: [".woff2"], all: [".woff2", ".woff", ".ttf"] } as const;

/** Which of KaTeX's font formats to emit. */
export type KatexFontFormats = keyof typeof FONT_FORMATS;

/**
 * Copies `katex.min.css` and the requested font formats into the SSG output.
 * Returns an empty list when KaTeX is not installed.
 *
 * The `.ttf` and `.woff` sets are three quarters of the font bytes and no
 * browser that can run the rest of the site needs them: `@font-face` lists
 * `woff2` first and stops at the first format it supports. `formats: "all"`
 * brings them back.
 */
export async function copyKatexAssets(
  outDir: string,
  formats: KatexFontFormats = "woff2",
): Promise<string[]> {
  const dist = resolveKatexDist();
  if (!dist) {
    return [];
  }

  const dest = join(outDir, KATEX_ASSET_DIR);
  await mkdir(join(dest, "fonts"), { recursive: true });
  const cssDest = join(dest, "katex.min.css");
  await copyFile(join(dist, "katex.min.css"), cssDest);

  const wanted: readonly string[] = FONT_FORMATS[formats] ?? FONT_FORMATS.woff2;
  await cp(join(dist, "fonts"), join(dest, "fonts"), {
    recursive: true,
    filter: (source) => {
      const extension = extname(source);
      return extension === "" || wanted.includes(extension);
    },
  });
  return [cssDest];
}

/** Dev-server middleware that serves `/__ox_katex__/*` from `katex/dist`. */
export function createKatexAssetsPlugin(): Plugin {
  return {
    name: "ox-content:katex-assets",
    configureServer(server) {
      const dist = resolveKatexDist();
      if (!dist) {
        return;
      }

      server.middlewares.use((req, res, next) => {
        const url = req.url ?? "";
        const marker = `/${KATEX_ASSET_DIR}/`;
        const index = url.indexOf(marker);
        if (index === -1) {
          next();
          return;
        }

        const rel = decodeURIComponent(url.slice(index + marker.length).split("?")[0] ?? "");
        const file = safeKatexFile(dist, rel);
        if (!file) {
          res.statusCode = 404;
          res.end();
          return;
        }

        stat(file)
          .then((info) => {
            if (!info.isFile()) {
              res.statusCode = 404;
              res.end();
              return;
            }
            res.setHeader("Content-Type", katexContentType(file));
            createReadStream(file).pipe(res);
          })
          .catch(() => {
            res.statusCode = 404;
            res.end();
          });
      });
    },
  };
}

function safeKatexFile(dist: string, rel: string): string | null {
  if (!rel || rel.includes("\0") || rel.split(/[\\/]/).includes("..")) {
    return null;
  }
  const full = resolve(dist, rel);
  const root = resolve(dist) + sep;
  if (full !== resolve(dist) && !full.startsWith(root)) {
    return null;
  }
  const inside = relative(dist, full);
  if (inside.startsWith("..") || inside.includes(`..${sep}`)) {
    return null;
  }
  return full;
}

function katexContentType(file: string): string {
  const ext = extname(file);
  if (ext === ".css") {
    return "text/css; charset=utf-8";
  }
  if (ext === ".woff2") {
    return "font/woff2";
  }
  if (ext === ".woff") {
    return "font/woff";
  }
  if (ext === ".ttf") {
    return "font/ttf";
  }
  return "application/octet-stream";
}
