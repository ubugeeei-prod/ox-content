import * as fs from "node:fs/promises";
import type { IncomingMessage } from "node:http";
import * as path from "node:path";
import type { Plugin, ResolvedConfig } from "vite";
import {
  ICON_ASSET_DIR,
  iconStylesheetHref,
  renderSelfHostedIconsCss,
  writeSelfHostedIcons,
} from "./icons";
import {
  FONT_ASSET_DIR,
  normalizeBasePath,
  planSelfHostedFaces,
  themeFontStylesheetHref,
  writeSelfHostedThemeFonts,
  type ThemeFontsLike,
} from "./theme-fonts";
import { fontMime, renderFontFaceCss, type AcquiredSelfHostFace } from "./theme-fonts-acquire";
import type { ResolvedOptions } from "./types";

const ASSETS_CSS_ID = "\0virtual:ox-content/assets.css";
const ASSET_MANIFEST_ID = "\0virtual:ox-content/asset-manifest";

export interface OxContentAssetPreload {
  href: string;
  as: "font";
  type: string;
  crossorigin: true;
}

export interface OxContentAssetManifest {
  stylesheets: string[];
  preloads: OxContentAssetPreload[];
  headTags: string;
}

export type SelfHostedAssetOptions = Pick<ResolvedOptions, "base" | "srcDir" | "icons" | "ssg">;

export interface WriteSelfHostedAssetsInput {
  options: SelfHostedAssetOptions;
  outDir: string;
  root?: string;
  cacheDir?: string;
}

export interface WriteSelfHostedAssetsResult {
  files: string[];
  errors: string[];
}

export interface RenderSelfHostedAssetsCssResult {
  css: string;
  errors: string[];
}

export function resolveSelfHostedAssetManifest(
  options: SelfHostedAssetOptions,
): OxContentAssetManifest {
  const base = options.base;
  const stylesheets: string[] = [];
  const preloads: OxContentAssetPreload[] = [];

  if (options.icons?.enabled) {
    stylesheets.push(iconStylesheetHref(base));
  }

  const fontFaces = planSelfHostedFaces(assetFonts(options));
  if (fontFaces.length > 0) {
    stylesheets.push(themeFontStylesheetHref(base));
    const root = normalizeBasePath(base);
    for (const face of fontFaces) {
      if (face.preload) {
        preloads.push({
          href: `${root}${FONT_ASSET_DIR}/${face.fileName}`,
          as: "font",
          type: fontMime(face.fileName),
          crossorigin: true,
        });
      }
    }
  }

  const headTags = [
    ...stylesheets.map((href) => `<link rel="stylesheet" href="${href}">`),
    ...preloads.map(
      (item) =>
        `<link rel="preload" href="${item.href}" as="${item.as}" type="${item.type}" crossorigin>`,
    ),
  ].join("\n");

  return { stylesheets, preloads, headTags };
}

export async function writeSelfHostedAssets(
  input: WriteSelfHostedAssetsInput,
): Promise<WriteSelfHostedAssetsResult> {
  const root = input.root ?? process.cwd();
  const files: string[] = [];
  const errors: string[] = [];
  const fonts = assetFonts(input.options);

  if (planSelfHostedFaces(fonts).length > 0) {
    files.push(
      ...(await writeSelfHostedThemeFonts({
        fonts,
        outDir: input.outDir,
        root,
        cacheDir: input.cacheDir,
      })),
    );
  }

  if (input.options.icons?.enabled) {
    const icons = await writeSelfHostedIcons({
      options: input.options.icons,
      outDir: input.outDir,
      root,
      srcDir: path.resolve(root, input.options.srcDir),
      socialLinks: input.options.ssg.theme?.socialLinks,
    });
    files.push(...icons.files);
    errors.push(...icons.errors);
  }

  return { files, errors };
}

export async function renderSelfHostedAssetsCss(
  options: SelfHostedAssetOptions,
  root = process.cwd(),
): Promise<RenderSelfHostedAssetsCssResult> {
  const css: string[] = [];
  const errors: string[] = [];
  const fonts = assetFonts(options);
  const faces = planSelfHostedFaces(fonts);

  if (faces.length > 0) {
    css.push(
      renderFontFaceCss(
        faces.map((face) => ({ ...face, bytes: new Uint8Array() }) satisfies AcquiredSelfHostFace),
        { urlPrefix: `${normalizeBasePath(options.base)}${FONT_ASSET_DIR}/` },
      ),
    );
  }

  if (options.icons?.enabled) {
    const icons = await renderSelfHostedIconsCss({
      options: options.icons,
      root,
      srcDir: path.resolve(root, options.srcDir),
      socialLinks: options.ssg.theme?.socialLinks,
    });
    css.push(icons.css);
    errors.push(...icons.errors);
  }

  return {
    css: css.length > 0 ? `${css.join("\n\n")}\n` : "/* ox-content: no self-hosted assets */\n",
    errors,
  };
}

