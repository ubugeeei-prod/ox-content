import { unified } from "unified";
import rehypeParse from "rehype-parse";
import rehypeStringify from "rehype-stringify";
import type { Element, Root } from "hast";
import type { CollectionAssetManifest } from "./collection-assets";

const DEFAULT_ORIGIN = "https://ox-content.local";
const DEFAULT_ATTRIBUTES = ["href", "src", "poster"] as const;

export interface RewriteCollectionAssetUrlsInput {
  html: string;
  pagePath: string;
  manifest: CollectionAssetManifest;
  origin?: string;
  document?: boolean;
  attributes?: readonly string[];
}

export interface CollectionAssetUrlRewrite {
  attribute: string;
  original: string;
  replacement: string;
  aliasPath: string;
  contentPath: string;
}

export interface RewriteCollectionAssetUrlsResult {
  html: string;
  rewrites: CollectionAssetUrlRewrite[];
}

/**
 * Rewrites host-rendered HTML references from collection aliases to their
 * content-addressed URLs. The default input form is an HTML fragment.
 */
export function rewriteCollectionAssetUrls(
  input: RewriteCollectionAssetUrlsInput,
): RewriteCollectionAssetUrlsResult {
  const aliases = collectionAssetAliasMap(input.manifest);
  if (aliases.size === 0 || input.html.length === 0) {
    return { html: input.html, rewrites: [] };
  }

  const origin = normalizeOrigin(input.origin);
  const base = new URL(input.pagePath || "/", origin);
  const attributes = new Set(input.attributes ?? DEFAULT_ATTRIBUTES);
  const processor = unified()
    .use(rehypeParse, { fragment: input.document !== true })
    .use(rehypeStringify);
  const tree = processor.parse(input.html) as Root;
  const rewrites: CollectionAssetUrlRewrite[] = [];

  visitElements(tree, (node) => {
    for (const attribute of attributes) {
      const value = node.properties?.[attribute];
      if (typeof value !== "string") {
        continue;
      }
      const rewrite = rewriteAttribute(value, attribute, aliases, base);
      if (!rewrite || rewrite.replacement === value) {
        continue;
      }
      node.properties[attribute] = rewrite.replacement;
      rewrites.push(rewrite);
    }
  });

  return {
    html: processor.stringify(tree),
    rewrites,
  };
}

function collectionAssetAliasMap(manifest: CollectionAssetManifest): Map<string, string> {
  const aliases = new Map<string, string>();
  for (const asset of manifest.assets) {
    const contentPath = normalizePublicPath(asset.contentPath);
    aliases.set(contentPath, contentPath);
    for (const publicPath of asset.publicPaths) {
      aliases.set(normalizePublicPath(publicPath), contentPath);
    }
  }
  return aliases;
}

function rewriteAttribute(
  original: string,
  attribute: string,
  aliases: ReadonlyMap<string, string>,
  base: URL,
): CollectionAssetUrlRewrite | undefined {
  if (!original.trim() || original.startsWith("#")) {
    return undefined;
  }
  let url: URL;
  try {
    url = new URL(original, base);
  } catch {
    return undefined;
  }
  if ((url.protocol !== "http:" && url.protocol !== "https:") || url.origin !== base.origin) {
    return undefined;
  }
  let aliasPath: string;
  try {
    aliasPath = normalizePublicPath(url.pathname);
  } catch {
    return undefined;
  }
  const contentPath = aliases.get(aliasPath);
  if (!contentPath) {
    return undefined;
  }
  return {
    attribute,
    original,
    aliasPath,
    contentPath,
    replacement: `${contentPath}${url.search}${url.hash}`,
  };
}

function visitElements(node: Root | Element, visit: (node: Element) => void): void {
  if (node.type === "element") {
    visit(node);
  }
  if (!("children" in node)) {
    return;
  }
  for (const child of node.children) {
    if (child.type === "element") {
      visitElements(child, visit);
    }
  }
}

function normalizeOrigin(origin: string | undefined): string {
  if (!origin) {
    return DEFAULT_ORIGIN;
  }
  const url = new URL(origin);
  return url.origin;
}

function normalizePublicPath(value: string): string {
  if (!value.startsWith("/") || value.startsWith("//") || value.includes("\0")) {
    throw new Error(
      `Collection asset public path ${JSON.stringify(value)} must be an absolute URL path.`,
    );
  }
  const segments = value.slice(1).split("/");
  if (segments.length === 0 || segments.some((segment) => !segment)) {
    throw new Error(
      `Collection asset public path ${JSON.stringify(value)} must not contain empty segments.`,
    );
  }
  return `/${segments.map((segment) => encodeSegment(segment, value)).join("/")}`;
}

function encodeSegment(segment: string, path: string): string {
  let decoded: string;
  try {
    decoded = decodeURIComponent(segment);
  } catch {
    throw new Error(
      `Collection asset public path ${JSON.stringify(path)} contains invalid URL encoding.`,
    );
  }
  if (
    !decoded ||
    decoded === "." ||
    decoded === ".." ||
    decoded.includes("/") ||
    decoded.includes("\\") ||
    hasControlCharacter(decoded)
  ) {
    throw new Error(`Collection asset public path ${JSON.stringify(path)} is unsafe.`);
  }
  return encodeURIComponent(decoded);
}

function hasControlCharacter(value: string): boolean {
  for (const character of value) {
    const code = character.charCodeAt(0);
    if (code < 32 || code === 127) {
      return true;
    }
  }
  return false;
}
