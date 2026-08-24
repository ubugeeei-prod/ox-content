/**
 * Opt-in static redirects / aliases.
 *
 * HTML bodies follow `ox_content_ssg::generate_redirects`. The Vite plugin
 * writes those files during SSG without adding a NAPI surface.
 */

import * as fs from "node:fs/promises";
import * as path from "node:path";
import type { RedirectsOptions, ResolvedRedirectsOptions } from "./types";

const OPTION_KEYS = new Set(["map", "netlify", "headers", "json", "allowExternal"]);

/** One page that may declare aliases or a single `redirect` source. */
export interface RedirectPageInput {
  dest: string;
  aliases?: unknown;
  redirect?: unknown;
}

/** Inputs for planning redirect files. */
export interface RedirectPlanInput {
  options?: ResolvedRedirectsOptions | null;
  base?: string;
  pages: readonly RedirectPageInput[];
}

/** One planned static HTML redirect. */
export interface RedirectFilePlan {
  from: string;
  to: string;
  relativePath: string;
  html: string;
}

/** Planned redirect files and optional host / JSON bodies. */
export interface RedirectPlan {
  files: RedirectFilePlan[];
  netlify?: string;
  headers?: string;
  json?: string;
}

/** Inputs for writing redirect files next to generated HTML. */
export interface WriteRedirectFilesInput {
  outDir: string;
  base?: string;
  options?: ResolvedRedirectsOptions;
  pages: readonly RedirectPageInput[];
}

/**
 * Resolves `redirects` with defaults.
 *
 * `false` / omitted stays off. `true` or `{}` enables empty defaults.
 * A path map (`{ "/old": "/new" }`) enables the feature with that map.
 * `{ map, netlify, headers, json, allowExternal }` overrides only set fields.
 */
export function resolveRedirectsOptions(
  value: boolean | RedirectsOptions | Record<string, string> | undefined,
): ResolvedRedirectsOptions {
  if (!value) {
    return {
      enabled: false,
      map: {},
      netlify: false,
      headers: false,
      json: false,
      allowExternal: false,
    };
  }
  if (value === true) {
    return {
      enabled: true,
      map: {},
      netlify: false,
      headers: false,
      json: false,
      allowExternal: false,
    };
  }
  if (isOptionsObject(value)) {
    return {
      enabled: true,
      map: { ...value.map },
      netlify: value.netlify ?? false,
      headers: value.headers ?? false,
      json: value.json ?? false,
      allowExternal: value.allowExternal ?? false,
    };
  }
  return {
    enabled: true,
    map: { ...value },
    netlify: false,
    headers: false,
    json: false,
    allowExternal: false,
  };
}

/** Plans redirect HTML files without writing them. */
export function planRedirectFiles(input: RedirectPlanInput): RedirectPlan {
  if (!input.options?.enabled) {
    return { files: [] };
  }

  const occupied = new Set<string>();
  for (const page of input.pages) {
    const dest = normalizePath(page.dest);
    if (dest) {
      occupied.add(dest);
    }
  }

  const files: RedirectFilePlan[] = [];
  const index = new Map<string, number>();

  for (const page of input.pages) {
    const to = normalizeDest(page.dest, input.options.allowExternal);
    if (!to) {
      continue;
    }
    for (const alias of readStringList(page.aliases)) {
      upsert(files, index, occupied, alias, to, input.base);
    }
    if (typeof page.redirect === "string") {
      upsert(files, index, occupied, page.redirect, to, input.base);
    }
  }
  for (const [from, to] of Object.entries(input.options.map)) {
    const dest = normalizeDest(to, input.options.allowExternal);
    if (!dest) {
      continue;
    }
    upsert(files, index, occupied, from, dest, input.base);
  }

  if (files.length === 0) {
    return { files: [] };
  }

  const plan: RedirectPlan = { files };
  if (input.options.netlify) {
    plan.netlify = files.map((file) => `${file.from} ${file.to} 301`).join("\n") + "\n";
  }
  if (input.options.headers) {
    plan.headers = files.map((file) => `${file.from}\n  Location: ${file.to}`).join("\n") + "\n";
  }
  if (input.options.json) {
    plan.json = JSON.stringify(files.map((file) => ({ from: file.from, to: file.to })));
  }
  return plan;
}

