/**
 * Syntax highlighting with the native tree-sitter engine.
 *
 * Markup uses `<pre class="ox-highlight css-variables">` and `--octc-syntax-*`
 * custom properties so theme-color packages resolve token colors.
 */

import { unified } from "unified";
import { parseHtmlFragment, rehypeParseFragment } from "./html-fragment";
import rehypeStringifyPlugin from "rehype-stringify";
import type { Root, Element } from "hast";
import type { HighlightOptions } from "./types";
import { interopDefault } from "./interop";
import {
  getTextContent,
  highlightDocumentNatively,
  highlightNatively,
  normalizeClassName,
} from "./highlight-native";

// ESM-only plugins are double-wrapped by the CommonJS interop; unwrap. See #452.
const rehypeStringify = interopDefault(rehypeStringifyPlugin);

function highlightNativeElement(code: string, language: string): Element | null {
  const html = highlightNatively(code, language);
  const pre = html && parseHtmlFragment(html).children[0];
  return pre && pre.type === "element" ? pre : null;
}

/**
 * Per-block walk used when the native document pass cannot read the markup.
 * Unknown languages stay as the original `<pre><code>`.
 */
function rehypeCodeHighlight(highlighter?: (code: string, language: string) => Element | null) {
  function highlight(code: Element, language: string) {
    const source = getTextContent(code);
    return highlighter?.(source, language) ?? highlightNativeElement(source, language);
  }
  return (tree: Root) => {
    const highlightBlockCode = (pre: Element, codeElement: Element): Element | null => {
      let lang = "text";
      const originalCodeClasses = normalizeClassName(codeElement.properties?.className);

      const langClass = originalCodeClasses.find((value) => value.startsWith("language-"));
      if (langClass) {
        lang = langClass.replace("language-", "");
      }

      const highlighted = highlight(codeElement, lang);
      if (!highlighted) return null;
      const generated = highlighted.properties;
      highlighted.properties = {
        ...pre.properties,
        ...generated,
        className: [
          ...new Set([
            ...normalizeClassName(generated.className),
            ...normalizeClassName(pre.properties.className),
          ]),
        ],
        style: [generated.style, pre.properties.style].filter(Boolean).join(";"),
        tabIndex: pre.properties.tabIndex ?? generated.tabIndex,
        dataLanguage: lang,
      };
      return highlighted;
    };

    const highlightInlineCode = (codeElement: Element): Element | null => {
      const originalCodeClasses = normalizeClassName(codeElement.properties?.className);

      const langClass = originalCodeClasses.find((value) => value.startsWith("language-"));
      if (!langClass) {
        return null;
      }

      const lang = langClass.replace("language-", "");
      const highlighted = highlight(codeElement, lang);
      const highlightedCode = highlighted?.children.find(
        (child): child is Element => child.type === "element" && child.tagName === "code",
      );
      if (!highlightedCode) return null;
      const highlightedClasses = normalizeClassName(highlightedCode.properties.className);
      highlightedCode.properties = {
        ...codeElement.properties,
        ...highlightedCode.properties,
        className: [
          ...new Set([...originalCodeClasses, ...highlightedClasses, "ox-highlight-inline"]),
        ],
        dataLanguage: lang,
      };
      return highlightedCode;
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
            "ox-highlight",
          );

          if (codeElement && !alreadyHighlighted) {
            const highlightedPre = highlightBlockCode(child, codeElement);
            if (highlightedPre) {
              node.children[i] = highlightedPre;
            } else {
              const tag =
                normalizeClassName(codeElement.properties?.className)
                  .find((value) => value.startsWith("language-"))
                  ?.slice("language-".length) ?? "text";
              if (["text", "plain", "plaintext", "txt"].includes(tag.toLowerCase())) {
                child.properties ??= {};
                child.properties.tabIndex ??= 0;
              }
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
 * Apply the selected syntax-highlighting backend to HTML.
 *
 * Tries the document pass first. If that pass skips unreadable markup, falls
 * back to a per-block walk. Languages without a supported grammar stay
 * as the original `<pre><code>`.
 */
export async function highlightCode(html: string, options?: HighlightOptions): Promise<string> {
  const highlights =
    options?.engine === "highlights"
      ? (await import("./highlight-highlights")).highlightWithHighlights
      : undefined;
  const native = highlights ? null : await highlightDocumentNatively(html);
  if (native && native.skipped.length === 0) {
    return native.html;
  }

  const result = await unified()
    .use(rehypeParseFragment)
    .use(rehypeCodeHighlight, highlights)
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
  options?: HighlightOptions,
): Promise<string> {
  const native = options?.engine === "highlights" ? null : await highlightDocumentNatively(html);
  if (native && native.skipped.length === 0) {
    return native.html;
  }

  return mergeHighlightedCodeBlocks(html, await highlightCode(html, options));
}
