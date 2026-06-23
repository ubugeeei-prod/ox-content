import * as fs from "node:fs/promises";
import * as path from "node:path";
import { glob } from "glob";
import { generateCollectionsModule } from "./collections-runtime";
import { isMarkdownFilePath, stripMarkdownExtension } from "./markdown";
import { importNapiModule } from "./napi";
import { transformMarkdown } from "./transform";
import type {
  CollectionEntry,
  CollectionManifest,
  CollectionOptions,
  CollectionsOptions,
  ResolvedCollectionOptions,
  ResolvedCollectionsOptions,
  ResolvedOptions,
} from "./types";

const DEFAULT_COLLECTION_NAME = "content";
const DEFAULT_COLLECTION_SOURCE = "**/*";
const DEFAULT_CONCURRENCY = 32;
const GLOB_IGNORE = ["**/node_modules/**", "**/.*/**", "**/.*"];

type PreparedSource = {
  content: string;
  frontmatter: Record<string, unknown>;
};

type PrepareSourceNapi = {
  prepareSource: (
    source: string,
    options?: { frontmatter?: boolean },
  ) => { content: string; frontmatter: Record<string, unknown> };
};

export function defineCollection<T extends CollectionOptions>(collection: T): T {
  return collection;
}

export function defineCollections<T extends CollectionsOptions>(collections: T): T {
  return collections;
}

export function resolveCollectionsOptions(
  options: CollectionsOptions | boolean | undefined,
): ResolvedCollectionsOptions {
  if (options === false) {
    return { enabled: false, collections: {} };
  }

  const source = options === true || options === undefined ? defaultCollections() : options;
  const collections: Record<string, ResolvedCollectionOptions> = {};

  for (const [name, value] of Object.entries(source)) {
    const collection = normalizeCollectionOptions(value);
    collections[name] = {
      name,
      source: normalizeSourcePatterns(collection.source),
      include: [...new Set(collection.include ?? [])],
    };
  }

  return { enabled: true, collections };
}

export async function buildCollectionManifest(
  root: string,
  options: ResolvedOptions,
): Promise<CollectionManifest> {
  const collections: CollectionManifest["collections"] = {};
  if (!options.collections.enabled) {
    return { collections };
  }

  const srcDir = path.resolve(root, options.srcDir);
  const collectionFiles = await Promise.all(
    Object.values(options.collections.collections).map(async (collection) => ({
      collection,
      files: await collectCollectionFiles(srcDir, collection, options.extensions),
    })),
  );
  const hasFiles = collectionFiles.some(({ files }) => files.length > 0);
  if (!hasFiles) {
    for (const { collection } of collectionFiles) {
      collections[collection.name] = [];
    }
    return { collections };
  }

  const napi = (await importNapiModule()) as PrepareSourceNapi;
  const preparedCache = new Map<string, Promise<PreparedSource>>();
  const transformCache = new Map<string, ReturnType<typeof transformMarkdown>>();

  for (const { collection, files } of collectionFiles) {
    collections[collection.name] = await mapLimit(files, DEFAULT_CONCURRENCY, (filePath) => {
      return createCollectionEntry(filePath, srcDir, collection, options, {
        preparedCache,
        transformCache,
        napi,
      });
    });
  }

  return { collections };
}

export async function generateCollectionsVirtualModule(
  root: string,
  options: ResolvedOptions,
): Promise<string> {
  return generateCollectionsModule(await buildCollectionManifest(root, options));
}

async function collectCollectionFiles(
  srcDir: string,
  collection: ResolvedCollectionOptions,
  extensions: readonly string[],
): Promise<string[]> {
  try {
    const files = await glob(collection.source, {
      absolute: true,
      cwd: srcDir,
      ignore: GLOB_IGNORE,
      nodir: true,
      windowsPathsNoEscape: true,
    });
    return files
      .filter((file) => isMarkdownFilePath(file, extensions))
      .sort((left, right) =>
        left.localeCompare(right, undefined, { numeric: true, sensitivity: "base" }),
      );
  } catch {
    return [];
  }
}