/** Writes planned redirect HTML (and optional host files) into `outDir`. */
export async function writeRedirectFiles(
  input: WriteRedirectFilesInput,
): Promise<{ files: string[] }> {
  const plan = planRedirectFiles(input);
  if (plan.files.length === 0 && !plan.netlify && !plan.headers && !plan.json) {
    return { files: [] };
  }

  await fs.mkdir(input.outDir, { recursive: true });
  const files: string[] = [];
  for (const entry of plan.files) {
    const outputPath = path.join(input.outDir, entry.relativePath);
    try {
      await fs.access(outputPath);
      continue;
    } catch {
      await fs.mkdir(path.dirname(outputPath), { recursive: true });
      await fs.writeFile(outputPath, entry.html, "utf8");
      files.push(outputPath);
    }
  }
  for (const [body, name] of [
    [plan.netlify, "_redirects"],
    [plan.headers, "_headers"],
    [plan.json, "redirects.json"],
  ] as const) {
    if (!body) {
      continue;
    }
    const outputPath = path.join(input.outDir, name);
    await fs.writeFile(outputPath, body, "utf8");
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
  return isAllowedDest(value, false);
}

/** Strips a trailing slash except for `/`. Unsafe values become `null`. */
export function normalizePath(value: string): string | null {
  return normalizeDest(value, false);
}

function normalizeDest(value: string, allowExternal: boolean): string | null {
  if (!isAllowedDest(value, allowExternal)) {
    return null;
  }
  const trimmed = value.trim();
  if (isHttpUrl(trimmed)) {
    return trimmed;
  }
  if (trimmed === "/") {
    return "/";
  }
  return trimmed.replace(/\/+$/u, "");
}

function isAllowedDest(value: string, allowExternal: boolean): boolean {
  const trimmed = value.trim();
  if (!trimmed || hasDisallowedDestChars(trimmed)) {
    return false;
  }
  if (isHttpUrl(trimmed)) {
    return allowExternal;
  }
  if (!trimmed.startsWith("/") || trimmed.startsWith("//") || hasUnsafePathSegments(trimmed)) {
    return false;
  }
  const lower = trimmed.toLowerCase();
  return !lower.includes("javascript:") && !lower.includes("data:") && !lower.includes("://");
}

function hasDisallowedDestChars(value: string): boolean {
  for (let index = 0; index < value.length; index++) {
    const code = value.charCodeAt(index);
    if (code <= 0x1f || code === 0x7f || code === 0x3b) {
      return true;
    }
  }
  return false;
}

function hasUnsafePathSegments(value: string): boolean {
  return (
    value.includes("\\") || value.split("/").some((segment) => segment === "." || segment === "..")
  );
}

function isHttpUrl(value: string): boolean {
  const lower = value.toLowerCase();
  return lower.startsWith("https://") || lower.startsWith("http://");
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

function applyBase(dest: string, base: string | undefined): string {
  if (isHttpUrl(dest) || !base || base === "/") {
    return dest;
  }
  const prefix = base.replace(/\/+$/u, "");
  return dest === "/" ? `${prefix}/` : `${prefix}${dest}`;
}

function upsert(
  files: RedirectFilePlan[],
  index: Map<string, number>,
  occupied: Set<string>,
  from: string,
  to: string,
  base: string | undefined,
): void {
  const source = normalizePath(from);
  if (!source || source === to || occupied.has(source)) {
    return;
  }
  const href = applyBase(to, base);
  const html = generateRedirectHtml(href);
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
