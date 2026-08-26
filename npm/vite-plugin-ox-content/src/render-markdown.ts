/**
 * Programmatic Markdown/MDX pipeline for hosts that do not import files through Vite.
 */

import { resolveOptions } from "./resolve-options";
import { transformMarkdown } from "./transform";
import type { OxContentOptions, TransformResult } from "./types";

export type { TransformResult };

/**
 * Processor that resolves `OxContentOptions` once and renders many documents.
 */
export interface MarkdownProcessor {
  render(source: string, filePath: string): Promise<TransformResult>;
}

/**
 * Resolves public options once so custom `ssg: false` hosts can reuse the pipeline.
 */
export function createMarkdownProcessor(options: OxContentOptions = {}): MarkdownProcessor {
  const resolved = resolveOptions(options);
  return {
    render(source, filePath) {
      return transformMarkdown(source, filePath, resolved);
    },
  };
}

/**
 * Run the Vite plugin Markdown/MDX pipeline from public `OxContentOptions`.
 *
 * Returns structured `TransformResult` fields (`html`, `frontmatter`, `toc`,
 * MDX metadata) so consumers do not need to cast a Vite hook or parse
 * generated module source. `.md` / `.mdx` inference matches `oxContent()`.
 */
export function renderMarkdown(
  source: string,
  filePath: string,
  options: OxContentOptions = {},
): Promise<TransformResult> {
  return createMarkdownProcessor(options).render(source, filePath);
}
