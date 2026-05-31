import * as path from "node:path";
import { transformMarkdown, type ResolvedOptions } from "@ox-content/vite-plugin";
import { loadNapiModule } from "./napi";
import type { ResolvedSlidesPluginOptions, SlidePageData } from "./internal-types";
import { normalizeExtension } from "./path-utils";
import { wrapSlideContent } from "./slide-layout";

/**
 * Shared render payload produced by every slide source renderer.
 */
export type SlideRenderFragment = Pick<
  SlidePageData,
  "title" | "description" | "contentHtml" | "notesHtml" | "notes" | "frontmatter" | "sourcePath"
>;

function extractTitle(
  contentHtml: string,
  frontmatter: Record<string, unknown>,
  fallback: string,
): string {
  if (typeof frontmatter.title === "string" && frontmatter.title.trim()) {
    return frontmatter.title.trim();
  }

  const match = contentHtml.match(/<h1[^>]*>([\s\S]*?)<\/h1>/i);
  return match ? match[1].replace(/<[^>]+>/g, "").trim() : fallback;
}

function normalizeNotes(notes: string | string[] | undefined): string[] {
  if (!notes) {
    return [];
  }

  return (Array.isArray(notes) ? notes : [notes]).map((note) => note.trim()).filter(Boolean);
}

async function renderNotesHtml(
  notes: string[],
  sourcePath: string,
  markdownOptions: ResolvedOptions,
): Promise<string | undefined> {
  if (notes.length === 0) {
    return undefined;
  }

  const transformed = await transformMarkdown(
    notes.join("\n\n"),
    `${sourcePath}#notes.md`,
    markdownOptions,
  );
  return transformed.html;
}

/**
 * Renders a Markdown slide into HTML and extracted note markup.
 */
export async function renderMarkdownSlide(
  source: string,
  sourcePath: string,
  notes: string[],
  deckFrontmatter: Record<string, unknown>,
  fallbackTitle: string,
  markdownOptions: ResolvedOptions,
): Promise<SlideRenderFragment> {
  const transformed = await transformMarkdown(source, sourcePath, markdownOptions);
  const frontmatter = { ...deckFrontmatter, ...transformed.frontmatter };

  return {
    title: extractTitle(transformed.html, frontmatter, fallbackTitle),
    description: typeof frontmatter.description === "string" ? frontmatter.description : undefined,
    contentHtml: wrapSlideContent(transformed.html, frontmatter),
    notesHtml: await renderNotesHtml(notes, sourcePath, markdownOptions),
    notes,
    frontmatter,
    sourcePath,
  };
}

/**
 * Renders a raw HTML slide after extracting HTML comment notes.
 */
export async function renderHtmlSlide(
  source: string,
  sourcePath: string,
  options: ResolvedSlidesPluginOptions,
  deckFrontmatter: Record<string, unknown>,
  fallbackTitle: string,
): Promise<SlideRenderFragment> {
  const napi = await loadNapiModule();
  const extracted = napi.extractSlideComments(source);

  return {
    title: extractTitle(extracted.content, deckFrontmatter, fallbackTitle),
    description:
      typeof deckFrontmatter.description === "string" ? deckFrontmatter.description : undefined,
    contentHtml: wrapSlideContent(extracted.content, deckFrontmatter),
    notesHtml: await renderNotesHtml(extracted.notes, sourcePath, options.markdown),
    notes: extracted.notes,
    frontmatter: { ...deckFrontmatter },
    sourcePath,
  };
}

/**
 * Renders a custom slide source via a user-provided renderer.
 */
export async function renderCustomSlide(
  source: string,
  sourcePath: string,
  root: string,
  options: ResolvedSlidesPluginOptions,
  fallbackTitle: string,
): Promise<SlideRenderFragment> {
  const ext = normalizeExtension(path.extname(sourcePath));
  const renderer = options.renderers[ext];
  if (!renderer) {
    throw new Error(`No slide renderer is registered for '${ext}'.`);
  }

  const result = await renderer(source, { filePath: sourcePath, root, options });
  const frontmatter = result.frontmatter ?? {};
  const notes = normalizeNotes(result.notes);

  return {
    title: extractTitle(result.html, frontmatter, result.title ?? fallbackTitle),
    description:
      result.description ??
      (typeof frontmatter.description === "string" ? frontmatter.description : undefined),
    contentHtml: wrapSlideContent(result.html, frontmatter),
    notesHtml: await renderNotesHtml(notes, sourcePath, options.markdown),
    notes,
    frontmatter,
    sourcePath,
  };
}
