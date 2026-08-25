/**
 * Generated blog index, tag, and archive pages.
 */

import * as fs from "node:fs/promises";
import {
  archiveIndexContent,
  archiveMonthContent,
  archiveYearContent,
  containedPath,
  indexPageContent,
  postMetaMarkup,
  siteHref,
  tagPageContent,
  type BlogSourcePage,
} from "./blog-html";
import { readingTimeMinutes } from "./blog-reading";
import {
  collectTags,
  datedPosts,
  isAmbiguousCollection,
  postTagLinks,
  resolvePostAuthors,
  selectBlogPosts,
  sortPosts,
  toListItem,
  uniqueMonths,
  uniqueYears,
} from "./blog-posts";
import type { ResolvedBlogOptions, ResolvedCollectionsOptions } from "./types";

const AMBIGUOUS_COLLECTION =
  "[ox-content] blog is enabled but multiple collections are configured; set blog.collection";

/** Synthetic page passed back to `generateHtmlPage`. */
export interface BlogGeneratedPage {
  title: string;
  content: string;
  outputPath: string;
  urlPath: string;
  href: string;
}

export async function injectBlogPostMeta(input: {
  pages: BlogSourcePage[];
  listed: readonly BlogSourcePage[];
  options?: ResolvedBlogOptions;
  srcDir: string;
  collections?: ResolvedCollectionsOptions;
  base: string;
}): Promise<void> {
  if (!input.options?.enabled) {
    return;
  }
  const posts = selectBlogPosts(input.listed, input.options, input.srcDir, input.collections);
  if (posts === undefined) {
    return;
  }
  const listedPaths = new Set(posts.map((page) => page.inputPath));
  for (const page of input.pages) {
    if (!listedPaths.has(page.inputPath)) {
      continue;
    }
    const markdown = await readMarkdown(page.inputPath);
    page.transformedHtml =
      postMetaMarkup({
        authors: resolvePostAuthors(page.frontmatter, input.options.authors),
        minutes: readingTimeMinutes(markdown),
        tags: postTagLinks(page.frontmatter, input.base),
      }) + page.transformedHtml;
  }
}

/** Maps a generated blog page onto the SSG render shape. */
export function toBlogProcessResult(page: BlogGeneratedPage): {
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

/** Renders index, tag, and archive pages and appends them to the build. */
export async function appendBlogPages(input: {
  generatedPages: Array<{ inputPath: string; outputPath: string; html: string }>;
  listedPages: readonly BlogSourcePage[];
  options?: ResolvedBlogOptions;
  collections?: ResolvedCollectionsOptions;
  srcDir: string;
  outDir: string;
  base: string;
  render: (page: BlogGeneratedPage) => Promise<string>;
  errors: string[];
}): Promise<void> {
  if (!input.options?.enabled) {
    return;
  }
  if (isAmbiguousCollection(input.options, input.collections)) {
    input.errors.push(AMBIGUOUS_COLLECTION);
    return;
  }
  const posts = selectBlogPosts(input.listedPages, input.options, input.srcDir, input.collections);
  if (posts === undefined) {
    return;
  }
  for (const spec of blogPageSpecs(posts, input.options, input.outDir, input.base)) {
    try {
      input.generatedPages.push({
        inputPath: spec.outputPath,
        outputPath: spec.outputPath,
        html: await input.render(spec),
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      input.errors.push(`Failed to generate blog page ${spec.href}: ${message}`);
    }
  }
}

function blogPageSpecs(
  posts: readonly BlogSourcePage[],
  options: ResolvedBlogOptions,
  outDir: string,
  base: string,
): BlogGeneratedPage[] {
  const sorted = sortPosts(posts);
  const pages: BlogGeneratedPage[] = [];
  const pageSize = options.pageSize;
  const pageCount = Math.max(1, Math.ceil(sorted.length / pageSize) || 1);
  const totalPages = sorted.length === 0 ? 1 : pageCount;

  for (let pageNumber = 1; pageNumber <= totalPages; pageNumber += 1) {
    const slice = sorted.slice((pageNumber - 1) * pageSize, pageNumber * pageSize);
    const isFirst = pageNumber === 1;
    const urlPath = isFirst ? "blog" : `blog/page/${pageNumber}`;
    const outputPath = isFirst
      ? containedPath(outDir, "blog", "index.html")
      : containedPath(outDir, "blog", "page", String(pageNumber), "index.html");
    if (!outputPath) {
      continue;
    }
    pages.push({
      title: isFirst ? "Blog" : `Blog · page ${pageNumber}`,
      content: indexPageContent(slice.map(toListItem), {
        newerHref: isFirst
          ? undefined
          : siteHref(
              base,
              ...(pageNumber === 2 ? ["blog"] : ["blog", "page", String(pageNumber - 1)]),
            ),
        olderHref:
          pageNumber < totalPages
            ? siteHref(base, "blog", "page", String(pageNumber + 1))
            : undefined,
      }),
      outputPath,
      urlPath,
      href: siteHref(base, ...urlPath.split("/")),
    });
  }

  const tags = collectTags(sorted);
  for (const tag of tags) {
    const outputPath = containedPath(outDir, "blog", "tags", tag.slug, "index.html");
    if (!outputPath) {
      continue;
    }
    pages.push({
      title: tag.label,
      content: tagPageContent(tag.label, tag.pages.map(toListItem)),
      outputPath,
      urlPath: `blog/tags/${tag.slug}`,
      href: siteHref(base, "blog", "tags", tag.slug),
    });
  }

  const dated = datedPosts(sorted);
  if (dated.length > 0) {
    const years = uniqueYears(dated);
    const archiveIndex = containedPath(outDir, "blog", "archive", "index.html");
    if (archiveIndex) {
      pages.push({
        title: "Archive",
        content: archiveIndexContent(
          years.map((year) => ({ year, href: siteHref(base, "blog", "archive", year) })),
        ),
        outputPath: archiveIndex,
        urlPath: "blog/archive",
        href: siteHref(base, "blog", "archive"),
      });
    }
    for (const year of years) {
      const yearPosts = dated.filter((entry) => entry.year === year);
      const months = uniqueMonths(yearPosts);
      const yearPath = containedPath(outDir, "blog", "archive", year, "index.html");
      if (yearPath) {
        pages.push({
          title: year,
          content: archiveYearContent(
            year,
            months.map((month) => ({
              month: `${year}-${month}`,
              href: siteHref(base, "blog", "archive", year, month),
            })),
            yearPosts.map((entry) => toListItem(entry.page)),
          ),
          outputPath: yearPath,
          urlPath: `blog/archive/${year}`,
          href: siteHref(base, "blog", "archive", year),
        });
      }
      for (const month of months) {
        const monthPosts = yearPosts.filter((entry) => entry.month === month);
        const monthPath = containedPath(outDir, "blog", "archive", year, month, "index.html");
        if (!monthPath) {
          continue;
        }
        pages.push({
          title: `${year}-${month}`,
          content: archiveMonthContent(
            `${year}-${month}`,
            monthPosts.map((entry) => toListItem(entry.page)),
          ),
          outputPath: monthPath,
          urlPath: `blog/archive/${year}/${month}`,
          href: siteHref(base, "blog", "archive", year, month),
        });
      }
    }
  }

  return pages;
}

async function readMarkdown(inputPath: string): Promise<string> {
  try {
    return await fs.readFile(inputPath, "utf8");
  } catch {
    return "";
  }
}
