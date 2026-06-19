import rehypeParse from "rehype-parse";
import { unified } from "unified";
import type { Element, Nodes, Root } from "hast";
import type { ResolvedOptions, TocEntry } from "./types";

export type FrameworkRenderTarget = "html" | "native";

export interface FrameworkMarkdownOptions {
  srcDir: string;
  outDir: string;
  base: string;
  extensions: string[];
  gfm: boolean;
  frontmatter?: boolean;
  toc: boolean;
  tocMaxDepth: number;
  codeAnnotations?: {
    enabled?: boolean;
    metaKey?: string;
  };
  embeds?: {
    github?: ResolvedOptions["embeds"]["github"];
    openGraph?: ResolvedOptions["embeds"]["openGraph"];
  };
}

export interface FrameworkComponentIsland {
  name: string;
  props: Record<string, unknown>;
  id: string;
  content?: string;
}

export interface FrameworkTransformData {
  html: string;
  frontmatter: Record<string, unknown>;
  toc: TocEntry[];
}

export function createFrameworkMarkdownOptions(options: FrameworkMarkdownOptions): ResolvedOptions {
  return {
    srcDir: options.srcDir,
    outDir: options.outDir,
    base: options.base,
    extensions: options.extensions,
    ssg: {
      enabled: false,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
    },
    gfm: options.gfm,
    frontmatter: options.frontmatter ?? false,
    toc: options.toc,
    tocMaxDepth: options.tocMaxDepth,
    codeAnnotations: {
      enabled: options.codeAnnotations?.enabled ?? false,
      notation: "attribute",
      metaKey: options.codeAnnotations?.metaKey ?? "annotate",
      defaultLineNumbers: false,
    },
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    highlight: false,
    highlightTheme: "github-dark",
    highlightLangs: [],
    mermaid: false,
    ogImage: false,
    ogImageOptions: {
      vuePlugin: "vitejs",
      width: 1200,
      height: 630,
      cache: true,
      concurrency: 1,
    },
    transformers: [],
    docs: false,
    ogViewer: false,
    search: {
      enabled: false,
      limit: 10,
      prefix: true,
      placeholder: "Search...",
      hotkey: "k",
    },
    embeds: {
      github: options.embeds?.github ?? {},
      openGraph: options.embeds?.openGraph ?? {},
      pm: false,
      spotify: false,
      stackBlitz: false,
      twitter: false,
      bluesky: false,
      webContainer: false,
    },
    i18n: false,
    wikiLinks: { enabled: false, baseUrl: options.base },
    emojiShortcodes: { enabled: false, custom: {} },
    attrs: { enabled: false },
    codeImports: { enabled: false },
    sanitize: { enabled: false },
    editThisPage: { enabled: false, branch: "main", label: "Edit this page" },
    cjkEmphasis: false,
    codeBlockLint: { enabled: false, requireLanguage: false, trailingSpaces: true, mode: "warn" },
    codeBlockTypecheck: {
      enabled: false,
      languages: ["ts", "tsx"],
      requireMeta: true,
      tsgoCommand: "tsgo",
      mode: "warn",
    },
    docsTests: {
      enabled: false,
      languages: ["js", "jsx", "ts", "tsx"],
      requireMeta: true,
    },
  } as ResolvedOptions;
}

export function renderHtmlToReactCreateElement(
  html: string,
  islands: readonly FrameworkComponentIsland[] = [],
): string {
  const children = parseHtmlChildren(html).map((node) => renderReactNode(node, islands));
  return `createElement('div', { className: 'ox-content' }${renderReactChildren(children)})`;
}

export function renderHtmlToVueH(
  html: string,
  islands: readonly FrameworkComponentIsland[] = [],
): string {
  const children = parseHtmlChildren(html).map((node) => renderVueNode(node, islands));
  return `h('div', { class: 'ox-content' }${renderVueChildren(children)})`;
}

export function escapeSvelteMarkup(html: string): string {
  return html.replaceAll("{", "&#123;").replaceAll("}", "&#125;");
}

function parseHtmlChildren(html: string): Nodes[] {
  const tree = unified().use(rehypeParse, { fragment: true }).parse(html) as Root;
  return tree.children;
}

function renderReactNode(node: Nodes, islands: readonly FrameworkComponentIsland[]): string {
  if (node.type === "text") {
    return JSON.stringify(node.value);
  }

  if (node.type !== "element") {
    return "null";
  }

  const island = findIslandForElement(node, islands);
  if (island) {
    return renderReactIsland(island);
  }

  const props = renderReactProps(node.properties ?? {});
  const children = node.children.map((child) => renderReactNode(child, islands));
  return `createElement(${JSON.stringify(node.tagName)}, ${props}${renderReactChildren(children)})`;
}

function renderReactIsland(island: FrameworkComponentIsland): string {
  const props = renderObjectLiteral(island.props);
  const content = island.content ? `, ${JSON.stringify(island.content)}` : "";
  return `createElement(${island.name}, ${props}${content})`;
}

