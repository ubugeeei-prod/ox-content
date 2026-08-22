/**
 * The native tree-sitter highlighting path, plus the small hast helpers both
 * engines read code blocks with.
 *
 * Kept apart from the Shiki plumbing so the two engines stay legible on their
 * own, and arranged as a leaf: `highlight.ts` reaches in here, never the other
 * way round.
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
 * Parsing once and walking the tree is roughly eight times faster than
 * matching TextMate patterns line by line — 10.5 ms against 81.5 ms over the
 * documentation corpus's code blocks — and it emits the same
 * `--octc-shiki-*` markup, so themes are unaffected.
 */
export function highlightNatively(code: string, lang: string): string | null {
  try {
    return importNapiModuleSync().highlightCodeBlock(code, lang);
  } catch {
    return null;
  }
}

/** Whether the native engine claims `lang`. */
export function nativeSupports(lang: string): boolean {
  try {
    return importNapiModuleSync().supportsHighlightLanguage(lang);
  } catch {
    return false;
  }
}

/**
 * Whether any block in this tree still needs Shiki.
 *
 * Creating a Shiki highlighter parses two dozen TextMate grammars, about
 * 190 ms, so a document whose languages are all covered natively must not
 * touch it at all.
 */
export function treeNeedsShiki(tree: Root, nativeThemeApplies: boolean): boolean {
  if (!nativeThemeApplies) {
    return true;
  }
  let needed = false;
  const walk = (node: Root | Element): void => {
    if (needed || !("children" in node)) {
      return;
    }
    for (const child of node.children) {
      if (child.type !== "element") {
        continue;
      }
      if (child.tagName === "code") {
        const lang = languageOf(child);
        if (lang !== null && !nativeSupports(lang)) {
          needed = true;
          return;
        }
      }
      walk(child);
    }
  };
  walk(tree);
  return needed;
}

/** The `language-*` class on a `<code>` element, if it carries one. */
export function languageOf(codeElement: Element): string | null {
  const className = normalizeClassName(codeElement.properties?.className).find((value) =>
    value.startsWith("language-"),
  );
  return className ? className.slice("language-".length) : null;
}

/**
 * Highlights every code block in a rendered document in one native call.
 *
 * Returns the rewritten HTML and the languages it declined, so the caller
 * knows whether Shiki still has to run over the result. Returns `null` when
 * the native module is unavailable.
 *
 * This exists because the plumbing dwarfed the work: walking each page
 * through an HTML parser and serializer to find `<pre>` elements cost 139 ms
 * over the documentation corpus, and re-parsing each highlighted block to
 * splice it back cost another 38 ms, against 14 ms of actual highlighting.
 */
export function highlightDocumentNatively(html: string): NativeDocument | null {
  try {
    return importNapiModuleSync().highlightHtmlCodeBlocks(html);
  } catch {
    return null;
  }
}

/** A block the native pass left for another highlighter. */
export interface PendingBlock {
  language: string;
  source: string;
}

/** What {@link highlightDocumentNatively} produced for a page. */
export interface NativeDocument {
  html: string;
  /**
   * Languages of elements the native pass could not read. Non-empty means the
   * page has to be produced by the HTML-parser-based highlighter instead.
   */
  skipped: string[];
  /** Well-formed blocks whose language has no native grammar, in order. */
  pending: PendingBlock[];
}

/**
 * Splices `replacements` back over the blocks the native pass left pending.
 *
 * Entry `i` is the highlighted `<pre>` for pending block `i`, or an empty
 * string to leave that block alone. This keeps a page that needs one exotic
 * grammar — a Vue SFC, a Mermaid diagram — off the HTML round trip, rather
 * than surrendering the whole document for it.
 */
export function applyPendingHighlights(html: string, replacements: string[]): string {
  try {
    return importNapiModuleSync().applyPendingHighlights(html, replacements);
  } catch {
    return html;
  }
}
