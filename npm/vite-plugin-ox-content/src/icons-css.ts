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

const SVG_TAG = /<\s*(\/)?\s*([A-Za-z][\w:.-]*)([^<>]*?)(\/?)\s*>/g;
const SVG_ATTRIBUTE = /([A-Za-z_:][\w:.-]*)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'=<>`]+))/g;
const PAINT_ATTRIBUTES = new Set(["fill", "stroke"]);
const ANIMATION_ELEMENTS = new Set([
  "animate",
  "animatecolor",
  "animatemotion",
  "animatetransform",
  "set",
]);
const SMIL_FILL_VALUES = new Set(["freeze", "remove"]);

interface SvgTag {
  name: string;
  attrs: string;
  closing: boolean;
  selfClosing: boolean;
}

interface SvgAttribute {
  name: string;
  value: string;
}

export function isMulticolorIcon(body: string): boolean {
  let maskDepth = 0;
  for (const tag of svgTags(body)) {
    if (tag.closing) {
      if (tag.name === "mask" && maskDepth > 0) {
        maskDepth -= 1;
      }
      continue;
    }

    const inMask = maskDepth > 0 || tag.name === "mask";
    if (!inMask && tagHasFixedVisiblePaint(tag.name, tag.attrs)) {
      return true;
    }

    if (tag.name === "mask" && !tag.selfClosing) {
      maskDepth += 1;
    }
  }
  return false;
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

function* svgTags(body: string): Iterable<SvgTag> {
  SVG_TAG.lastIndex = 0;
  for (const match of body.matchAll(SVG_TAG)) {
    const source = match[0]!;
    if (/^<\s*[!?]/.test(source)) {
      continue;
    }
    yield {
      name: localName(match[2]!),
      attrs: match[3] ?? "",
      closing: Boolean(match[1]),
      selfClosing: Boolean(match[4]) || /\/\s*>$/.test(source),
    };
  }
}

function tagHasFixedVisiblePaint(name: string, attrsSource: string): boolean {
  const attrs = svgAttributes(attrsSource);
  const isAnimation = ANIMATION_ELEMENTS.has(name);
  for (const attr of attrs) {
    if (attr.name === "style" && !isAnimation && styleHasFixedPaint(attr.value)) {
      return true;
    }
    if (!PAINT_ATTRIBUTES.has(attr.name)) {
      continue;
    }
    if (isAnimation && attr.name === "fill" && isSmilTimingFill(attr.value)) {
      continue;
    }
    if (isFixedPaintValue(attr.value)) {
      return true;
    }
  }
  return isAnimation && animationTargetsFixedPaint(attrs);
}

function svgAttributes(source: string): SvgAttribute[] {
  const attrs: SvgAttribute[] = [];
  SVG_ATTRIBUTE.lastIndex = 0;
  for (const match of source.matchAll(SVG_ATTRIBUTE)) {
    attrs.push({
      name: localName(match[1]!),
      value: match[2] ?? match[3] ?? match[4] ?? "",
    });
  }
  return attrs;
}

function styleHasFixedPaint(style: string): boolean {
  for (const declaration of style.split(";")) {
    const separator = declaration.indexOf(":");
    if (separator <= 0) {
      continue;
    }
    const property = localName(declaration.slice(0, separator).trim());
    if (PAINT_ATTRIBUTES.has(property) && isFixedPaintValue(declaration.slice(separator + 1))) {
      return true;
    }
  }
  return false;
}

function animationTargetsFixedPaint(attrs: SvgAttribute[]): boolean {
  const attributeName = attrs.find((attr) => attr.name === "attributename")?.value.trim();
  if (!attributeName || !PAINT_ATTRIBUTES.has(localName(attributeName))) {
    return false;
  }
  return attrs.some(
    (attr) =>
      (attr.name === "from" ||
        attr.name === "to" ||
        attr.name === "by" ||
        attr.name === "values") &&
      attr.value.split(";").some(isFixedPaintValue),
  );
}

function isSmilTimingFill(value: string): boolean {
  return SMIL_FILL_VALUES.has(cleanPaintValue(value));
}

function isFixedPaintValue(value: string): boolean {
  const normalized = cleanPaintValue(value);
  return normalized !== "" && normalized !== "currentcolor" && normalized !== "none";
}

function cleanPaintValue(value: string): string {
  return value
    .trim()
    .replace(/\s*!important\s*$/i, "")
    .toLowerCase();
}

function localName(name: string): string {
  const normalized = name.toLowerCase();
  const separator = normalized.lastIndexOf(":");
  return separator === -1 ? normalized : normalized.slice(separator + 1);
}
