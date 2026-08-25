/**
 * Syntax highlighting with the native tree-sitter engine.
 *
 * Markup keeps the historical `<pre class="shiki css-variables">` wrapper and
 * `--octc-shiki-*` custom properties so theme-color packages keep working.
 */

import { unified } from "unified";
import rehypeParsePlugin from "rehype-parse";
import rehypeStringifyPlugin from "rehype-stringify";
import type { Root, Element } from "hast";
import { interopDefault } from "./interop";
import {
  getTextContent,
  highlightDocumentNatively,
  highlightNatively,
  normalizeClassName,
} from "./highlight-native";

// ESM-only plugins are double-wrapped by the CommonJS interop; unwrap. See #452.
const rehypeParse = interopDefault(rehypeParsePlugin);
const rehypeStringify = interopDefault(rehypeStringifyPlugin);

/**
 * Per-block walk used when the native document pass cannot read the markup.
 * Unknown languages stay as the original `<pre><code>`.
 */
function rehypeNativeHighlight() {
  return (tree: Root) => {
    const highlightBlockCode = (codeElement: Element): Element | null => {
      let lang = "text";
      const originalCodeClasses = normalizeClassName(codeElement.properties?.className);

      const langClass = originalCodeClasses.find((value) => value.startsWith("language-"));
      if (langClass) {
        lang = langClass.replace("language-", "");
      }

      const highlighted = highlightNatively(getTextContent(codeElement), lang);
      if (!highlighted) {
        return null;
      }

      try {
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
      const originalCodeClasses = normalizeClassName(codeElement.properties?.className);

      const langClass = originalCodeClasses.find((value) => value.startsWith("language-"));
      if (!langClass) {
        return null;
      }

      const lang = langClass.replace("language-", "");
      const highlighted = highlightNatively(getTextContent(codeElement), lang);
      if (!highlighted) {
        return null;
      }

      try {
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

    const visit = (node: Root | Element) => {
      if (!("children" in node)) {
        return;
      }

      for (let i = 0; i < node.children.length; i++) {
        const child = node.children[i];

        if (child.type === "element" && child.tagName === "pre") {
          const codeElement = child.children.find(
            (c): c is Element => c.type === "element" && c.tagName === "code",
          );

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
          visit(child);
        }
      }
    };

    visit(tree);
  };
}

/**
 * Apply native tree-sitter highlighting to HTML.
 *
 * Tries the document pass first. If that pass skips unreadable markup, falls
 * back to a native-only per-block walk. Languages with no native grammar stay
 * as the original `<pre><code>`.
 */
export async function highlightCode(html: string): Promise<string> {
  const native = await highlightDocumentNatively(html);
  if (native && native.skipped.length === 0) {
    return native.html;
  }

  const result = await unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeNativeHighlight)
    .use(rehypeStringify)
    .process(html);

  return String(result);
}

/**
 * Highlight every code block in a rendered page, preserving original classes
 * and per-line metadata when the native document pass cannot read the markup.
 */
export async function highlightPageHtml(
  html: string,
  mergeHighlightedCodeBlocks: (originalHtml: string, highlightedHtml: string) => string,
): Promise<string> {
  const native = await highlightDocumentNatively(html);
  if (native && native.skipped.length === 0) {
    return native.html;
  }

  return mergeHighlightedCodeBlocks(html, await highlightCode(html));
}
