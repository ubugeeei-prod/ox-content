import * as fs from "node:fs/promises";
import * as path from "node:path";
import { glob } from "glob";
import type { ResolvedSlidesPluginOptions, SlideDeckData, SlideFileEntry } from "./internal-types";
import { loadNapiModule } from "./napi";
import {
  formatTitle,
  getDeckHref,
  getDeckOutputPath,
  getDeckPdfOutputPath,
  getDeckPrintOutputPath,
  getPresenterHref,
  getPresenterOutputPath,
  getSlideHref,
  getSlideOutputPath,
  normalizeExtension,
  normalizeRouteSegment,
} from "./path-utils";
import {
  type SlideRenderFragment,
  renderCustomSlide,
  renderHtmlSlide,
  renderMarkdownSlide,
} from "./render-content";

async function collectSlideFiles(
  options: ResolvedSlidesPluginOptions,
  root: string,
): Promise<SlideFileEntry[]> {
  const extensions = [
    ...new Set([...options.extensions, ...Object.keys(options.renderers).map(normalizeExtension)]),
  ];
  const slidesDir = path.resolve(root, options.srcDir);
  const pattern = path.join(
    slidesDir,
    `**/*.{${extensions.map((ext) => ext.replace(/^\./, "")).join(",")}}`,
  );
  const files = await glob(pattern, {
    nodir: true,
    ignore: ["**/node_modules/**", "**/dist/**", "**/.git/**"],
  });

  return files.sort().map((filePath) => ({ filePath }));
}

function classifyDecks(
  entries: SlideFileEntry[],
  slidesDir: string,
): Map<string, SlideFileEntry[]> {
  const decks = new Map<string, SlideFileEntry[]>();
  const numericName = /^\d+$/;

  for (const entry of entries) {
    const ext = path.extname(entry.filePath);
    const basename = path.basename(entry.filePath, ext);
    const relativeDir = normalizeRouteSegment(
      path.relative(slidesDir, path.dirname(entry.filePath)),
    );

    if (numericName.test(basename)) {
      decks.set(relativeDir, [...(decks.get(relativeDir) ?? []), entry]);
      continue;
    }

    const slug = normalizeRouteSegment(
      path.join(relativeDir, basename === "index" ? "" : basename),
    );
    decks.set(slug, [entry]);
  }

  return decks;
}

async function renderDeckMarkdownFile(
  filePath: string,
  separator: string,
  markdownOptions: ResolvedSlidesPluginOptions["markdown"],
): Promise<{ deckFrontmatter: Record<string, unknown>; renderedSlides: SlideRenderFragment[] }> {
  const napi = await loadNapiModule();
  const source = await fs.readFile(filePath, "utf-8");
  const parsed = napi.parseMarkdownSlideDeck(source, separator);
  const deckFrontmatter = JSON.parse(parsed.frontmatter || "{}") as Record<string, unknown>;

  return {
    deckFrontmatter,
    renderedSlides: await Promise.all(
      parsed.slides.map((slide, index) =>
        renderMarkdownSlide(
          slide.content,
          filePath,
          slide.notes,
          deckFrontmatter,
          `Slide ${index + 1}`,
          markdownOptions,
        ),
      ),
    ),
  };
}

async function renderDeckEntryFiles(
  entries: SlideFileEntry[],
  root: string,
  options: ResolvedSlidesPluginOptions,
): Promise<SlideRenderFragment[]> {
  const napi = await loadNapiModule();

  return Promise.all(
    entries.map(async (entry, index) => {
      const source = await fs.readFile(entry.filePath, "utf-8");
      const ext = normalizeExtension(path.extname(entry.filePath));

      if (ext === ".md" || ext === ".markdown") {
        const extracted = napi.extractSlideComments(source);
        return renderMarkdownSlide(
          extracted.content,
          entry.filePath,
          extracted.notes,
          {},
          `Slide ${index + 1}`,
          options.markdown,
        );
      }

      if (ext === ".html") {
        return renderHtmlSlide(source, entry.filePath, options, {}, `Slide ${index + 1}`);
      }

      return renderCustomSlide(source, entry.filePath, root, options, `Slide ${index + 1}`);
    }),
  );
}

