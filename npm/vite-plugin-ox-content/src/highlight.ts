/**
 * Syntax highlighting with Shiki via rehype.
 */

import { unified } from "unified";
import rehypeParsePlugin from "rehype-parse";
import rehypeStringifyPlugin from "rehype-stringify";
import type { Root, Element } from "hast";
import {
  createHighlighter,
  type Highlighter,
  type BundledTheme,
  type LanguageRegistration,
  type ThemeRegistration,
} from "shiki";
import { interopDefault } from "./interop";
import {
  getTextContent,
  highlightNatively,
  normalizeClassName,
  treeNeedsShiki,
} from "./highlight-native";
import { CSS_VARIABLES_THEME, resolveHighlightTheme } from "./shiki-theme";

// ESM-only plugins are double-wrapped by the CommonJS interop; unwrap. See #452.
const rehypeParse = interopDefault(rehypeParsePlugin);
const rehypeStringify = interopDefault(rehypeStringifyPlugin);

const BUILTIN_LANGS = [
  "javascript",
  "typescript",
  "jsx",
  "tsx",
  "vue",
  "svelte",
  "html",
  "css",
  "scss",
  "json",
  "yaml",
  "markdown",
  "bash",
  "shell",
  "rust",
  "python",
  "go",
  "java",
  "c",
  "cpp",
  "sql",
  "graphql",
  "diff",
  "toml",
] as const;

// Cache highlighters by theme + language registration set.
const highlighterCache = new Map<string, Promise<Highlighter>>();

/**
 * Get or create the Shiki highlighter.
 */
async function getHighlighter(
  theme: string | ThemeRegistration,
  customLangs: LanguageRegistration[] = [],
): Promise<Highlighter> {
  const { themeInput } = normalizeThemeInput(theme);
  const cacheKey = JSON.stringify({
    theme: themeInput,
    langs: customLangs,
  });

  let highlighterPromise = highlighterCache.get(cacheKey);
  if (!highlighterPromise) {
    highlighterPromise = createHighlighter({
      themes: [themeInput as BundledTheme | ThemeRegistration],
      langs: [...BUILTIN_LANGS, ...customLangs],
    });
    highlighterCache.set(cacheKey, highlighterPromise);
  }
  return highlighterPromise;
}

function normalizeThemeInput(input: string | ThemeRegistration): {
  themeInput: string | ThemeRegistration;
  themeName: string;
} {
  // `"css-variables"` is an alias rather than a bundled Shiki theme, so expand
  // it here — every caller funnels through this function.
  const theme = resolveHighlightTheme(input);

  if (typeof theme === "string") {
    return {
      themeInput: theme,
      themeName: theme,
    };
  }

  const themeName = theme.name || "ox-content-custom-theme";
  return {
    themeInput: theme.name ? theme : { ...theme, name: themeName },
    themeName,
  };
}

/**
 * Rehype plugin for syntax highlighting with Shiki.
 */
