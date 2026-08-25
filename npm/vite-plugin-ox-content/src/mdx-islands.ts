/**
 * Discover registered MDX islands from the mdast tree or rendered HTML.
 *
 * Framework plugins use this instead of a source regex when MDX is on, so
 * nested JSX, expression attributes, and fragments stay visible. Names that
 * are not in the global `components` map are left as static HTML.
 */

import { importNapiModule } from "./napi";

/** Global component map: object, Map, or name list. */
export type ComponentRegistry =
  | Readonly<Record<string, unknown>>
  | ReadonlyMap<string, unknown>
  | Iterable<string>;

const OX_ISLAND_NAME = /data-ox-island="([^"]+)"/g;

/**
 * Collect named MDX JSX tags from a parsed mdast tree (JSON from NAPI `parse()`).
 * Fragments (`name: null`) and non-JSX nodes are ignored. Walks nested children
 * so inner islands are found.
 */
export function collectMdxJsxNamesFromAst(ast: unknown): string[] {
  const names = new Set<string>();
  walkMdast(ast, names);
  return [...names];
}

/**
 * Collect `data-ox-island` names from Rust-rendered HTML.
 * Used when an AST walk is unavailable.
 */
export function collectMdxIslandNamesFromHtml(html: string): string[] {
  const names = new Set<string>();
  OX_ISLAND_NAME.lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = OX_ISLAND_NAME.exec(html)) !== null) {
    const name = match[1];
    if (name) names.add(decodeHtmlAttr(name));
  }
  return [...names];
}

/** Keep names that exist on the global component map, in first-seen order. */
export function intersectRegisteredComponentNames(
  names: Iterable<string>,
  components: ComponentRegistry,
): string[] {
  const used: string[] = [];
  for (const name of names) {
    if (isRegisteredComponent(name, components) && !used.includes(name)) {
      used.push(name);
    }
  }
  return used;
}

export interface DiscoverRegisteredMdxComponentsInput {
  /** Markdown/MDX body (frontmatter already stripped). */
  source: string;
  /** Rendered HTML, used when `parse()` is missing or the AST walk fails. */
  html?: string;
  components: ComponentRegistry;
}

/**
 * Resolve registered island names for an MDX document.
 *
 * Prefers a NAPI `parse()` AST walk. Falls back to rendered `data-ox-island`
 * names so plugins still hydrate if #659 metadata is not present.
 */
export async function discoverRegisteredMdxComponents(
  input: DiscoverRegisteredMdxComponentsInput,
): Promise<string[]> {
  const astNames = await tryCollectNamesFromParse(input.source);
  const names =
    astNames ?? (input.html !== undefined ? collectMdxIslandNamesFromHtml(input.html) : []);
  return intersectRegisteredComponentNames(names, input.components);
}

export function isRegisteredComponent(name: string, components: ComponentRegistry): boolean {
  if (isMapRegistry(components)) {
    return components.has(name);
  }
  if (isPlainObjectRegistry(components)) {
    return Object.prototype.hasOwnProperty.call(components, name);
  }
  for (const entry of components) {
    if (entry === name) return true;
  }
  return false;
}

async function tryCollectNamesFromParse(source: string): Promise<string[] | null> {
  try {
    const napi = await importNapiModule();
    const parsed = napi.parse(source, { mdx: true, gfm: true });
    if (!parsed.ast) return null;
    return collectMdxJsxNamesFromAst(JSON.parse(parsed.ast) as unknown);
  } catch {
    return null;
  }
}

function walkMdast(node: unknown, names: Set<string>): void {
  if (!node || typeof node !== "object") return;

  const record = node as { type?: unknown; name?: unknown; children?: unknown };
  if (
    (record.type === "mdxJsxFlowElement" || record.type === "mdxJsxTextElement") &&
    typeof record.name === "string" &&
    record.name
  ) {
    names.add(record.name);
  }

  if (Array.isArray(record.children)) {
    for (const child of record.children) {
      walkMdast(child, names);
    }
  }
}

function isMapRegistry(value: ComponentRegistry): value is ReadonlyMap<string, unknown> {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as ReadonlyMap<string, unknown>).has === "function" &&
    typeof (value as ReadonlyMap<string, unknown>).get === "function"
  );
}

function isPlainObjectRegistry(
  value: ComponentRegistry,
): value is Readonly<Record<string, unknown>> {
  return Object.prototype.toString.call(value) === "[object Object]";
}

function decodeHtmlAttr(value: string): string {
  return value
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&");
}