function renderReactChildren(children: string[]): string {
  const rendered = children.filter((child) => child !== "null");
  return rendered.length > 0 ? `, ${rendered.join(", ")}` : "";
}

function renderVueNode(node: Nodes, islands: readonly FrameworkComponentIsland[]): string {
  if (node.type === "text") {
    return JSON.stringify(node.value);
  }

  if (node.type !== "element") {
    return "null";
  }

  const island = findIslandForElement(node, islands);
  if (island) {
    return renderVueIsland(island);
  }

  const props = renderVueProps(node.properties ?? {});
  const children = node.children.map((child) => renderVueNode(child, islands));
  return `h(${JSON.stringify(node.tagName)}, ${props}${renderVueChildren(children)})`;
}

function renderVueIsland(island: FrameworkComponentIsland): string {
  const props = renderObjectLiteral(island.props);
  const content = island.content ? `, ${JSON.stringify(island.content)}` : "";
  return `h(${island.name}, ${props}${content})`;
}

function renderVueChildren(children: string[]): string {
  const rendered = children.filter((child) => child !== "null");
  if (rendered.length === 0) return "";
  if (rendered.length === 1) return `, ${rendered[0]}`;
  return `, [${rendered.join(", ")}]`;
}

function findIslandForElement(
  node: Element,
  islands: readonly FrameworkComponentIsland[],
): FrameworkComponentIsland | undefined {
  const islandId = getPropertyString(node.properties, "dataOxId", "data-ox-id");
  if (!islandId) return undefined;
  return islands.find((island) => island.id === islandId);
}

function renderReactProps(properties: Element["properties"]): string {
  const entries: string[] = [];

  for (const [key, value] of Object.entries(properties)) {
    const propName = toReactPropName(key);
    if (shouldSkipProperty(propName, value)) continue;

    if (propName === "style" && typeof value === "string") {
      entries.push(`${JSON.stringify(propName)}: ${renderStyleObject(value)}`);
      continue;
    }

    entries.push(`${JSON.stringify(propName)}: ${renderPropertyValue(value)}`);
  }

  return entries.length > 0 ? `{ ${entries.join(", ")} }` : "null";
}

function renderVueProps(properties: Element["properties"]): string {
  const entries: string[] = [];

  for (const [key, value] of Object.entries(properties)) {
    const propName = toVuePropName(key);
    if (shouldSkipProperty(propName, value)) continue;
    entries.push(`${JSON.stringify(propName)}: ${renderPropertyValue(value)}`);
  }

  return entries.length > 0 ? `{ ${entries.join(", ")} }` : "null";
}

function shouldSkipProperty(name: string, value: unknown): boolean {
  return name.startsWith("data-ox-") || value === null || value === undefined || value === false;
}

function toReactPropName(name: string): string {
  if (name === "class" || name === "className") return "className";
  if (name === "for" || name === "htmlFor") return "htmlFor";
  return toDataOrAriaAttributeName(name);
}

function toVuePropName(name: string): string {
  if (name === "className") return "class";
  if (name === "htmlFor") return "for";
  return toDataOrAriaAttributeName(name);
}

function toDataOrAriaAttributeName(name: string): string {
  if (name.startsWith("data") && /[A-Z]/.test(name)) {
    return camelToKebab(name);
  }
  if (name.startsWith("aria") && /[A-Z]/.test(name)) {
    return camelToKebab(name);
  }
  return name;
}

function camelToKebab(value: string): string {
  return value.replace(/[A-Z]/g, (char) => `-${char.toLowerCase()}`);
}

function renderPropertyValue(value: unknown): string {
  if (Array.isArray(value)) {
    return JSON.stringify(value.join(" "));
  }
  return JSON.stringify(value);
}

function renderStyleObject(value: string): string {
  const entries = value
    .split(";")
    .map((part) => part.trim())
    .filter(Boolean)
    .map((declaration) => {
      const separator = declaration.indexOf(":");
      if (separator === -1) return null;
      const name = declaration.slice(0, separator).trim();
      const cssValue = declaration.slice(separator + 1).trim();
      if (!name || !cssValue) return null;
      return `${JSON.stringify(cssPropertyToReactName(name))}: ${JSON.stringify(cssValue)}`;
    })
    .filter((entry): entry is string => Boolean(entry));

  return entries.length > 0 ? `{ ${entries.join(", ")} }` : "{}";
}

function cssPropertyToReactName(name: string): string {
  if (name.startsWith("--")) return name;
  return name.replace(/-([a-z])/g, (_, char: string) => char.toUpperCase());
}

function renderObjectLiteral(value: Record<string, unknown>): string {
  const entries = Object.entries(value).map(
    ([key, propValue]) => `${JSON.stringify(key)}: ${JSON.stringify(propValue)}`,
  );
  return entries.length > 0 ? `{ ${entries.join(", ")} }` : "null";
}

function getPropertyString(
  properties: Element["properties"],
  ...keys: string[]
): string | undefined {
  for (const key of keys) {
    const value = properties[key];
    if (typeof value === "string") return value;
    if (typeof value === "number" || typeof value === "boolean") return String(value);
  }
  return undefined;
}