function rehypeShikiHighlight(options: {
  theme: string | ThemeRegistration;
  langs?: LanguageRegistration[];
}) {
  const { theme, langs } = options;

  return async (tree: Root) => {
    const { themeName } = normalizeThemeInput(theme);
    // Deferred on purpose: a document whose languages the native engine all
    // covers must never construct a Shiki highlighter, because doing so parses
    // two dozen TextMate grammars up front.
    // The native engine emits the `--octc-shiki-*` custom properties and
    // nothing else, so an explicitly requested bundled theme — `github-dark`,
    // `vitesse-dark` — still has to go through Shiki to get its baked colors.
    const nativeThemeApplies = themeName === CSS_VARIABLES_THEME;
    const highlighter = treeNeedsShiki(tree, nativeThemeApplies)
      ? await getHighlighter(theme, langs)
      : undefined;

    const highlightBlockCode = (codeElement: Element): Element | null => {
      let lang = "text";
      const originalCodeClasses = normalizeClassName(codeElement.properties?.className);

      const langClass = originalCodeClasses.find((value) => value.startsWith("language-"));
      if (langClass) {
        lang = langClass.replace("language-", "");
      }

      const codeText = getTextContent(codeElement);

      try {
        const highlighted =
          (nativeThemeApplies ? highlightNatively(codeText, lang) : null) ??
          highlighter?.codeToHtml(codeText, {
            lang: lang as any,
            theme: themeName as BundledTheme,
          });
        if (highlighted === undefined) {
          return null;
        }

        const parsed = unified().use(rehypeParse, { fragment: true }).parse(highlighted);

        if (parsed.children[0]?.type === "element") {
          const highlightedPre = parsed.children[0];
          highlightedPre.properties ??= {};
          highlightedPre.properties["data-language"] = lang;
          return highlightedPre;
        }
      } catch {
        // If highlighting fails, keep the original
      }

      return null;
    };

    const highlightInlineCode = (codeElement: Element): Element | null => {
      let lang = "text";
      const originalCodeClasses = normalizeClassName(codeElement.properties?.className);

      const langClass = originalCodeClasses.find((value) => value.startsWith("language-"));
      if (!langClass) {
        return null;
      }

      lang = langClass.replace("language-", "");
      const codeText = getTextContent(codeElement);

      try {
        const highlighted =
          (nativeThemeApplies ? highlightNatively(codeText, lang) : null) ??
          highlighter?.codeToHtml(codeText, {
            lang: lang as any,
            theme: themeName as BundledTheme,
          });
        if (highlighted === undefined) {
          return null;
        }

        const parsed = unified().use(rehypeParse, { fragment: true }).parse(highlighted);

        if (parsed.children[0]?.type === "element") {
          const highlightedPre = parsed.children[0];
          const highlightedCode = highlightedPre.children.find(
            (child): child is Element => child.type === "element" && child.tagName === "code",
          );

          if (highlightedCode) {
            highlightedCode.properties ??= {};
            const highlightedClasses = normalizeClassName(highlightedCode.properties.className);
            highlightedCode.properties.className = [
              ...new Set([...originalCodeClasses, ...highlightedClasses, "shiki-inline"]),
            ];
            highlightedCode.properties["data-language"] = lang;
            return highlightedCode;
          }
        }
      } catch {
        // If highlighting fails, keep the original
      }

      return null;
    };

    // Find all pre > code elements
    const visit = async (node: Root | Element) => {
      if ("children" in node) {
        for (let i = 0; i < node.children.length; i++) {
          const child = node.children[i];

          if (child.type === "element" && child.tagName === "pre") {
            const codeElement = child.children.find(
              (c): c is Element => c.type === "element" && c.tagName === "code",
            );

            // A block the native pass already handled carries `shiki` on its
            // `<pre>`. Re-running Shiki over it would read the highlighted
            // text back out and overwrite the result with its own.
            const alreadyHighlighted = normalizeClassName(child.properties?.className).includes(
              "shiki",
            );

            if (codeElement && !alreadyHighlighted) {
              const highlightedPre = highlightBlockCode(codeElement);
              if (highlightedPre) {
                node.children[i] = highlightedPre;
              }
            }
          } else if (child.type === "element" && child.tagName === "code") {
            const highlightedCode = highlightInlineCode(child);
            if (highlightedCode) {
              node.children[i] = highlightedCode;
            }
          } else if (child.type === "element") {
            await visit(child);
          }
        }
      }
    };

    await visit(tree);
  };
}

/**
 * Apply syntax highlighting to HTML using Shiki.
 */
export async function highlightCode(
  html: string,
  theme: string | ThemeRegistration = CSS_VARIABLES_THEME,
  langs: LanguageRegistration[] = [],
): Promise<string> {
  const result = await unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeShikiHighlight, { theme, langs })
    .use(rehypeStringify)
    .process(html);

  return String(result);
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
  const highlighter = await getHighlighter(theme, langs);

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
