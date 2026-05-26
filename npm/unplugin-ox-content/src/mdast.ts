import { createRequire } from "node:module";
import { deserializeMdastFromRaw } from "./mdast-raw";
import type {
  MdastNode,
  MdastPluginContext,
  MdastRoot,
  MdastTransformer,
  OxContentMdastPlugin,
  ResolvedOptions,
  TocEntry,
} from "./types";

const require = createRequire(import.meta.url);

interface NapiBindings {
  parseTransferRaw: (
    source: string,
    kind: string,
    options?: {
      gfm?: boolean;
      footnotes?: boolean;
      taskLists?: boolean;
      tables?: boolean;
      strikethrough?: boolean;
      autolinks?: boolean;
    },
  ) => Uint8Array;
}

/**
 * Parser options for the Ox Content mdast unified plugin.
 */
export interface OxContentMdastOptions {
  /**
   * Enable GitHub Flavored Markdown extensions.
   * @default true
   */
  gfm?: boolean;

  /**
   * Enable footnotes.
   * @default true
   */
  footnotes?: boolean;

  /**
   * Enable task lists.
   * @default true
   */
  taskLists?: boolean;

  /**
   * Enable tables.
   * @default true
   */
  tables?: boolean;

  /**
   * Enable strikethrough.
   * @default true
   */
  strikethrough?: boolean;

  /**
   * Enable autolinks.
   * @default true
   */
  autolinks?: boolean;
}

type ProcessorWithParser = {
  parser?: (document: string) => MdastRoot;
};

let cachedNapiBindings: NapiBindings | null | undefined;
const DEFAULT_MDAST_OPTIONS: Required<OxContentMdastOptions> = {
  gfm: true,
  footnotes: true,
  taskLists: true,
  tables: true,
  strikethrough: true,
  autolinks: true,
};

/**
 * Unified parser plugin backed by Ox Content's native mdast parser.
 *
 * This lets existing remark/unified plugins run on top of Ox Content's parser
 * while keeping a more mdast-native integration point.
 */
export function oxContentMdast(
  this: ProcessorWithParser,
  options: OxContentMdastOptions = {},
): void {
  this.parser = (document: string) => parseMarkdownToMdast(document, options);
}

/**
 * Defines an Ox Content-native mdast plugin.
 */
export function defineMdastPlugin(name: string, transform: MdastTransformer): OxContentMdastPlugin {
  return { name, transform };
}

/**
 * Parses markdown source into an mdast-compatible tree using the native parser.
 */
export function parseMarkdownToMdast(
  source: string,
  options: OxContentMdastOptions = {},
): MdastRoot {
  const napi = loadNapiBindings();
  const resolvedOptions = resolveMdastOptions(options);
  const parserOptions = {
    gfm: resolvedOptions.gfm,
    footnotes: resolvedOptions.footnotes,
    taskLists: resolvedOptions.taskLists,
    tables: resolvedOptions.tables,
    strikethrough: resolvedOptions.strikethrough,
    autolinks: resolvedOptions.autolinks,
  };
  const buffer = requireNapiMethod(napi.parseTransferRaw, "parseTransferRaw")(
    source,
    "mdast",
    parserOptions,
  );
  return deserializeMdastFromRaw(buffer, source);
}

/**
 * Wraps an Ox Content-native mdast plugin as a standard unified transformer.
 */
export function toUnifiedMdastPlugin(
  plugin: OxContentMdastPlugin,
  context: MdastPluginContext,
): () => (tree: MdastRoot) => MdastRoot | Promise<MdastRoot> {
  return () => {
    return async (tree: MdastRoot) => {
      const nextTree = await plugin.transform(tree, context);
      return nextTree ?? tree;
    };
  };
}

/**
 * Extracts a nested TOC tree from mdast after mdast-stage plugins have run.
 */
