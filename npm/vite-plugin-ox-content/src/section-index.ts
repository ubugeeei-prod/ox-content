/**
 * Opt-in generated section index pages.
 *
 * Resolution and directory walking live here. Listing HTML is rendered in
 * Rust (`ox_content_ssg::render_section_index`) when the NAPI helper is
 * available; a matching TypeScript renderer covers the same escape / href
 * rules so the SSG path stays safe either way. The Vite plugin appends
 * themed HTML during SSG and never overwrites an existing index page.
 */

import * as path from "node:path";
import {
  isSafeSectionHref,
  renderSectionIndexHtml,
  type SectionIndexItem,
} from "./section-index-html";
import {
  dirFromOutputPath,
  firstChildDir,
  normalizeUrlPath,
  pageTitle,
  parentDir,
  sectionHref,
  sectionOutputPath,
  sectionTitle,
} from "./section-index-paths";
import type { ResolvedSectionIndexOptions, SectionIndexOptions } from "./types";

export {
  escapeSectionIndexHtml,
  isSafeSectionHref,
  renderSectionIndexHtml,
  type SectionIndexItem,
} from "./section-index-html";

/** One built page considered when deciding indexes and children. */
export interface SectionIndexSourcePage {
  title: string;
  description?: string;
  frontmatter: Record<string, unknown>;
  inputPath?: string;
  routePaths: {
    href: string;
    urlPath: string;
    outputPath?: string;
  };
}

/** Synthetic page passed back to `generateHtmlPage`. */
export interface SectionIndexGeneratedPage {
  title: string;
  content: string;
  outputPath: string;
  urlPath: string;
  href: string;
}

/**
 * Resolves `ssg.sectionIndex` with defaults.
 *
 * `false` / omitted stays off. `true` enables card listings. An object
 * enables the feature and overrides only the fields the site set.
 */
export function resolveSectionIndexOptions(
  value: boolean | SectionIndexOptions | undefined,
): ResolvedSectionIndexOptions {
  if (!value) {
    return { enabled: false, style: "cards" };
  }
  if (value === true) {
    return { enabled: true, style: "cards" };
  }
  return {
    enabled: true,
    style: value.style === "list" ? "list" : "cards",
  };
}

/** Maps a generated section index onto the SSG render shape. */
export function toSectionIndexProcessResult(page: SectionIndexGeneratedPage): {
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

/** Appends generated section indexes for directories that have no real index. */
export async function appendSectionIndexPages(input: {
  generatedPages: Array<{ inputPath: string; outputPath: string; html: string }>;
  collectedPages: readonly SectionIndexSourcePage[];
  listedPages: readonly SectionIndexSourcePage[];
  options?: ResolvedSectionIndexOptions;
  outDir: string;
  base: string;
  extension: string;
  errors: string[];
  render: (page: SectionIndexGeneratedPage) => Promise<string>;
}): Promise<void> {
  if (!input.options?.enabled) {
    return;
  }

  const existingOutputs = new Set(
    input.generatedPages.map((page) => path.normalize(page.outputPath)),
  );
  for (const spec of sectionIndexSpecs(
    input.collectedPages,
    input.listedPages,
    input.options,
    input.outDir,
    input.base,
    input.extension,
  )) {
    if (existingOutputs.has(path.normalize(spec.outputPath))) {
      continue;
    }
    try {
      const html = await input.render(spec);
      input.generatedPages.push({
        inputPath: spec.outputPath,
        outputPath: spec.outputPath,
        html,
      });
      existingOutputs.add(path.normalize(spec.outputPath));
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      input.errors.push(`Failed to generate section index ${spec.href}: ${message}`);
    }
  }
}

function sectionIndexSpecs(
  collected: readonly SectionIndexSourcePage[],
  listed: readonly SectionIndexSourcePage[],
  options: ResolvedSectionIndexOptions,
  outDir: string,
  base: string,
  extension: string,
): SectionIndexGeneratedPage[] {
  const occupied = new Set<string>();
  for (const page of collected) {
    occupied.add(normalizeUrlPath(page.routePaths.urlPath));
  }
  for (const page of collected) {
    const output = page.routePaths.outputPath;
    if (output) {
      occupied.add(dirFromOutputPath(output, outDir));
    }
  }

  const visible = listed.filter((page) => !isHiddenByFlags(page.frontmatter));
  const childrenByDir = new Map<string, SectionIndexItem[]>();

  for (const page of visible) {
    const urlPath = normalizeUrlPath(page.routePaths.urlPath);
    const parent = parentDir(urlPath);
    if (parent === null) {
      continue;
    }
    pushChild(childrenByDir, parent, {
      title: pageTitle(page),
      href: page.routePaths.href,
      description: page.description,
    });

    let ancestor = parent;
    while (ancestor !== "") {
      const grand = parentDir(ancestor);
      if (grand === null) {
        break;
      }
      const nested = firstChildDir(urlPath, grand);
      if (nested) {
        pushUniqueDir(childrenByDir, grand, nested, visible, base, extension);
      }
      ancestor = grand;
    }
  }

  const pages: SectionIndexGeneratedPage[] = [];
  const dirs = [...childrenByDir.keys()].sort();
  for (const dir of dirs) {
    if (occupied.has(dir)) {
      continue;
    }
    const children = uniqueItems(childrenByDir.get(dir) ?? []).filter((item) =>
      isSafeSectionHref(item.href),
    );
    if (children.length === 0) {
      continue;
    }
    children.sort((left, right) => {
      const titleCmp = left.title.localeCompare(right.title);
      return titleCmp !== 0 ? titleCmp : left.href.localeCompare(right.href);
    });
    const outputPath = sectionOutputPath(outDir, dir, extension);
    if (!outputPath) {
      continue;
    }
    const title = sectionTitle(dir);
    pages.push({
      title,
      content: renderSectionIndexHtml(title, children, options.style),
      outputPath,
      urlPath: dir || "/",
      href: sectionHref(base, dir, extension),
    });
  }
  return pages;
}

function pushChild(
  map: Map<string, SectionIndexItem[]>,
  dir: string,
  item: SectionIndexItem,
): void {
  const list = map.get(dir);
  if (list) {
    list.push(item);
    return;
  }
  map.set(dir, [item]);
}

function pushUniqueDir(
  map: Map<string, SectionIndexItem[]>,
  parent: string,
  childDir: string,
  visible: readonly SectionIndexSourcePage[],
  base: string,
  extension: string,
): void {
  const href = sectionHref(base, childDir, extension);
  const existing = map.get(parent);
  if (existing?.some((item) => item.href === href)) {
    return;
  }
  const indexPage = visible.find((page) => normalizeUrlPath(page.routePaths.urlPath) === childDir);
  pushChild(map, parent, {
    title: indexPage ? pageTitle(indexPage) : sectionTitle(childDir),
    href: indexPage?.routePaths.href ?? href,
    description: indexPage?.description,
  });
}

function uniqueItems(items: SectionIndexItem[]): SectionIndexItem[] {
  const seen = new Set<string>();
  const unique: SectionIndexItem[] = [];
  for (const item of items) {
    if (seen.has(item.href)) {
      continue;
    }
    seen.add(item.href);
    unique.push(item);
  }
  return unique;
}

function isHiddenByFlags(frontmatter: Record<string, unknown>): boolean {
  return frontmatter.draft === true || frontmatter.unlisted === true;
}
