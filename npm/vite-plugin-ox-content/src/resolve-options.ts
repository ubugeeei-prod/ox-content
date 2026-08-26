import { resolveBlogOptions } from "./blog";
import { resolveBuiltinEmbedOptions } from "./builtin-embed-options";
import { resolveCardOptions } from "./card-options";
import { resolveCodeGroupOptions } from "./code-group-options";
import { resolveConditionalBlockOptions } from "./conditional-block-options";
import { resolveCollectionsOptions } from "./collections";
import { resolveDocsOptions } from "./docs";
import { resolveFeedsOptions } from "./feeds";
import { resolveDataTableOptions } from "./data-table-options";
import { resolveFileTreeOptions } from "./file-tree-options";
import { resolveGraphvizOptions } from "./plugins/graphviz";
import { resolveIconsOptions } from "./icons";
import { resolveHeadingPermalinksOptions } from "./heading-permalinks-options";
import { resolveI18nOptions } from "./i18n";
import { resolveImageGalleryOptions } from "./image-gallery-options";
import { resolveIncludeOptions } from "./include-options";
import { resolvePartialsOptions } from "./partials-options";
import { resolveAbbreviationsOptions } from "./abbreviations-options";
import { resolveMagicLinkOptions } from "./magic-link-options";
import { resolveDefinitionListOptions } from "./definition-list-options";
import { resolveNotByAiOptions } from "./not-by-ai-options";
import { normalizeMarkdownExtensions } from "./markdown";
import { resolveOgImageOptions } from "./og-image";
import { resolveCascadeOptions, resolvePermalinksOptions } from "./permalinks";
import { resolvePublishStateOptions } from "./publish-state";
import { resolvePwaOptions } from "./pwa";
import { resolveRedirectsOptions } from "./redirects";
import { resolveImageOptions } from "./resolve-image-options";
import { resolveResourcesOptions } from "./resources";
import { resolveSearchOptions } from "./search";
import { resolveSiteMapsOptions } from "./site-maps";
import { resolveSsgOptions } from "./ssg";
import { resolveStepsOptions } from "./step-options";
import { resolveTaxonomiesOptions } from "./taxonomies";
import { resolveTimelineOptions } from "./timeline-options";
import { resolveTypedHoverOptions } from "./typed-hover";
import type { OxContentOptions, ResolvedOptions } from "./types";
import { resolveVersionsOptions } from "./versions";

export { resolveBuiltinEmbedOptions } from "./builtin-embed-options";

