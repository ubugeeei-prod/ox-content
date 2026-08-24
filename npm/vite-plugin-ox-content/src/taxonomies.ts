/**
 * Opt-in taxonomy term pages and related-page lists.
 *
 * Resolution and HTML live here. The Vite plugin injects related markup into
 * page content, then writes themed list and per-term pages during SSG.
 */

import {
  containedPath,
  displayTaxonomyName,
  listPageContent,
  relatedMarkup,
  siteHref,
  termPageContent,
  type TaxonomySourcePage,
  type TermBucket,
} from "./taxonomies-html";
import type { ResolvedTaxonomiesOptions, TaxonomiesOptions } from "./types";

export type { TaxonomySourcePage } from "./taxonomies-html";

const DEFAULT_TAXONOMIES = ["tags", "categories"];
const DEFAULT_RELATED_LIMIT = 5;
const HOSTILE_TERM = /^(?:javascript|data):/i;

/** Synthetic page passed back to `generateHtmlPage`. */
export interface TaxonomyGeneratedPage {
  title: string;
  content: string;
  outputPath: string;
  urlPath: string;
  href: string;
}

/**
 * Resolves `taxonomies` with defaults.
 *
 * `false` / omitted stays off. `true` enables `tags` and `categories` with
 * relatedLimit 5. An object enables the feature and overrides only set fields.
 */
export function resolveTaxonomiesOptions(
  value: boolean | TaxonomiesOptions | undefined,
): ResolvedTaxonomiesOptions {
  if (!value) {
    return {
      enabled: false,
      taxonomies: [...DEFAULT_TAXONOMIES],
      relatedLimit: DEFAULT_RELATED_LIMIT,
    };
  }
  if (value === true) {
    return {
      enabled: true,
      taxonomies: [...DEFAULT_TAXONOMIES],
      relatedLimit: DEFAULT_RELATED_LIMIT,
    };
  }
  return {
    enabled: true,
    taxonomies: normalizeTaxonomyNames(value.taxonomies),
    relatedLimit: normalizeRelatedLimit(value.relatedLimit),
  };
}

/**
 * Stable URL slug for a frontmatter term.
 *
 * Returns `undefined` when the value cannot become a safe `[a-z0-9-]` href.
 */
export function termSlug(term: string): string | undefined {
  const trimmed = term.trim();
  if (!trimmed || HOSTILE_TERM.test(trimmed) || trimmed.includes("..") || trimmed.includes("//")) {
    return undefined;
  }
  const slug = trimmed
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");
  return slug || undefined;
}

/** Appends related-page HTML to source pages that share a listed term. */
export function injectRelatedPages(
  pages: TaxonomySourcePage[],
  listed: readonly TaxonomySourcePage[],
  options?: ResolvedTaxonomiesOptions,
): void {
  if (!options?.enabled) {
    return;
  }
  const listedKeys = listed.map((page) => pageTermKeys(page, options.taxonomies));
  for (const page of pages) {
    const keys = pageTermKeys(page, options.taxonomies);
    if (keys.size === 0) {
      continue;
    }
    const related = listed
      .map((candidate, index) => ({
        page: candidate,
        score: samePage(page, candidate) ? 0 : sharedCount(keys, listedKeys[index] ?? new Set()),
      }))
      .filter((entry) => entry.score > 0)
      .sort((left, right) => {
        if (left.score !== right.score) {
          return right.score - left.score;
        }
        const titleCmp = left.page.title.localeCompare(right.page.title);
        return titleCmp !== 0
          ? titleCmp
          : left.page.routePaths.href.localeCompare(right.page.routePaths.href);
      })
      .slice(0, options.relatedLimit)
      .map((entry) => entry.page);
    if (related.length === 0) {
      continue;
    }
    page.transformedHtml += relatedMarkup(related);
  }
}

/** Maps a generated taxonomy page onto the SSG render shape. */
export function toTaxonomyProcessResult(page: TaxonomyGeneratedPage): {
  inputPath: string;
  routePaths: {
    outputPath: string;
    urlPath: string;
    href: string;
    ogImagePath: string;
    ogImageUrl: string;
  };
  transformedHtml: string;
  title: string;
  frontmatter: Record<string, unknown>;
  toc: [];
} {
  return {
    inputPath: page.outputPath,
    routePaths: {
      outputPath: page.outputPath,
      urlPath: page.urlPath,
      href: page.href,
      ogImagePath: "",
      ogImageUrl: "",
    },
    transformedHtml: page.content,
    title: page.title,
    frontmatter: {},
    toc: [],
  };
}

