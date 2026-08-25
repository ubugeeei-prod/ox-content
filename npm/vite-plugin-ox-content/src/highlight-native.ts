/**
 * The native tree-sitter highlighting path, plus the small hast helpers the
 * per-block walk uses when the document pass cannot read the markup.
 */

import type { Root, Element } from "hast";

import { importNapiModuleSync } from "./napi";

/**
 * Extract text content from a hast node.
 */
export function getTextContent(node: Element | Root): string {
  let text = "";

  if ("children" in node) {
    for (const child of node.children) {
      if (child.type === "text") {
        text += child.value;
      } else if (child.type === "element") {
        text += getTextContent(child);
      }
    }
  }

  return text;
}

export function normalizeClassName(className: unknown): string[] {
  if (Array.isArray(className)) {
    return className.filter((value): value is string => typeof value === "string");
  }

  if (typeof className === "string" && className) {
    return className.split(/\s+/).filter(Boolean);
  }

  return [];
}

/**
 * Highlights with the native tree-sitter engine, or `null` when it has no
 * grammar for `lang`.
 *
 * It emits `--octc-syntax-*` markup so theme-color packages resolve token
 * colors.
 */
export function highlightNatively(code: string, lang: string): string | null {
  try {
    return importNapiModuleSync().highlightCodeBlock(code, lang);
  } catch {
    return null;
  }
}

/**
 * Highlights every code block in a rendered document in one native call.
 *
 * Returns the rewritten HTML and the languages it declined. Pending languages
 * stay unhighlighted. Returns `null` when the native module is unavailable.
 */
export async function highlightDocumentNatively(html: string): Promise<NativeDocument | null> {
  try {
    return await importNapiModuleSync().highlightHtmlCodeBlocksAsync(html);
  } catch {
    return null;
  }
}

/** A block the native pass left unhighlighted (no grammar). */
export interface PendingBlock {
  language: string;
  source: string;
}

/** What {@link highlightDocumentNatively} produced for a page. */
export interface NativeDocument {
  html: string;
  /**
   * Languages of elements the native pass could not read. Non-empty means the
   * page has to be produced by the per-block walk instead.
   */
  skipped: string[];
  /** Well-formed blocks whose language has no native grammar, in order. */
  pending: PendingBlock[];
}
