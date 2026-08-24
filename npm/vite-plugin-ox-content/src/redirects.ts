/**
 * Opt-in static redirects / aliases.
 *
 * HTML bodies follow `ox_content_ssg::generate_redirects`. The Vite plugin
 * writes those files during SSG without adding a NAPI surface.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { RedirectsOptions, ResolvedRedirectsOptions } from "./types";

const OPTION_KEYS = new Set(["enabled", "map", "netlify", "writeNetlify"]);

/** One page that may declare aliases or a single `redirect` source. */
export interface RedirectPageInput {
  dest: string;
  aliases?: unknown;
  redirect?: unknown;
}

/** Inputs for planning redirect files. */
export interface RedirectPlanInput {
  options?: ResolvedRedirectsOptions | null;
  pages: readonly RedirectPageInput[];
}

/** One planned static HTML redirect. */
export interface RedirectFilePlan {
  from: string;
  to: string;
  relativePath: string;
  html: string;
}

/** Planned redirect files and an optional host `_redirects` body. */
export interface RedirectPlan {
  files: RedirectFilePlan[];
  netlify?: string;
}

/** Inputs for writing redirect files next to generated HTML. */
export interface WriteRedirectFilesInput {
  outDir: string;
  options?: ResolvedRedirectsOptions;
  pages: readonly RedirectPageInput[];
}

/**
 * Resolves `redirects` with defaults.
 *
 * `false` / omitted stays off. `true` or `{}` enables empty defaults.
 * A path map (`{ "/old": "/new" }`) enables the feature with that map.
 * `{ enabled, map, netlify | writeNetlify }` overrides only set fields.
 */
export function resolveRedirectsOptions(
  value: boolean | RedirectsOptions | Record<string, string> | undefined,
): ResolvedRedirectsOptions {
  if (!value) {
    return { enabled: false, map: {}, netlify: false };
  }
  if (value === true) {
    return { enabled: true, map: {}, netlify: false };
  }
  if (isOptionsObject(value)) {
    return {
      enabled: value.enabled ?? true,
      map: { ...value.map },
      netlify: value.netlify ?? value.writeNetlify ?? false,
    };
  }
  return { enabled: true, map: { ...value }, netlify: false };
}

/** Plans redirect HTML files without writing them. */
export function planRedirectFiles(input: RedirectPlanInput): RedirectPlan {
  if (!input.options?.enabled) {
    return { files: [] };
  }

  const files: RedirectFilePlan[] = [];
  const index = new Map<string, number>();

  for (const page of input.pages) {
    const to = normalizePath(page.dest);
    if (!to) {
      continue;
    }
    for (const alias of readStringList(page.aliases)) {
      upsert(files, index, alias, to);
    }
    if (typeof page.redirect === "string") {
      upsert(files, index, page.redirect, to);
    }
  }
  for (const [from, to] of Object.entries(input.options.map)) {
    const dest = normalizePath(to);
    if (!dest) {
      continue;
    }
    upsert(files, index, from, dest);
  }

  const plan: RedirectPlan = { files };
  if (input.options.netlify && files.length > 0) {
    plan.netlify = files.map((file) => `${file.from} ${file.to} 301`).join("\n") + "\n";
  }
  return plan;
}

/** Writes planned redirect HTML (and optional `_redirects`) into `outDir`. */
export async function writeRedirectFiles(
  input: WriteRedirectFilesInput,
): Promise<{ files: string[] }> {
  const plan = planRedirectFiles(input);
  if (plan.files.length === 0 && !plan.netlify) {
    return { files: [] };
  }

  await fs.mkdir(input.outDir, { recursive: true });
  const files: string[] = [];
  for (const entry of plan.files) {
    const outputPath = path.join(input.outDir, entry.relativePath);
    await fs.mkdir(path.dirname(outputPath), { recursive: true });
    await fs.writeFile(outputPath, entry.html, "utf8");
    files.push(outputPath);
  }
  if (plan.netlify) {
    const outputPath = path.join(input.outDir, "_redirects");
    await fs.writeFile(outputPath, plan.netlify, "utf8");
    files.push(outputPath);
  }
  return { files };
}

/** Static HTML redirect body. `dest` is escaped. */
export function generateRedirectHtml(dest: string): string {
  const escaped = escapeHtml(dest);
  return `\
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta http-equiv="refresh" content="0;url=${escaped}">
<link rel="canonical" href="${escaped}">
<title>Redirecting</title>
</head>
<body>
<p>Redirecting to <a href="${escaped}">${escaped}</a>.</p>
</body>
</html>
`;
}

/** Same-origin path: leading `/`, not `//`, and no scheme. */
export function isSafeDest(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed.startsWith("/") || trimmed.startsWith("//")) {
    return false;
  }
  const lower = trimmed.toLowerCase();
  return !lower.includes("javascript:") && !lower.includes("data:") && !lower.includes("://");
}

/** Strips a trailing slash except for `/`. Unsafe values become `null`. */
export function normalizePath(value: string): string | null {
  if (!isSafeDest(value)) {
    return null;
  }
  const trimmed = value.trim();
  if (trimmed === "/") {
    return "/";
  }
  return trimmed.replace(/\/+$/u, "");
}

function isOptionsObject(
  value: RedirectsOptions | Record<string, string>,
): value is RedirectsOptions {
  return Object.keys(value).some((key) => OPTION_KEYS.has(key));
}

function readStringList(value: unknown): string[] {
  if (typeof value === "string") {
    return [value];
  }
  if (!Array.isArray(value)) {
    return [];
  }
  return value.filter((entry): entry is string => typeof entry === "string");
}

function upsert(
  files: RedirectFilePlan[],
  index: Map<string, number>,
  from: string,
  to: string,
): void {
  const source = normalizePath(from);
  if (!source || source === to) {
    return;
  }
  const html = generateRedirectHtml(to);
  const relativePath = source === "/" ? "index.html" : `${source.slice(1)}/index.html`;
  const slot = index.get(source);
  if (slot !== undefined) {
    files[slot] = { from: source, to, relativePath, html };
    return;
  }
  index.set(source, files.length);
  files.push({ from: source, to, relativePath, html });
}

function escapeHtml(value: string): string {
  return value.replace(/[&<>"']/g, (ch) => {
    switch (ch) {
      case "&":
        return "&amp;";
      case "<":
        return "&lt;";
      case ">":
        return "&gt;";
      case '"':
        return "&quot;";
      default:
        return "&#39;";
    }
  });
}
