import { existsSync, readdirSync, statSync } from "node:fs";
import path from "node:path";
import type { ResolvedCodePlayOptions } from "./config";

export const MARKDOWN_RE = /\.(?:md|markdown|mdx)(?:$|\?)/i;

export function cleanMarkdownPath(id: string, root: string): string | undefined {
  const file = id.split("?")[0];
  if (!file || file.startsWith("\0")) {
    return undefined;
  }
  return path.isAbsolute(file) ? file : path.resolve(root, file);
}

export function guessHtmlPath(file: string, srcDir: string, outDir: string): string | undefined {
  const relative = path.relative(srcDir, file).replace(/\.(?:md|markdown|mdx)$/i, "");
  const candidates = [
    path.join(outDir, `${relative}.html`),
    path.join(outDir, relative, "index.html"),
  ];
  return candidates.find((candidate) => existsSync(candidate));
}

export function normalizeBase(base: string): string {
  if (!base || base === "/") {
    return "/";
  }
  return base.endsWith("/") ? base : `${base}/`;
}

export function sourceRoot(root: string, resolved: ResolvedCodePlayOptions): string {
  return path.resolve(root, resolved.srcDir ?? "docs");
}

export function urlToMarkdown(
  urlPath: string,
  root: string,
  srcDir: string,
  base: string,
): string | undefined {
  let relative = urlPath;
  if (base !== "/" && relative.startsWith(base)) {
    relative = relative.slice(base.length);
  }
  relative = relative.replace(/^\//, "").replace(/\.html$/, "");
  if (!relative || relative.includes("..")) {
    return undefined;
  }
  const candidates = [
    path.resolve(root, srcDir, `${relative}.md`),
    path.resolve(root, srcDir, relative, "index.md"),
  ];
  return candidates.find((candidate) => existsSync(candidate));
}

export function walkFiles(dir: string): string[] {
  const entries = readdirSync(dir);
  const files: string[] = [];
  for (const entry of entries) {
    const full = path.join(dir, entry);
    const stat = statSync(full);
    if (stat.isDirectory()) {
      files.push(...walkFiles(full));
    } else {
      files.push(full);
    }
  }
  return files;
}
