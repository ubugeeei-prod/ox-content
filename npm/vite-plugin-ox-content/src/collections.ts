import * as path from "node:path";
import { applyCollectionRoutes } from "./apply-permalinks";
import { toJsDataTableOptions } from "./data-table-options";
import { toJsFileTreeOptions } from "./file-tree-options";
import { toJsImageGalleryOptions } from "./image-gallery-options";
import { toJsPartialsOptions } from "./partials-options";
import { toJsTimelineOptions } from "./timeline-options";
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
  semanticFootnotes?: boolean;
  taskLists?: boolean;
  tables?: boolean;
  strikethrough?: boolean;
  autolinks?: boolean;
  autolinkUrls?: boolean;
  frontmatter?: boolean;
  tocMaxDepth?: number;
  headingPermalinks?: boolean;
  codeAnnotations?: boolean;
  codeAnnotationMetaKey?: string;
  codeAnnotationSyntax?: string;
  codeAnnotationDefaultLineNumbers?: boolean;
  wikiLinks?: { enabled?: boolean; baseUrl?: string };
  emojiShortcodes?: { enabled?: boolean; custom?: Record<string, string> };
  attributes?: { enabled?: boolean };
  badges?: { enabled?: boolean };
  notByAi?: { enabled?: boolean; label?: string; href?: string };
  keyboardKeys?: {
    enabled?: boolean;
    aliases?: Record<string, string>;
    style?: string;
  };
  abbreviations?: {
    enabled?: boolean;
    terms?: Record<string, string>;
    firstUseOnly?: boolean;
  };
  definitionLists?: {
    enabled?: boolean;
  };
  magicLinks?: {
    enabled?: boolean;
    aliases?: Record<string, { href: string; label?: string; image?: string }>;
    favicon?: boolean;
    faviconTemplate?: string;
    imageOverrides?: Array<{ href?: string; prefix?: string; image: string }>;
  };
  containers?: {
    enabled?: boolean;
    types?: Record<string, { title?: string; tag?: string }>;
  };
  images?: { enabled?: boolean; lazy?: boolean };
  imageGalleries?: {
    enabled?: boolean;
    lazy?: boolean;
    missingAlt?: "error" | "warn" | "ignore";
    empty?: "error" | "warn" | "ignore";
  };
  timelines?: {
    enabled?: boolean;
    ordered?: boolean;
    invalidDate?: "error" | "warn" | "ignore";
    unknownMeta?: "error" | "warn" | "ignore";
    empty?: "error" | "warn" | "ignore";
  };
  cjkEmphasis?: boolean;
  codeImports?: { enabled?: boolean; rootDir?: string };
  includes?: { enabled?: boolean; rootDir?: string };
  partials?: { enabled?: boolean; rootDir?: string; root?: string; missing?: string };
  steps?: { enabled?: boolean };
  fileTree?: {
    enabled?: boolean;
    defaultOpen?: boolean;
    icons?: boolean;
    iconFolder?: string;
    iconFolderOpen?: string;
    iconFile?: string;
    iconFiles?: Record<string, string>;
  };
  dataTables?: {
    enabled?: boolean;
    rootDir?: string;
    missing?: "error" | "warn";
  };
  editThisPage?: {
    enabled?: boolean;
    repoUrl?: string;
    branch?: string;
    rootDir?: string;
    label?: string;
  };
  math?: boolean | { enabled?: boolean };
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

  const { manifest, errors } = applyCollectionRoutes(
    parseCollectionManifest(manifestJson),
    options.permalinks,
    options.cascade,
  );
  for (const error of errors) {
    console.warn(error);
  }
  return manifest;
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
    semanticFootnotes: options.semanticFootnotes ?? false,
    taskLists: options.taskLists,
    tables: options.tables,
    strikethrough: options.strikethrough,
    autolinks: options.autolinks,
    autolinkUrls: options.autolinks,
    frontmatter: options.frontmatter,
    tocMaxDepth: options.tocMaxDepth,
    headingPermalinks: options.headingPermalinks?.enabled ?? false,
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
    badges: options.badges?.enabled ? { enabled: true } : undefined,
    notByAi: options.notByAi?.enabled
      ? {
          enabled: true,
          label: options.notByAi.label,
          href: options.notByAi.href,
        }
      : undefined,
    keyboardKeys: options.keyboardKeys?.enabled
      ? {
          enabled: true,
          aliases: options.keyboardKeys.aliases,
          style: options.keyboardKeys.style,
        }
      : undefined,
    abbreviations: options.abbreviations?.enabled
      ? {
          enabled: true,
          terms: options.abbreviations.terms,
          firstUseOnly: options.abbreviations.firstUseOnly,
        }
      : undefined,
    definitionLists: options.definitionLists?.enabled ? { enabled: true } : undefined,
    magicLinks: options.magicLinks?.enabled
      ? {
          enabled: true,
          aliases: options.magicLinks.aliases,
          favicon: options.magicLinks.favicon,
          faviconTemplate: options.magicLinks.faviconTemplate,
          imageOverrides: options.magicLinks.imageOverrides,
        }
      : undefined,
    containers: options.containers?.enabled
      ? {
          enabled: true,
          types: options.containers.types,
        }
      : undefined,
    images: options.images?.enabled
      ? {
          enabled: true,
          lazy: options.images.lazy,
        }
      : undefined,
    imageGalleries: toJsImageGalleryOptions(options.imageGalleries, options.images),
    timelines: toJsTimelineOptions(options.timelines),
    cjkEmphasis: options.cjkEmphasis ?? false,
    codeImports: options.codeImports?.enabled
      ? {
          enabled: true,
          rootDir: options.codeImports.rootDir,
        }
      : undefined,
    includes: options.includes?.enabled
      ? {
          enabled: true,
          rootDir: options.includes.rootDir,
        }
      : undefined,
    partials: toJsPartialsOptions(options.partials),
    cards: options.cards?.enabled ? { enabled: true } : undefined,
    steps: options.steps?.enabled ? { enabled: true } : undefined,
    codeGroups: options.codeGroups?.enabled ? { enabled: true } : undefined,
    fileTree: toJsFileTreeOptions(options.fileTree),
    dataTables: toJsDataTableOptions(options.dataTables),
    editThisPage: options.editThisPage?.enabled
      ? {
          enabled: true,
          repoUrl: options.editThisPage.repoUrl,
          branch: options.editThisPage.branch,
          rootDir: options.editThisPage.rootDir,
          label: options.editThisPage.label,
        }
      : undefined,
    math: options.math?.enabled ?? false,
  };
}

function defaultCollections(): CollectionsOptions {
  return {
    [DEFAULT_COLLECTION_NAME]: {
      source: DEFAULT_COLLECTION_SOURCE,
    },
  };
}
