/**
 * Opt-in self-hosted Iconify CSS for used and safelisted icons.
 *
 * Collection lookup stays on disk (`@iconify-json/*` / `@iconify/json`).
 * Theme embed injection composes with self-hosted font `<link>` tags.
 */

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { glob } from "glob";
import type { IconsOptions, ResolvedIconsOptions } from "./types";
import { normalizeBasePath } from "./theme-fonts";
import {
  iconClassName,
  isMulticolorIcon,
  loadIconCollection,
  lookupIcon,
  renderIconsCss,
  type ResolvedIcon,
} from "./icons-css";

export const ICON_ASSET_DIR = "__ox_icons__";
export const ICON_CSS_NAME = "icons.css";

const URL_SCHEMES = new Set([
  "http",
  "https",
  "data",
  "mailto",
  "file",
  "javascript",
  "vscode",
  "tel",
  "blob",
]);

const COLON_ICON = /(?<![A-Za-z0-9_-])([a-z][a-z0-9-]*):([a-z0-9][a-z0-9-]*)/gi;
const CLASS_ICON = /icon-\[([a-z][a-z0-9-]*)--([a-z0-9][a-z0-9-]*)\]/gi;
const ICON_FIELD = /(?:^|[\s,{])icon\s*:\s*["']([^"']+)["']/g;

export function resolveIconsOptions(
  value: boolean | IconsOptions | undefined,
): ResolvedIconsOptions {
  if (!value) {
    return { enabled: false, mode: "css-mask", syntax: "unocss", include: [], safelist: [] };
  }
  if (value === true) {
    return { enabled: true, mode: "css-mask", syntax: "unocss", include: [], safelist: [] };
  }
  return {
    enabled: true,
    mode: value.mode ?? "css-mask",
    syntax: value.syntax ?? "unocss",
    include: value.include ?? [],
    safelist: value.safelist ?? [],
  };
}

export interface ParsedIconName {
  prefix: string;
  name: string;
}

export function parseIconName(value: string): ParsedIconName | undefined {
  const trimmed = value.trim();
  const classMatch = /^icon-\[(.+)\]$/.exec(trimmed);
  if (classMatch?.[1]) {
    const inner = classMatch[1];
    const sep = inner.indexOf("--");
    if (sep <= 0) {
      return undefined;
    }
    return tokenPair(inner.slice(0, sep), inner.slice(sep + 2));
  }
  const sep = trimmed.indexOf(":");
  if (sep <= 0) {
    return undefined;
  }
  return tokenPair(trimmed.slice(0, sep), trimmed.slice(sep + 1));
}

export function normalizeIconName(value: string): string {
  const parsed = parseIconName(value);
  return parsed ? `${parsed.prefix}:${parsed.name}` : value;
}

function tokenPair(prefix: string, name: string): ParsedIconName | undefined {
  if (!/^[a-z][a-z0-9-]*$/i.test(prefix) || !/^[a-z0-9][a-z0-9-]*$/i.test(name)) {
    return undefined;
  }
  if (URL_SCHEMES.has(prefix.toLowerCase())) {
    return undefined;
  }
  return { prefix, name };
}

export function collectIconNamesFromText(text: string, into: Set<string> = new Set()): Set<string> {
  COLON_ICON.lastIndex = 0;
  for (const match of text.matchAll(COLON_ICON)) {
    addParsed(into, match[1], match[2]);
  }
  CLASS_ICON.lastIndex = 0;
  for (const match of text.matchAll(CLASS_ICON)) {
    addParsed(into, match[1], match[2]);
  }
  return into;
}

export function collectIconFieldNames(text: string, into: Set<string> = new Set()): Set<string> {
  ICON_FIELD.lastIndex = 0;
  for (const match of text.matchAll(ICON_FIELD)) {
    const parsed = match[1] ? parseIconName(match[1]) : undefined;
    if (parsed) {
      into.add(`${parsed.prefix}:${parsed.name}`);
    }
  }
  return into;
}

export function collectThemeIconNames(socialLinks: unknown): string[] {
  if (!Array.isArray(socialLinks)) {
    return [];
  }
  const names: string[] = [];
  for (const link of socialLinks) {
    if (!link || typeof link !== "object") {
      continue;
    }
    const icon = (link as { icon?: unknown }).icon;
    if (typeof icon === "string" && parseIconName(icon)) {
      names.push(normalizeIconName(icon));
    }
  }
  return names;
}

function addParsed(into: Set<string>, prefix: string | undefined, name: string | undefined): void {
  if (!prefix || !name) {
    return;
  }
  const parsed = tokenPair(prefix, name);
  if (parsed) {
    into.add(`${parsed.prefix}:${parsed.name}`);
  }
}

export function iconStylesheetHref(base?: string): string {
  return `${normalizeBasePath(base)}${ICON_ASSET_DIR}/${ICON_CSS_NAME}`;
}

export function iconStylesheetLink(base?: string): string {
  return `<link rel="stylesheet" href="${iconStylesheetHref(base)}">`;
}

export function withSelfHostedIconHead<T extends { head?: string }>(
  embed: T | undefined,
  enabled: boolean,
  base?: string,
): T | { head: string } | undefined {
  if (!enabled) {
    return embed;
  }
  const extra = iconStylesheetLink(base);
  if (!embed) {
    return { head: extra };
  }
  return { ...embed, head: embed.head ? `${extra}\n${embed.head}` : extra };
}

export interface WriteSelfHostedIconsOptions {
  options: ResolvedIconsOptions;
  outDir: string;
  root: string;
  srcDir?: string;
  socialLinks?: unknown;
}

export interface WriteSelfHostedIconsResult {
  files: string[];
  errors: string[];
  names: string[];
}

/** Copy resolved icon CSS into `outDir`. Missing collections or names become errors. */
export async function writeSelfHostedIcons(
  input: WriteSelfHostedIconsOptions,
): Promise<WriteSelfHostedIconsResult> {
  if (!input.options.enabled) {
    return { files: [], errors: [], names: [] };
  }
  const names = await collectResolvedIconNames(input);
  const { icons, errors } = await resolveIconBodies(names, input.root);
  const destDir = join(input.outDir, ICON_ASSET_DIR);
  await mkdir(destDir, { recursive: true });
  const cssPath = join(destDir, ICON_CSS_NAME);
  await writeFile(cssPath, renderIconsCss(icons), "utf8");
  return { files: [cssPath], errors, names };
}

export { iconClassName };

async function collectResolvedIconNames(input: WriteSelfHostedIconsOptions): Promise<string[]> {
  const names = new Set<string>();
  for (const item of input.options.safelist) {
    addName(names, item);
  }
  for (const item of collectThemeIconNames(input.socialLinks)) {
    names.add(item);
  }
  const { names: includeNames, globs } = partitionInclude(input.options.include);
  for (const item of includeNames) {
    names.add(item);
  }
  for (const pattern of globs) {
    const files = await glob(pattern, {
      cwd: input.root,
      nodir: true,
      absolute: true,
      ignore: ["**/node_modules/**"],
    });
    for (const file of files) {
      collectIconNamesFromText(await readFile(file, "utf8"), names);
    }
  }
  if (input.srcDir) {
    const files = await glob("**/*.{md,mdx,markdown}", {
      cwd: input.srcDir,
      nodir: true,
      absolute: true,
      ignore: ["**/node_modules/**"],
    });
    for (const file of files) {
      collectIconFieldNames(await readFile(file, "utf8"), names);
    }
  }
  return [...names].sort();
}

function partitionInclude(include: string[]): { names: string[]; globs: string[] } {
  const names: string[] = [];
  const globs: string[] = [];
  for (const entry of include) {
    if (parseIconName(entry)) {
      names.push(normalizeIconName(entry));
    } else {
      globs.push(entry);
    }
  }
  return { names, globs };
}

function addName(into: Set<string>, value: string): void {
  const parsed = parseIconName(value);
  if (parsed) {
    into.add(`${parsed.prefix}:${parsed.name}`);
  }
}

async function resolveIconBodies(
  names: string[],
  root: string,
): Promise<{ icons: ResolvedIcon[]; errors: string[] }> {
  const icons: ResolvedIcon[] = [];
  const errors: string[] = [];
  const collections = new Map<string, Awaited<ReturnType<typeof loadIconCollection>>>();
  for (const id of names) {
    const parsed = parseIconName(id);
    if (!parsed) {
      continue;
    }
    if (!collections.has(parsed.prefix)) {
      collections.set(parsed.prefix, await loadIconCollection(parsed.prefix, root));
    }
    const collection = collections.get(parsed.prefix);
    if (!collection) {
      errors.push(
        `[ox-content] icons: missing Iconify collection "${parsed.prefix}". Install @iconify-json/${parsed.prefix} or @iconify/json.`,
      );
      continue;
    }
    const found = lookupIcon(collection, parsed.name);
    if (!found) {
      errors.push(
        `[ox-content] icons: missing icon "${parsed.prefix}:${parsed.name}" in collection "${parsed.prefix}".`,
      );
      continue;
    }
    icons.push({
      prefix: parsed.prefix,
      name: parsed.name,
      body: found.body,
      width: found.width,
      height: found.height,
      multicolor: isMulticolorIcon(found.body),
    });
  }
  return { icons, errors };
}