/**
 * Builds normalized deck metadata and rendered slide fragments from slide sources.
 */
export async function buildSlideDecks(
  options: ResolvedSlidesPluginOptions,
  root: string,
): Promise<SlideDeckData[]> {
  const slidesDir = path.resolve(root, options.srcDir);
  try {
    await fs.access(slidesDir);
  } catch {
    return [];
  }

  const decks = classifyDecks(await collectSlideFiles(options, root), slidesDir);
  const outDir = path.resolve(root, options.outDir);
  const results: SlideDeckData[] = [];

  for (const [slug, deckEntries] of decks) {
    const sortedEntries = [...deckEntries].sort((left, right) =>
      left.filePath.localeCompare(right.filePath),
    );
    const firstEntry = sortedEntries[0];
    if (!firstEntry) {
      continue;
    }

    const firstExt = normalizeExtension(path.extname(firstEntry.filePath));
    let deckFrontmatter: Record<string, unknown> = {};
    const isDeckMarkdownFile =
      sortedEntries.length === 1 &&
      (firstExt === ".md" || firstExt === ".markdown") &&
      !/^\d+$/.test(path.basename(firstEntry.filePath, firstExt));
    const renderedSlides = isDeckMarkdownFile
      ? await renderDeckMarkdownFile(firstEntry.filePath, options.separator, options.markdown)
      : {
          deckFrontmatter,
          renderedSlides: await renderDeckEntryFiles(sortedEntries, root, options),
        };
    deckFrontmatter = renderedSlides.deckFrontmatter;

    const deckTitle =
      (typeof deckFrontmatter.title === "string" && deckFrontmatter.title) ||
      (typeof deckFrontmatter.deckTitle === "string" && deckFrontmatter.deckTitle) ||
      renderedSlides.renderedSlides[0]?.title ||
      formatTitle(slug || "slides");
    const deckDescription =
      (typeof deckFrontmatter.description === "string" && deckFrontmatter.description) ||
      (typeof deckFrontmatter.deckDescription === "string" && deckFrontmatter.deckDescription) ||
      renderedSlides.renderedSlides.find((slide) => slide.description)?.description;
    const slideCount = renderedSlides.renderedSlides.length;

    const slides = renderedSlides.renderedSlides.map((slide, index) => ({
      ...slide,
      slideNumber: index + 1,
      slideCount,
      href: getSlideHref(options.base, options.routeBase, slug, index + 1, options.ssg.extension),
      outputPath: getSlideOutputPath(
        outDir,
        options.routeBase,
        slug,
        index + 1,
        options.ssg.extension,
      ),
      presenterHref: options.presenter
        ? getPresenterHref(options.base, options.routeBase, slug, index + 1, options.ssg.extension)
        : undefined,
      presenterOutputPath: options.presenter
        ? getPresenterOutputPath(outDir, options.routeBase, slug, index + 1, options.ssg.extension)
        : undefined,
    }));

    results.push({
      slug,
      title: deckTitle,
      description: deckDescription,
      href: getDeckHref(options.base, options.routeBase, slug, options.ssg.extension),
      outputPath: getDeckOutputPath(outDir, options.routeBase, slug, options.ssg.extension),
      printOutputPath: options.pdf.enabled
        ? getDeckPrintOutputPath(outDir, options.routeBase, slug)
        : undefined,
      pdfOutputPath: options.pdf.enabled
        ? getDeckPdfOutputPath(outDir, options.routeBase, slug, options.pdf.fileName)
        : undefined,
      presenterHref: slides[0]?.presenterHref,
      presenterOutputPath: slides[0]?.presenterOutputPath
        ? path.join(outDir, options.routeBase, slug, "presenter", `index${options.ssg.extension}`)
        : undefined,
      slides,
    });
  }

  return results.sort((left, right) => left.slug.localeCompare(right.slug));
}
