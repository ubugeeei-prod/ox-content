import { codeToTokens, isSupportedLanguage, type Theme } from "@pierre/highlights";
import type { Element, ElementContent } from "hast";

// Highlights inherits dotted scopes from their parent. The palette maps those
// scopes directly onto the variables provided by ox-content color themes.
export const highlightsTheme = {
  name: "ox-content",
  appearance: "dark",
  style: {
    syntax: {
      attribute: "token-constant",
      boolean: "token-constant",
      comment: "token-comment",
      constant: "token-constant",
      constructor: "token-function",
      "diff.minus": "token-keyword",
      "diff.plus": "token-string",
      embedded: "token-string-expression",
      emphasis: "token-constant",
      enum: "token-constant",
      function: "token-function",
      hint: "token-comment",
      keyword: "token-keyword",
      label: "token-keyword",
      link_text: "token-link",
      link_uri: "token-link",
      namespace: "token-constant",
      number: "token-constant",
      operator: "token-keyword",
      predictive: "token-comment",
      preproc: "token-keyword",
      primary: "token-constant",
      property: "token-constant",
      punctuation: "token-punctuation",
      selector: "token-constant",
      string: "token-string",
      tag: "token-constant",
      text: "foreground",
      "text.literal": "token-string",
      title: "token-function",
      type: "token-constant",
      variable: "foreground",
      "variable.parameter": "token-parameter",
      "variable.special": "token-constant",
      variant: "token-constant",
    },
  },
  cssVariables: {
    prefix: "--octc-syntax-",
    defaults: {
      background: "#0d1117",
      foreground: "#e6edf3",
      "token-comment": "#8b949e",
      "token-constant": "#79c0ff",
      "token-function": "#d2a8ff",
      "token-keyword": "#ff7b72",
      "token-link": "#a5d6ff",
      "token-parameter": "#ffa657",
      "token-punctuation": "#c9d1d9",
      "token-string": "#a5d6ff",
      "token-string-expression": "#a5d6ff",
    },
  },
} satisfies Theme;

/** Build the rehype node directly so its serializer escapes the source once. */
export function highlightWithHighlights(code: string, language: string): Element | null {
  if (
    ["text", "plain", "plaintext", "txt"].includes(language.toLowerCase()) ||
    !isSupportedLanguage(language)
  ) {
    return null;
  }
  const { tokens, fg, bg } = codeToTokens(code, { lang: language, theme: highlightsTheme });
  const lines: ElementContent[] = [];
  for (const line of tokens) {
    if (lines.length > 0) lines.push({ type: "text", value: "\n" });
    lines.push({
      type: "element",
      tagName: "span",
      properties: { className: ["line"] },
      children: line.map((token) => ({
        type: "element",
        tagName: "span",
        properties: { style: `color:${token.color ?? fg}` },
        children: [{ type: "text", value: token.content }],
      })),
    });
  }
  return {
    type: "element",
    tagName: "pre",
    properties: {
      className: ["ox-highlight", "css-variables"],
      dataHighlighter: "highlights",
      style: `background-color:${bg};color:${fg}`,
      tabIndex: 0,
    },
    children: [{ type: "element", tagName: "code", properties: {}, children: lines }],
  };
}
