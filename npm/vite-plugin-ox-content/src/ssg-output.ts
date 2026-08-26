/**
 * Composable SSG outputs for custom hosts that set `ssg: false`.
 *
 * Ox Content owns resource fingerprinting, Markdown companions, feeds,
 * sitemaps, and git lastmod. The host keeps page templates and HTML.
 */

import { resolveFeedsOptions, writeFeedFiles, type FeedItemInput } from "./feeds";
import { shouldPublishMarkdownSource, type MarkdownSourcePageInput } from "./markdown-source";
import { resolvePublishStateOptions } from "./publish-state";
import { resolveResourcesOptions } from "./resources";
import { resolveSiteMapsOptions, writeSiteMapFiles, type SiteMapPageInput } from "./site-maps";
import { resolveSsgOptions } from "./ssg";
import {
  resolveGitLastmod,
  writeMarkdownCompanions,
  writeResourceFiles,
  type WriteResourceFilesInput,
  type WriteResourceFilesPage,
} from "./ssg-output-write";
import type {
  FeedsOptions,
  OxContentOptions,
  PublishStateOptions,
  ResolvedFeedsOptions,
  ResolvedMarkdownSourceOptions,
  ResolvedPublishStateOptions,
  ResolvedResourcesOptions,
  ResolvedSiteMapsOptions,
  ResourcesOptions,
  SiteMapsOptions,
  SsgOptions,
  SsgOutputPageInput,
} from "./types";

export {
  resolveGitLastmod,
  writeFeedFiles,
  writeMarkdownCompanions,
  writeResourceFiles,
  writeSiteMapFiles,
};
export type {
  FeedItemInput,
  WriteFeedFilesInput,
  WriteMarkdownSourceFilesInput,
  WriteResourceFilesInput,
  WriteResourceFilesPage,
  WriteResourceFilesResult,
  WriteSiteMapFilesInput,
} from "./ssg-output-write";
export type { SiteMapPageInput } from "./site-maps";
export type { SsgOutputPageInput } from "./types";

/** Same option objects `oxContent()` / `buildSsg()` accept. `ssg.enabled` is ignored. */
export interface PlanSsgOutputsOptions {
  base?: string;
  resources?: boolean | ResourcesOptions;
  feeds?: boolean | FeedsOptions;
  siteMaps?: boolean | SiteMapsOptions;
  publishState?: boolean | PublishStateOptions;
  ssg?: boolean | SsgOptions;
}

/** Inputs for planning composable SSG outputs from host-rendered pages. */
export interface PlanSsgOutputsInput {
  pages: readonly SsgOutputPageInput[];
  outDir: string;
  srcDir?: string;
  root?: string;
  siteDescription?: string;
  collections?: Record<string, readonly FeedItemInput[]>;
  collectionNames?: readonly string[];
  items?: readonly FeedItemInput[];
  options?: PlanSsgOutputsOptions | Pick<OxContentOptions, keyof PlanSsgOutputsOptions>;
}

/** Planned writer inputs. Call the matching `write*` function for each feature. */
export interface SsgOutputPlan {
  resources: WriteResourceFilesInput;
  markdownCompanions: {
    outDir: string;
    base: string;
    options: ResolvedMarkdownSourceOptions;
    publishState: ResolvedPublishStateOptions;
    pages: MarkdownSourcePageInput[];
  };
  feeds: {
    outDir: string;
    base: string;
    siteUrl?: string;
    siteName?: string;
    siteDescription?: string;
    options: ResolvedFeedsOptions;
    publishState: ResolvedPublishStateOptions;
    collections?: Record<string, readonly FeedItemInput[]>;
    collectionNames?: readonly string[];
    items?: readonly FeedItemInput[];
  };
  siteMaps: {
    outDir: string;
    base: string;
    siteUrl?: string;
    siteName?: string;
    siteDescription?: string;
    options: ResolvedSiteMapsOptions;
    pages: SiteMapPageInput[];
  };
}

/**
 * Plan resource, companion, feed, and sitemap outputs without rendering pages.
 *
 * `ssg.enabled` is ignored. Use `ssg: { enabled: false, markdownSource, lastUpdated, siteUrl }`
 * so those fields still resolve. `lastUpdated` on a page wins over git.
 */