/** Resolves plugin options with defaults. */
export function resolveOptions(options: OxContentOptions): ResolvedOptions {
  return {
    srcDir: options.srcDir ?? "content",
    outDir: options.outDir ?? "dist",
    base: options.base ?? "/",
    extensions: normalizeMarkdownExtensions(options.extensions),
    ssg: resolveSsgOptions(options.ssg),
    siteMaps: resolveSiteMapsOptions(options.siteMaps),
    publishState: resolvePublishStateOptions(options.publishState),
    permalinks: resolvePermalinksOptions(options.permalinks),
    cascade: resolveCascadeOptions(options.cascade),
    redirects: resolveRedirectsOptions(options.redirects),
    blog: resolveBlogOptions(
      options.blog ??
        (typeof options.ssg === "object" && options.ssg ? options.ssg.blog : undefined),
    ),
    feeds: resolveFeedsOptions(options.feeds),
    pwa: resolvePwaOptions(options.pwa),
    icons: resolveIconsOptions(options.icons),
    taxonomies: resolveTaxonomiesOptions(options.taxonomies),
    versions: resolveVersionsOptions(options.versions),
    resources: resolveResourcesOptions(options.resources),
    gfm: options.gfm ?? true,
    mdx: options.mdx,
    footnotes: options.footnotes ?? true,
    semanticFootnotes: options.semanticFootnotes ?? false,
    tables: options.tables ?? true,
    taskLists: options.taskLists ?? true,
    strikethrough: options.strikethrough ?? true,
    autolinks: options.autolinks ?? options.gfm ?? true,
    highlight: options.highlight ?? false,
    codeAnnotations: resolveCodeAnnotationsOptions(options.codeAnnotations),
    wikiLinks: resolveWikiLinkOptions(options.wikiLinks, options.base ?? "/"),
    emojiShortcodes: resolveEmojiShortcodeOptions(options.emojiShortcodes),
    attrs: resolveAttrsOptions(options.attrs),
    badges: resolveBadgeOptions(options.badges),
    notByAi: resolveNotByAiOptions(options.notByAi),
    keyboardKeys: resolveKeyboardKeysOptions(options.keyboardKeys),
    abbreviations: resolveAbbreviationsOptions(options.abbreviations),
    definitionLists: resolveDefinitionListOptions(options.definitionLists),
    magicLinks: resolveMagicLinkOptions(options.magicLinks),
    containers: resolveContainerOptions(options.containers),
    images: resolveImageOptions(options.images),
    imageGalleries: resolveImageGalleryOptions(options.imageGalleries),
    timelines: resolveTimelineOptions(options.timelines),
    conditionalBlocks: resolveConditionalBlockOptions(options.conditionalBlocks),
    codeImports: resolveCodeImportOptions(options.codeImports),
    includes: resolveIncludeOptions(options.includes),
    partials: resolvePartialsOptions(options.partials),
    cards: resolveCardOptions(options.cards),
    steps: resolveStepsOptions(options.steps),
    codeGroups: resolveCodeGroupOptions(options.codeGroups),
    fileTree: resolveFileTreeOptions(options.fileTree),
    dataTables: resolveDataTableOptions(options.dataTables),
    sanitize: resolveSanitizeOptions(options.sanitize),
    editThisPage: resolveEditThisPageOptions(options.editThisPage),
    cjkEmphasis: options.cjkEmphasis ?? false,
    codeBlockLint: resolveCodeBlockLintOptions(options.codeBlockLint),
    codeBlockTypecheck: resolveCodeBlockTypecheckOptions(options.codeBlockTypecheck),
    typedHover: resolveTypedHoverOptions(options.typedHover),
    docsTests: resolveDocsTestOptions(options.docsTests),
    mermaid: options.mermaid ?? false,
    graphviz: resolveGraphvizOptions(options.graphviz),
    math: resolveMathOptions(options.math),
    frontmatter: options.frontmatter ?? true,
    toc: options.toc ?? true,
    tocMaxDepth: options.tocMaxDepth ?? 3,
    headingPermalinks: resolveHeadingPermalinksOptions(options.headingPermalinks),
    ogImage: options.ogImage ?? false,
    ogImageOptions: resolveOgImageOptions(options.ogImageOptions),
    transformers: options.transformers ?? [],
    docs: resolveDocsOptions(options.docs),
    search: resolveSearchOptions(options.search),
    collections: resolveCollectionsOptions(options.collections),
    ogViewer: options.ogViewer ?? true,
    embeds: resolveBuiltinEmbedOptions(options.embeds),
    i18n: resolveI18nOptions(options.i18n),
  };
}
function resolveWikiLinkOptions(
  options: OxContentOptions["wikiLinks"],
  baseUrl: string,
): ResolvedOptions["wikiLinks"] {
  if (!options) return { enabled: false, baseUrl };
  if (options === true) return { enabled: true, baseUrl };
  return { enabled: true, baseUrl: options.baseUrl ?? baseUrl };
}
function resolveEmojiShortcodeOptions(
  options: OxContentOptions["emojiShortcodes"],
): ResolvedOptions["emojiShortcodes"] {
  if (!options) return { enabled: false, custom: {} };
  if (options === true) return { enabled: true, custom: {} };
  return { enabled: true, custom: options.custom ?? {} };
}

export function resolveMathOptions(options: OxContentOptions["math"]): ResolvedOptions["math"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: options.enabled ?? true };
}
function resolveAttrsOptions(options: OxContentOptions["attrs"]): ResolvedOptions["attrs"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: options.enabled ?? true };
}

export function resolveBadgeOptions(
  options: OxContentOptions["badges"],
): ResolvedOptions["badges"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: options.enabled ?? true };
}

export function resolveKeyboardKeysOptions(
  options: OxContentOptions["keyboardKeys"],
): NonNullable<ResolvedOptions["keyboardKeys"]> {
  if (!options) return { enabled: false, aliases: {}, style: "words" };
  if (options === true) return { enabled: true, aliases: {}, style: "words" };
  return {
    enabled: options.enabled ?? true,
    aliases: options.aliases ?? {},
    style: options.style === "symbols" ? "symbols" : "words",
  };
}