/** Renders themed list and per-term pages and appends them to the build. */
export async function appendTaxonomyPages(input: {
  generatedPages: Array<{ inputPath: string; outputPath: string; html: string }>;
  listedPages: readonly TaxonomySourcePage[];
  options?: ResolvedTaxonomiesOptions;
  outDir: string;
  base: string;
  render: (page: TaxonomyGeneratedPage) => Promise<string>;
  errors: string[];
}): Promise<void> {
  if (!input.options?.enabled) {
    return;
  }
  for (const spec of taxonomyPageSpecs(
    input.listedPages,
    input.options,
    input.outDir,
    input.base,
  )) {
    try {
      input.generatedPages.push({
        inputPath: spec.outputPath,
        outputPath: spec.outputPath,
        html: await input.render(spec),
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      input.errors.push(`Failed to generate taxonomy page ${spec.href}: ${message}`);
    }
  }
}

function taxonomyPageSpecs(
  listed: readonly TaxonomySourcePage[],
  options: ResolvedTaxonomiesOptions,
  outDir: string,
  base: string,
): TaxonomyGeneratedPage[] {
  const pages: TaxonomyGeneratedPage[] = [];
  for (const taxonomy of options.taxonomies) {
    const urlName = taxonomy.toLowerCase();
    const terms = collectTerms(listed, taxonomy);
    const listHref = siteHref(base, urlName);
    const listOutput = containedPath(outDir, urlName, "index.html");
    if (listOutput) {
      pages.push({
        title: displayTaxonomyName(urlName),
        content: listPageContent(terms, base, urlName),
        outputPath: listOutput,
        urlPath: urlName,
        href: listHref,
      });
    }
    for (const term of terms) {
      const outputPath = containedPath(outDir, urlName, term.slug, "index.html");
      if (!outputPath) {
        continue;
      }
      pages.push({
        title: term.label,
        content: termPageContent(term),
        outputPath,
        urlPath: `${urlName}/${term.slug}`,
        href: siteHref(base, urlName, term.slug),
      });
    }
  }
  return pages;
}

function collectTerms(listed: readonly TaxonomySourcePage[], taxonomy: string): TermBucket[] {
  const buckets = new Map<string, TermBucket>();
  for (const page of listed) {
    for (const label of termsFromValue(page.frontmatter[taxonomy])) {
      const slug = termSlug(label);
      if (!slug) {
        continue;
      }
      const existing = buckets.get(slug);
      if (existing) {
        existing.pages.push(page);
      } else {
        buckets.set(slug, { label, slug, pages: [page] });
      }
    }
  }
  return [...buckets.values()].sort((left, right) => left.label.localeCompare(right.label));
}

function pageTermKeys(page: TaxonomySourcePage, taxonomies: readonly string[]): Set<string> {
  const keys = new Set<string>();
  for (const taxonomy of taxonomies) {
    for (const label of termsFromValue(page.frontmatter[taxonomy])) {
      const slug = termSlug(label);
      if (slug) {
        keys.add(`${taxonomy.toLowerCase()}\0${slug}`);
      }
    }
  }
  return keys;
}

function termsFromValue(value: unknown): string[] {
  if (typeof value === "string") {
    return value.trim() ? [value.trim()] : [];
  }
  if (!Array.isArray(value)) {
    return [];
  }
  return value.flatMap((item) => (typeof item === "string" && item.trim() ? [item.trim()] : []));
}

function normalizeTaxonomyNames(names: string[] | undefined): string[] {
  if (!names) {
    return [...DEFAULT_TAXONOMIES];
  }
  const seen = new Set<string>();
  const resolved: string[] = [];
  for (const name of names) {
    if (typeof name !== "string") {
      continue;
    }
    const trimmed = name.trim();
    if (!/^[A-Za-z][A-Za-z0-9_-]*$/.test(trimmed)) {
      continue;
    }
    const url = trimmed.toLowerCase();
    if (seen.has(url)) {
      continue;
    }
    seen.add(url);
    resolved.push(trimmed);
  }
  return resolved;
}

function normalizeRelatedLimit(value: number | undefined): number {
  if (typeof value === "number" && Number.isFinite(value) && value >= 0) {
    return Math.floor(value);
  }
  return DEFAULT_RELATED_LIMIT;
}

function samePage(left: TaxonomySourcePage, right: TaxonomySourcePage): boolean {
  if (left.inputPath && right.inputPath) {
    return left.inputPath === right.inputPath;
  }
  return left.routePaths.href === right.routePaths.href;
}

function sharedCount(left: Set<string>, right: Set<string>): number {
  let count = 0;
  for (const key of left) {
    if (right.has(key)) {
      count += 1;
    }
  }
  return count;
}