export function planSsgOutputs(input: PlanSsgOutputsInput): SsgOutputPlan {
  const raw = input.options ?? {};
  const ssg = resolveSsgOptions(raw.ssg);
  const resources = resolveResourcesOptions(raw.resources);
  const feeds = resolveFeedsOptions(raw.feeds);
  const siteMaps = resolveSiteMapsOptions(raw.siteMaps);
  const publishState = resolvePublishStateOptions(raw.publishState);
  const markdownSource = ssg.markdownSource ?? { enabled: false, alternate: true, copy: false };
  const base = normalizeBase(raw.base);
  const lastmod = ssg.lastUpdated || siteMaps.enabled;

  return {
    resources: {
      pages: resourcePages(input.pages, resources),
      srcDir: input.srcDir ?? "",
      outDir: input.outDir,
      root: input.root,
      base,
      options: resources,
    },
    markdownCompanions: {
      outDir: input.outDir,
      base,
      options: markdownSource,
      publishState,
      pages: companionPages(input.pages, markdownSource, publishState),
    },
    feeds: {
      outDir: input.outDir,
      base,
      siteUrl: ssg.siteUrl,
      siteName: ssg.siteName,
      siteDescription: input.siteDescription,
      options: feeds,
      publishState,
      collections: input.collections,
      collectionNames: input.collectionNames,
      items: input.items ?? (input.collections ? undefined : feedItems(input.pages)),
    },
    siteMaps: {
      outDir: input.outDir,
      base,
      siteUrl: ssg.siteUrl,
      siteName: ssg.siteName,
      siteDescription: input.siteDescription,
      options: siteMaps,
      pages: sitemapPages(input, ssg.siteUrl, base, lastmod),
    },
  };
}

function resourcePages(
  pages: readonly SsgOutputPageInput[],
  resources: ResolvedResourcesOptions,
): WriteResourceFilesPage[] {
  if (!resources.enabled) {
    return [];
  }
  const planned: WriteResourceFilesPage[] = [];
  for (const page of pages) {
    if (page.html == null || !page.outputPath) {
      continue;
    }
    planned.push({ html: page.html, inputPath: page.inputPath, outputPath: page.outputPath });
  }
  return planned;
}

function companionPages(
  pages: readonly SsgOutputPageInput[],
  options: ResolvedMarkdownSourceOptions,
  publishState: ResolvedPublishStateOptions,
): MarkdownSourcePageInput[] {
  if (!options.enabled) {
    return [];
  }
  const planned: MarkdownSourcePageInput[] = [];
  for (const page of pages) {
    const frontmatter = pageFrontmatter(page);
    if (page.source == null || !shouldPublishMarkdownSource(frontmatter, publishState)) {
      continue;
    }
    planned.push({
      inputPath: page.inputPath,
      source: page.source,
      urlPath: page.urlPath,
      frontmatter,
    });
  }
  return planned;
}

function feedItems(pages: readonly SsgOutputPageInput[]): FeedItemInput[] {
  return pages.map((page) => ({
    title: page.title,
    description: page.description,
    path: page.urlPath,
    loc: page.loc,
    lastUpdated: page.lastUpdated,
    draft: page.draft ?? page.frontmatter?.draft,
    unlisted: page.unlisted ?? page.frontmatter?.unlisted,
    frontmatter: pageFrontmatter(page),
  }));
}

function pageFrontmatter(page: SsgOutputPageInput): Record<string, unknown> {
  return {
    ...page.frontmatter,
    ...(page.draft === true ? { draft: true } : {}),
    ...(page.unlisted === true ? { unlisted: true } : {}),
  };
}

function sitemapPages(
  input: PlanSsgOutputsInput,
  siteUrl: string | undefined,
  base: string,
  resolveLastmod: boolean,
): SiteMapPageInput[] {
  return input.pages.map((page) => ({
    loc: page.loc || canonicalPageLoc(siteUrl, base, page.urlPath),
    title: page.title ?? "",
    description: page.description,
    lastUpdated: pageLastUpdated(page, input.root, resolveLastmod),
    draft: page.draft === true || page.frontmatter?.draft === true,
    unlisted: page.unlisted === true || page.frontmatter?.unlisted === true,
  }));
}

function pageLastUpdated(
  page: SsgOutputPageInput,
  root: string | undefined,
  resolveLastmod: boolean,
): number | undefined {
  if (page.lastUpdated != null) {
    return page.lastUpdated;
  }
  if (!resolveLastmod || !page.inputPath) {
    return undefined;
  }
  return resolveGitLastmod(page.inputPath, root);
}

function canonicalPageLoc(siteUrl: string | undefined, base: string, urlPath: string): string {
  const origin = (siteUrl ?? "").trim().replace(/\/+$/, "");
  if (!origin) {
    return "";
  }
  const prefix = base;
  if (!urlPath || urlPath === "/") {
    return `${origin}${prefix}`;
  }
  return `${origin}${prefix}${urlPath.replace(/^\/+|\/+$/gu, "")}/`;
}

function normalizeBase(base: string | undefined): string {
  if (!base || base === "/") {
    return "/";
  }
  return base.endsWith("/") ? base : `${base}/`;
}