function resolveContainerOptions(
  options: OxContentOptions["containers"],
): ResolvedOptions["containers"] {
  if (!options) return { enabled: false, types: {} };
  if (options === true) return { enabled: true, types: {} };
  return { enabled: options.enabled ?? true, types: options.types ?? {} };
}

function resolveCodeImportOptions(
  options: OxContentOptions["codeImports"],
): ResolvedOptions["codeImports"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return { enabled: true, rootDir: options.rootDir };
}

function resolveSanitizeOptions(
  options: OxContentOptions["sanitize"],
): ResolvedOptions["sanitize"] {
  if (!options) return { enabled: false };
  if (options === true) return { enabled: true };
  return {
    enabled: true,
    allowedTags: options.allowedTags,
    allowedAttributes: options.allowedAttributes,
    allowedUrlSchemes: options.allowedUrlSchemes,
  };
}

function resolveEditThisPageOptions(
  options: OxContentOptions["editThisPage"],
): ResolvedOptions["editThisPage"] {
  if (!options) return { enabled: false, branch: "main", label: "Edit this page" };
  if (options === true) return { enabled: false, branch: "main", label: "Edit this page" };
  return {
    enabled: Boolean(options.repoUrl),
    repoUrl: options.repoUrl,
    branch: options.branch ?? "main",
    rootDir: options.rootDir,
    label: options.label ?? "Edit this page",
  };
}

function resolveCodeBlockLintOptions(
  options: OxContentOptions["codeBlockLint"],
): ResolvedOptions["codeBlockLint"] {
  if (!options) {
    return { enabled: false, requireLanguage: false, trailingSpaces: true, mode: "warn" };
  }
  if (options === true) {
    return { enabled: true, requireLanguage: false, trailingSpaces: true, mode: "warn" };
  }
  return {
    enabled: true,
    languages: options.languages,
    requireLanguage: options.requireLanguage ?? false,
    trailingSpaces: options.trailingSpaces ?? true,
    mode: options.mode ?? "warn",
  };
}

function resolveCodeBlockTypecheckOptions(
  options: OxContentOptions["codeBlockTypecheck"],
): ResolvedOptions["codeBlockTypecheck"] {
  if (!options) {
    return {
      enabled: false,
      languages: ["ts", "tsx"],
      requireMeta: true,
      tsgoCommand: "tsgo",
      mode: "warn",
    };
  }
  if (options === true) {
    return {
      enabled: true,
      languages: ["ts", "tsx"],
      requireMeta: true,
      tsgoCommand: "tsgo",
      mode: "warn",
    };
  }
  return {
    enabled: true,
    languages: options.languages ?? ["ts", "tsx"],
    requireMeta: options.requireMeta ?? true,
    tsgoCommand: options.tsgoCommand ?? "tsgo",
    mode: options.mode ?? "warn",
  };
}

function resolveDocsTestOptions(
  options: OxContentOptions["docsTests"],
): ResolvedOptions["docsTests"] {
  if (!options) return { enabled: false, languages: ["js", "jsx", "ts", "tsx"], requireMeta: true };
  if (options === true) {
    return { enabled: true, languages: ["js", "jsx", "ts", "tsx"], requireMeta: true };
  }
  return {
    enabled: true,
    languages: options.languages ?? ["js", "jsx", "ts", "tsx"],
    requireMeta: options.requireMeta ?? true,
  };
}

function resolveCodeAnnotationsOptions(
  options: OxContentOptions["codeAnnotations"],
): ResolvedOptions["codeAnnotations"] {
  if (!options) {
    return {
      enabled: false,
      notation: "attribute",
      metaKey: "annotate",
      defaultLineNumbers: false,
    };
  }
  if (options === true) {
    return {
      enabled: true,
      notation: "attribute",
      metaKey: "annotate",
      defaultLineNumbers: false,
    };
  }
  return {
    enabled: true,
    notation: options.notation ?? "attribute",
    metaKey: options.metaKey ?? "annotate",
    defaultLineNumbers: options.defaultLineNumbers ?? false,
  };
}
