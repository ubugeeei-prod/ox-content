/**
 * Resolve Iconify JSON collections and emit CSS-mask rules.
 *
 * Collections come from installed `@iconify-json/*` or `@iconify/json`.
 * Tests supply fixture JSON under the project `root` — no network.
 */

import { createRequire } from "node:module";
import { existsSync } from "node:fs";
import { readFile } from "node:fs/promises";
import { join } from "node:path";

export interface IconifyIcon {
  body: string;
  width?: number;
  height?: number;
}

export interface IconifyJSON {
  prefix?: string;
  width?: number;
  height?: number;
  icons: Record<string, IconifyIcon>;
  aliases?: Record<string, { parent: string; width?: number; height?: number }>;
}

export interface ResolvedIcon {
  prefix: string;
  name: string;
  body: string;
  width: number;
  height: number;
  multicolor: boolean;
}

export function iconClassName(prefix: string, name: string): string {
  return `icon-[${prefix}--${name}]`;
}

export function iconCssSelector(prefix: string, name: string): string {
  return `.icon-\\[${prefix}--${name}\\]`;
}

export function resolveIconCollectionPath(prefix: string, root: string): string | undefined {
  const files = [
    join(root, "node_modules", "@iconify-json", prefix, "icons.json"),
    join(root, "node_modules", "@iconify", "json", "json", `${prefix}.json`),
  ];
  for (const file of files) {
    if (existsSync(file)) {
      return file;
    }
  }
  return resolveViaNode(prefix, root);
}

function resolveViaNode(prefix: string, root: string): string | undefined {
  try {
    return createRequire(join(root, "package.json")).resolve(`@iconify-json/${prefix}/icons.json`);
  } catch {
    try {
      return createRequire(join(root, "package.json")).resolve(`@iconify/json/json/${prefix}.json`);
    } catch {
      return undefined;
    }
  }
}

export async function loadIconCollection(
  prefix: string,
  root: string,
): Promise<IconifyJSON | undefined> {
  const path = resolveIconCollectionPath(prefix, root);
  if (!path) {
    return undefined;
  }
  const raw = await readFile(path, "utf8");
  return JSON.parse(raw) as IconifyJSON;
}

export function lookupIcon(
  collection: IconifyJSON,
  name: string,
): { body: string; width: number; height: number } | undefined {
  const fallback = collection.width ?? 16;
  const fallbackH = collection.height ?? fallback;
  const direct = collection.icons[name];
  if (direct) {
    return {
      body: direct.body,
      width: direct.width ?? fallback,
      height: direct.height ?? fallbackH,
    };
  }
  const alias = collection.aliases?.[name];
  if (!alias) {
    return undefined;
  }
  const parent = collection.icons[alias.parent];
  if (!parent) {
    return undefined;
  }
  return {
    body: parent.body,
    width: alias.width ?? parent.width ?? fallback,
    height: alias.height ?? parent.height ?? fallbackH,
  };
}

export function isMulticolorIcon(body: string): boolean {
  return /(?:fill|stroke)=["'](?!currentColor|none)[^"']+["']/i.test(body);
}

export function renderIconsCss(icons: ResolvedIcon[]): string {
  const rules = icons.map(renderOneIconCss);
  return `/* ox-content self-hosted Iconify icons */\n${rules.join("\n")}\n`;
}

function renderOneIconCss(icon: ResolvedIcon): string {
  const selector = iconCssSelector(icon.prefix, icon.name);
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${icon.width} ${icon.height}">${maskBody(icon)}</svg>`;
  const url = svgToDataUrl(svg);
  if (icon.multicolor) {
    return `${selector}{display:inline-block;width:1em;height:1em;background-color:transparent;background-image:${url};background-repeat:no-repeat;background-size:100% 100%}`;
  }
  return `${selector}{display:inline-block;width:1em;height:1em;background-color:currentColor;-webkit-mask-image:${url};mask-image:${url};-webkit-mask-repeat:no-repeat;mask-repeat:no-repeat;-webkit-mask-size:100% 100%;mask-size:100% 100%}`;
}

function maskBody(icon: ResolvedIcon): string {
  return icon.multicolor ? icon.body : icon.body.replace(/currentColor/g, "black");
}

function svgToDataUrl(svg: string): string {
  const encoded = svg
    .replace(/"/g, "'")
    .replace(/%/g, "%25")
    .replace(/#/g, "%23")
    .replace(/</g, "%3C")
    .replace(/>/g, "%3E")
    .replace(/\s+/g, " ");
  return `url("data:image/svg+xml,${encoded}")`;
}