export function createAssetsPlugin(
  options: ResolvedOptions,
  getRoot: () => string,
  getConfig: () => ResolvedConfig | undefined,
): Plugin {
  let devWrite: Promise<WriteSelfHostedAssetsResult> | undefined;

  return {
    name: "ox-content:assets",

    resolveId(id) {
      if (id === "virtual:ox-content/assets.css") {
        return ASSETS_CSS_ID;
      }
      if (id === "virtual:ox-content/asset-manifest") {
        return ASSET_MANIFEST_ID;
      }
      return null;
    },

    async load(id) {
      if (id === ASSETS_CSS_ID) {
        const result = await renderSelfHostedAssetsCss(options, getRoot());
        for (const error of result.errors) {
          this.warn(error);
        }
        return result.css;
      }
      if (id === ASSET_MANIFEST_ID) {
        return generateAssetManifestModule(resolveSelfHostedAssetManifest(options));
      }
      return null;
    },

    configureServer(server) {
      const root = getRoot();
      const outDir = path.join(resolveConfigDir(root, server.config.cacheDir), "ox-content-assets");
      const cacheDir = path.join(outDir, ".font-cache");
      const ensureWritten = () => {
        devWrite ??= writeSelfHostedAssets({ options, outDir, root, cacheDir });
        return devWrite;
      };

      server.watcher.on("all", () => {
        devWrite = undefined;
      });
      server.middlewares.use(async (req, res, next) => {
        const rel = assetRequestPath(req, options.base);
        if (!rel || (req.method !== "GET" && req.method !== "HEAD")) {
          next();
          return;
        }
        try {
          await ensureWritten();
          const file = resolveAssetFile(outDir, rel);
          if (!file) {
            next();
            return;
          }
          const bytes = await fs.readFile(file).catch(() => undefined);
          if (!bytes) {
            next();
            return;
          }
          res.statusCode = 200;
          res.setHeader("Content-Type", contentType(rel));
          if (req.method === "HEAD") {
            res.end();
          } else {
            res.end(bytes);
          }
        } catch (error) {
          res.statusCode = 500;
          res.end(error instanceof Error ? error.message : String(error));
        }
      });
    },

    async closeBundle() {
      const config = getConfig();
      const root = getRoot();
      const outDir = resolveBuildOutDir(config, options, root);
      const result = await writeSelfHostedAssets({ options, outDir, root });
      for (const error of result.errors) {
        this.warn(error);
      }
    },
  };
}

function assetFonts(options: SelfHostedAssetOptions): ThemeFontsLike {
  return options.ssg.theme?.fonts ?? {};
}

function generateAssetManifestModule(manifest: OxContentAssetManifest): string {
  const json = JSON.stringify(manifest);
  return [
    `const manifest = ${json};`,
    "export const stylesheets = manifest.stylesheets;",
    "export const preloads = manifest.preloads;",
    "export const headTags = manifest.headTags;",
    "export default manifest;",
  ].join("\n");
}

function assetRequestPath(req: IncomingMessage, base: string): string | undefined {
  const rawUrl = req.url;
  if (!rawUrl) {
    return undefined;
  }
  const pathname = new URL(rawUrl, "http://ox-content.local").pathname;
  const normalizedBase = normalizeBasePath(base);
  const rel =
    normalizedBase === "/"
      ? pathname.slice(1)
      : pathname.startsWith(normalizedBase)
        ? pathname.slice(normalizedBase.length)
        : "";
  if (!rel || !isSafeAssetPath(rel)) {
    return undefined;
  }
  return rel;
}

function isSafeAssetPath(rel: string): boolean {
  const first = rel.split("/")[0];
  if (first !== ICON_ASSET_DIR && first !== FONT_ASSET_DIR) {
    return false;
  }
  return (
    !rel.includes("\0") &&
    !rel.includes("\\") &&
    path.posix.normalize(rel) === rel &&
    !path.posix.isAbsolute(rel)
  );
}

function resolveAssetFile(outDir: string, rel: string): string | undefined {
  const file = path.resolve(outDir, ...rel.split("/"));
  const root = path.resolve(outDir);
  const prefix = `${root}${path.sep}`;
  return file.startsWith(prefix) ? file : undefined;
}

function contentType(rel: string): string {
  if (rel.endsWith(".css")) {
    return "text/css; charset=utf-8";
  }
  if (rel.startsWith(`${FONT_ASSET_DIR}/`)) {
    return fontMime(rel);
  }
  return "application/octet-stream";
}

function resolveBuildOutDir(
  config: ResolvedConfig | undefined,
  options: ResolvedOptions,
  root: string,
): string {
  return config?.build.outDir
    ? resolveConfigDir(root, config.build.outDir)
    : path.resolve(root, options.outDir);
}

function resolveConfigDir(root: string, dir: string): string {
  return path.isAbsolute(dir) ? dir : path.resolve(root, dir);
}