async function createCollectionEntry(
  filePath: string,
  srcDir: string,
  collection: ResolvedCollectionOptions,
  options: ResolvedOptions,
  caches: {
    preparedCache: Map<string, Promise<PreparedSource>>;
    transformCache: Map<string, ReturnType<typeof transformMarkdown>>;
    napi: PrepareSourceNapi;
  },
): Promise<CollectionEntry> {
  const relativePath = normalizePathSeparators(path.relative(srcDir, filePath));
  const prepared = await getPreparedSource(filePath, options, caches.napi, caches.preparedCache);
  const routePath = getCollectionPath(relativePath, options.extensions);
  const stem = routePath === "/" ? "" : routePath.slice(1);
  const include = new Set(collection.include);
  const frontmatter = prepared.frontmatter;
  const transformed =
    include.has("html") || include.has("toc")
      ? await getTransformedSource(filePath, options, caches.transformCache)
      : undefined;

  const title =
    stringValue(frontmatter.title) ??
    transformed?.toc.find((entry) => entry.depth === 1)?.text ??
    extractFirstHeading(prepared.content) ??
    formatTitleFromPath(stem || "index");
  const description = stringValue(frontmatter.description);
  const entry: CollectionEntry = {
    ...frontmatter,
    id: stem,
    collection: collection.name,
    path: routePath,
    stem,
    source: relativePath,
    extension: path.extname(relativePath),
    title,
    description,
    frontmatter,
  };

  if (include.has("body")) {
    entry.body = prepared.content;
  }
  if (include.has("html") && transformed) {
    entry.html = transformed.html;
  }
  if (include.has("toc") && transformed) {
    entry.toc = transformed.toc;
  }

  return entry;
}

async function getPreparedSource(
  filePath: string,
  options: ResolvedOptions,
  napi: PrepareSourceNapi,
  cache: Map<string, Promise<PreparedSource>>,
): Promise<PreparedSource> {
  const cached = cache.get(filePath);
  if (cached) return cached;
  const promise = (async () => {
    const source = await fs.readFile(filePath, "utf-8");
    return napi.prepareSource(source, { frontmatter: options.frontmatter });
  })();
  cache.set(filePath, promise);
  return promise;
}

function getTransformedSource(
  filePath: string,
  options: ResolvedOptions,
  cache: Map<string, ReturnType<typeof transformMarkdown>>,
): ReturnType<typeof transformMarkdown> {
  const cached = cache.get(filePath);
  if (cached) return cached;
  const promise = fs
    .readFile(filePath, "utf-8")
    .then((source) => transformMarkdown(source, filePath, options));
  cache.set(filePath, promise);
  return promise;
}

function normalizeCollectionOptions(
  options: CollectionOptions | string | readonly string[],
): CollectionOptions {
  if (typeof options === "string" || Array.isArray(options)) {
    return { source: options };
  }
  return options as CollectionOptions;
}

function normalizeSourcePatterns(source: CollectionOptions["source"]): string[] {
  const values = Array.isArray(source) ? source : [source ?? DEFAULT_COLLECTION_SOURCE];
  return values.map((value) => value || DEFAULT_COLLECTION_SOURCE);
}

function getCollectionPath(relativePath: string, extensions: readonly string[]): string {
  const stem = stripMarkdownExtension(relativePath, extensions);
  const segments = normalizePathSeparators(stem)
    .split("/")
    .filter(Boolean)
    .map((segment) => segment.replace(/^\d+\./, ""));

  if (segments.at(-1) === "index") {
    segments.pop();
  }

  return segments.length ? `/${segments.join("/")}` : "/";
}

function extractFirstHeading(content: string): string | undefined {
  for (const line of content.split(/\r?\n/)) {
    const match = /^(#{1,6})\s+(.+?)\s*#*\s*$/.exec(line);
    if (match) return match[2].trim();
  }
  return undefined;
}

function formatTitleFromPath(stem: string): string {
  const last = stem.split("/").filter(Boolean).at(-1) ?? stem;
  return last
    .replace(/[-_]+/g, " ")
    .replace(/\b\w/g, (char) => char.toUpperCase())
    .trim();
}

function normalizePathSeparators(value: string): string {
  return value.replaceAll(path.sep, "/");
}

function stringValue(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function defaultCollections(): CollectionsOptions {
  return {
    [DEFAULT_COLLECTION_NAME]: {
      source: DEFAULT_COLLECTION_SOURCE,
    },
  };
}

async function mapLimit<T, U>(
  values: readonly T[],
  concurrency: number,
  mapper: (value: T) => Promise<U>,
): Promise<U[]> {
  const results = new Array<U>(values.length);
  let cursor = 0;

  async function worker(): Promise<void> {
    while (cursor < values.length) {
      const index = cursor++;
      results[index] = await mapper(values[index]);
    }
  }

  await Promise.all(Array.from({ length: Math.min(concurrency, values.length) }, worker));
  return results;
}
