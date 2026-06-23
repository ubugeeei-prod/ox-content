import * as path from "node:path";
import { generateCollectionsModule } from "./collections-runtime";
import { importNapiModule } from "./napi";
import type {
  CollectionManifest,
  CollectionOptions,
  CollectionsOptions,
  ResolvedCollectionsOptions,
  ResolvedOptions,
} from "./types";

const DEFAULT_COLLECTION_NAME = "content";
const DEFAULT_COLLECTION_SOURCE = "**/*";

type NativeCollectionDefinition = {
  name: string;
  source: string[];
  include: string[];
};

type NativeTransformOptions = {
  gfm?: boolean;
  footnotes?: boolean;
  taskLists?: boolean;
  tables?: boolean;
  strikethrough?: boolean;
  frontmatter?: boolean;
  tocMaxDepth?: number;
  codeAnnotations?: boolean;
  codeAnnotationMetaKey?: string;
  codeAnnotationSyntax?: string;
  codeAnnotationDefaultLineNumbers?: boolean;
  wikiLinks?: { enabled?: boolean; baseUrl?: string };
  emojiShortcodes?: { enabled?: boolean; custom?: Record<string, string> };
  attributes?: { enabled?: boolean };
  cjkEmphasis?: boolean;
  codeImports?: { enabled?: boolean; rootDir?: string };
  editThisPage?: {
    enabled?: boolean;
    repoUrl?: string;
    branch?: string;
    rootDir?: string;
    label?: string;
  };
};

type BuildCollectionManifestNapi = {
  buildCollectionManifest: (options: {
    srcDir: string;
    extensions: string[];
    frontmatter?: boolean;
    collections: NativeCollectionDefinition[];
    transformOptions?: NativeTransformOptions;
  }) => string;
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
  const collections: ResolvedCollectionsOptions["collections"] = {};

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
  if (!options.collections.enabled) {
    return { collections: {} };
  }

  const napi = (await importNapiModule()) as unknown as BuildCollectionManifestNapi;
  const manifestJson = napi.buildCollectionManifest({
    srcDir: path.resolve(root, options.srcDir),
    extensions: [...options.extensions],
    frontmatter: options.frontmatter,
    collections: Object.values(options.collections.collections).map((collection) => ({
      name: collection.name,
      source: collection.source,
      include: collection.include,
    })),
    transformOptions: createNativeTransformOptions(options),
  });

  return parseCollectionManifest(manifestJson);
}

export async function generateCollectionsVirtualModule(
  root: string,
  options: ResolvedOptions,
): Promise<string> {
  return generateCollectionsModule(await buildCollectionManifest(root, options));
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

function parseCollectionManifest(json: string): CollectionManifest {
  const value = JSON.parse(json) as unknown;
  if (!value || typeof value !== "object" || !("collections" in value)) {
    throw new Error("[ox-content] Native collection manifest returned an invalid payload.");
  }
  return value as CollectionManifest;
}

function createNativeTransformOptions(options: ResolvedOptions): NativeTransformOptions {
  return {
    gfm: options.gfm,
    footnotes: options.footnotes,
    taskLists: options.taskLists,
    tables: options.tables,
    strikethrough: options.strikethrough,
    frontmatter: options.frontmatter,
    tocMaxDepth: options.tocMaxDepth,
    codeAnnotations: options.codeAnnotations?.enabled ?? false,
    codeAnnotationMetaKey: options.codeAnnotations?.metaKey ?? "annotate",
    codeAnnotationSyntax: options.codeAnnotations?.notation ?? "attribute",
    codeAnnotationDefaultLineNumbers: options.codeAnnotations?.defaultLineNumbers ?? false,
    wikiLinks: options.wikiLinks?.enabled
      ? {
          enabled: true,
          baseUrl: options.wikiLinks.baseUrl,
        }
      : undefined,
    emojiShortcodes: options.emojiShortcodes?.enabled
      ? {
          enabled: true,
          custom: options.emojiShortcodes.custom,
        }
      : undefined,
    attributes: options.attrs?.enabled ? { enabled: true } : undefined,
    cjkEmphasis: options.cjkEmphasis ?? false,
    codeImports: options.codeImports?.enabled
      ? {
          enabled: true,
          rootDir: options.codeImports.rootDir,
        }
      : undefined,
    editThisPage: options.editThisPage?.enabled
      ? {
          enabled: true,
          repoUrl: options.editThisPage.repoUrl,
          branch: options.editThisPage.branch,
          rootDir: options.editThisPage.rootDir,
          label: options.editThisPage.label,
        }
      : undefined,
  };
}

function defaultCollections(): CollectionsOptions {
  return {
    [DEFAULT_COLLECTION_NAME]: {
      source: DEFAULT_COLLECTION_SOURCE,
    },
  };
}
