import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";

export const CLIENT_FILE_CANDIDATES = [
  "browser.mjs",
  "browser.js",
  "hydrate.mjs",
  "hydrate.js",
] as const;

export function resolveClientFile(baseUrl = import.meta.url): string | undefined {
  return CLIENT_FILE_CANDIDATES.map((name) => fileURLToPath(new URL(`./${name}`, baseUrl))).find(
    (candidate) => existsSync(candidate),
  );
}

export function resolveHydrateSpecifier(baseUrl = import.meta.url): string {
  return resolveClientFile(baseUrl) ?? fileURLToPath(new URL("./hydrate.ts", baseUrl));
}

export function isStandaloneBrowserClient(source: string): boolean {
  return (
    !/\bfrom\s+["']\.\.?\/[^"']+["']/.test(source) &&
    !/\bimport\s*\(\s*["']\.\.?\/[^"']+["']/.test(source)
  );
}

export function isAutoHydratingClient(source: string): boolean {
  return source.includes("bootCodePlay") && source.includes("DOMContentLoaded");
}

export function assertBrowserClientSource(source: string): string {
  if (!isAutoHydratingClient(source) || !isStandaloneBrowserClient(source)) {
    throw new Error(
      "@ox-content/code-play must emit a standalone ox-code-play.js that calls bootCodePlay(). Run the package build.",
    );
  }
  return source;
}
