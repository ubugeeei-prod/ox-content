/**
 * Highlighting the blocks the native pass could not claim.
 *
 * These arrive already named — the native pass reports each one's language —
 * which lets this load exactly the grammars a page needs instead of the whole
 * bundled set, and splice each result back without an HTML parser.
 */

import {
  createHighlighter,
  type BundledLanguage,
  type BundledTheme,
  type Highlighter,
  type LanguageRegistration,
  type ThemeRegistration,
} from "shiki";
import { BUILTIN_LANGS, normalizeThemeInput } from "./highlight";
import { CSS_VARIABLES_THEME } from "./shiki-theme";

/**
 * A highlighter that starts with no grammars and gains them on demand.
 *
 * `getHighlighter` loads all two dozen `BUILTIN_LANGS` up front, because the
 * tree walk it serves discovers a page's languages only while rewriting it.
 * The pending list does not have that problem — it names every language it
 * needs — and paying for two dozen TextMate grammars to highlight one Vue
 * block is most of what a page with an exotic block costs.
 */
const lazyHighlighterCache = new Map<string, Promise<Highlighter>>();
const loadedLangs = new WeakMap<Highlighter, Set<string>>();

async function getLazyHighlighter(
  theme: string | ThemeRegistration,
  customLangs: LanguageRegistration[],
  wanted: readonly string[],
): Promise<Highlighter> {
  const { themeInput } = normalizeThemeInput(theme);
  const cacheKey = JSON.stringify({ theme: themeInput, langs: customLangs });

  let highlighterPromise = lazyHighlighterCache.get(cacheKey);
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      themes: [themeInput as BundledTheme | ThemeRegistration],
      langs: customLangs,
    });
    lazyHighlighterCache.set(cacheKey, highlighterPromise);
  }
  const highlighter = await highlighterPromise;

  // Only grammars `getHighlighter` would have loaded are loaded here, so a
  // language neither of them carries stays unhighlighted either way. Adding
  // one would change how a page renders, which is not this function's call to
  // make.
  let loaded = loadedLangs.get(highlighter);
  if (!loaded) {
    loaded = new Set();
    loadedLangs.set(highlighter, loaded);
  }
  const missing = [
    ...new Set(
      wanted.filter(
        (lang) => !loaded.has(lang) && (BUILTIN_LANGS as readonly string[]).includes(lang),
      ),
    ),
  ];
  if (missing.length > 0) {
    await Promise.all(
      missing.map(async (lang) => {
        try {
          await highlighter.loadLanguage(lang as BundledLanguage);
          loaded.add(lang);
        } catch {
          // Leave it unloaded; `codeToHtml` then declines the block, which is
          // what the tree walk did for a grammar it could not resolve.
        }
      }),
    );
  }

  return highlighter;
}

/**
 * Highlight the blocks the native pass left pending, in order.
 *
 * Entry `i` of the result is the `<pre>` for block `i`, or an empty string
 * when Shiki has no grammar for it either — in which case the block is left
 * exactly as it arrived, the same outcome the tree walk reached by keeping the
 * original element.
 *
 * This is the whole point of the pending list: a page whose only unsupported
 * block is a Mermaid diagram used to be handed to the tree walk in full, which
 * re-highlighted every one of its other blocks and paid for a parse and a
 * serialize of the page to do it.
 */
export async function highlightPendingBlocks(
  blocks: readonly { language: string; source: string }[],
  theme: string | ThemeRegistration = CSS_VARIABLES_THEME,
  langs: LanguageRegistration[] = [],
): Promise<string[]> {
  if (blocks.length === 0) {
    return [];
  }

  const { themeName } = normalizeThemeInput(theme);
  const highlighter = await getLazyHighlighter(
    theme,
    langs,
    blocks.map((block) => block.language),
  );

  return blocks.map((block) => {
    try {
      return highlighter.codeToHtml(block.source, {
        lang: block.language as never,
        theme: themeName as BundledTheme,
      });
    } catch {
      return "";
    }
  });
}