export function extractTocFromMdast(root: MdastRoot, maxDepth: number): TocEntry[] {
  const headings: TocEntry[] = [];

  visitMdast(root, (node) => {
    if (node.type !== "heading") {
      return;
    }

    const depth = typeof node.depth === "number" ? node.depth : 0;
    if (depth < 1 || depth > maxDepth) {
      return;
    }

    const text = collectMdastText(node);
    headings.push({
      depth,
      text,
      slug: slugify(text),
      children: [],
    });
  });

  return buildTocTree(headings);
}

/**
 * Creates the context passed to Ox Content-native mdast plugins.
 */
export function createMdastPluginContext(
  filePath: string,
  source: string,
  frontmatter: Record<string, unknown>,
  options: ResolvedOptions,
  sourceOffset?: MdastPluginContext["sourceOffset"],
): MdastPluginContext {
  return {
    filePath,
    source,
    frontmatter,
    sourceOffset,
    options,
  };
}

function loadNapiBindings(): NapiBindings {
  if (cachedNapiBindings) {
    return cachedNapiBindings;
  }

  if (cachedNapiBindings === null) {
    throw new Error(
      "[ox-content] Failed to load @ox-content/napi. Please ensure the NAPI module is built. " +
        "Run: mise run build:napi",
    );
  }

  try {
    cachedNapiBindings = require("@ox-content/napi") as NapiBindings;
    return cachedNapiBindings;
  } catch {
    cachedNapiBindings = null;
    throw new Error(
      "[ox-content] Failed to load @ox-content/napi. Please ensure the NAPI module is built. " +
        "Run: mise run build:napi",
    );
  }
}

function requireNapiMethod<T extends (...args: never[]) => unknown>(
  method: T | undefined,
  name: string,
): T {
  if (typeof method === "function") {
    return method;
  }

  throw new Error(
    `[ox-content] @ox-content/napi is too old for mdast interop: missing ${name}(). ` +
      "Rebuild the NAPI module with `vp run build:napi`.",
  );
}

function resolveMdastOptions(options: OxContentMdastOptions): Required<OxContentMdastOptions> {
  return {
    gfm: options.gfm ?? DEFAULT_MDAST_OPTIONS.gfm,
    footnotes: options.footnotes ?? DEFAULT_MDAST_OPTIONS.footnotes,
    taskLists: options.taskLists ?? DEFAULT_MDAST_OPTIONS.taskLists,
    tables: options.tables ?? DEFAULT_MDAST_OPTIONS.tables,
    strikethrough: options.strikethrough ?? DEFAULT_MDAST_OPTIONS.strikethrough,
    autolinks: options.autolinks ?? DEFAULT_MDAST_OPTIONS.autolinks,
  };
}

function buildTocTree(entries: TocEntry[]): TocEntry[] {
  const result: TocEntry[] = [];
  const stack: TocEntry[] = [];

  for (const entry of entries) {
    const tocEntry: TocEntry = {
      depth: entry.depth,
      text: entry.text,
      slug: entry.slug,
      children: [],
    };

    while (stack.length > 0 && stack[stack.length - 1].depth >= entry.depth) {
      stack.pop();
    }

    if (stack.length === 0) {
      result.push(tocEntry);
    } else {
      stack[stack.length - 1].children.push(tocEntry);
    }

    stack.push(tocEntry);
  }

  return result;
}

function visitMdast(node: { children?: unknown }, visitor: (node: MdastNode) => void): void {
  if (!node || typeof node !== "object") {
    return;
  }

  if ("type" in node && typeof node.type === "string") {
    visitor(node as MdastNode);
  }

  if (!Array.isArray(node.children)) {
    return;
  }

  for (const child of node.children) {
    visitMdast(child as { children?: unknown }, visitor);
  }
}

function collectMdastText(node: { value?: unknown; children?: unknown }): string {
  if (!node || typeof node !== "object") {
    return "";
  }

  if (typeof node.value === "string") {
    return node.value;
  }

  if (!Array.isArray(node.children)) {
    return "";
  }

  return node.children.map((child) => collectMdastText(child as { value?: unknown })).join("");
}

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^\p{L}\p{N}\s-]+/gu, " ")
    .trim()
    .split(/\s+/)
    .filter(Boolean)
    .join("-");
}
