import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { Connect } from "vite";
import { resolveSelfHostedAssetManifest } from "./assets";
import { DEFAULT_THEME_TOKEN_HREF } from "./custom-host-constants";
import type {
  OxContentCustomHostAssetsContext,
  OxContentCustomHostOptions,
  ResolvedThemeTokens,
} from "./custom-host-types";
import { resolveOutputPath, withBase } from "./custom-host-utils";
import {
  renderDocumentAssets,
  type DocumentAssetManifest,
  type RenderDocumentAssetsInput,
} from "./document-assets";
import { renderThemeTokenCss } from "./theme-tokens";
import type { ResolvedOptions } from "./types";

export function createAssetsContext(
  options: ResolvedOptions,
  _outDir: string,
  clientManifest: DocumentAssetManifest | undefined,
  themeTokens: ResolvedThemeTokens | undefined,
): OxContentCustomHostAssetsContext {
  const selfHosted = resolveSelfHostedAssetManifest(options);
  return {
    selfHosted,
    clientManifest,
    themeTokens,
    document(input: RenderDocumentAssetsInput = {}) {
      return renderDocumentAssets({
        base: options.base,
        manifest: clientManifest,
        selfHostedAssets: selfHosted,
        ...input,
      });
    },
  };
}

export async function readClientManifest(
  outDir: string,
): Promise<DocumentAssetManifest | undefined> {
  const manifestPath = path.join(outDir, ".vite", "manifest.json");
  try {
    return JSON.parse(await fs.readFile(manifestPath, "utf8")) as DocumentAssetManifest;
  } catch {
    return undefined;
  }
}

export async function writeThemeTokens(
  outDir: string,
  themeTokens: ResolvedThemeTokens | undefined,
): Promise<void> {
  if (!themeTokens) {
    return;
  }
  const outputPath = resolveOutputPath(outDir, themeTokens.outputPath);
  await fs.mkdir(path.dirname(outputPath), { recursive: true });
  await fs.writeFile(outputPath, themeTokens.css, "utf8");
}

export function themeTokenMiddleware(
  themeTokens: ResolvedThemeTokens | undefined,
): Connect.NextHandleFunction {
  return (req, res, next) => {
    if (!themeTokens || req.url?.split("?")[0] !== themeTokens.href) {
      next();
      return;
    }
    res.statusCode = 200;
    res.setHeader("Content-Type", "text/css; charset=utf-8");
    res.end(themeTokens.css);
  };
}

export function resolveThemeTokens(
  input: OxContentCustomHostOptions["themeTokens"],
  base: string,
): ResolvedThemeTokens | undefined {
  if (!input) {
    return undefined;
  }
  const href = withBase(base, input.href ?? DEFAULT_THEME_TOKEN_HREF);
  return {
    href,
    outputPath: input.href ?? DEFAULT_THEME_TOKEN_HREF,
    css: renderThemeTokenCss(input.theme, { include: input.include }),
  };
}
