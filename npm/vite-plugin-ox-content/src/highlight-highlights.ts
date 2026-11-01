import { codeToTokens, isSupportedLanguage, type Theme } from "@pierre/highlights";

const theme: Theme = {
  name: "ox-content",
  appearance: "dark",
  style: {},
  cssVariables: true,
};
const prefix = "--octc-syntax-token-";
const foreground = "var(--octc-syntax-foreground, #e6edf3)";
const colors = {
  comment: "#8b949e",
  constant: "#79c0ff",
  function: "#d2a8ff",
  keyword: "#ff7b72",
  parameter: "#ffa657",
  punctuation: "#c9d1d9",
  string: "#a5d6ff",
  link: "#a5d6ff",
  "string-expression": "#a5d6ff",
} as const;

function tokenColor(color: string | undefined): string {
  const kind = color?.slice(`var(${prefix}`.length, -1) ?? "";
  let token: keyof typeof colors | undefined;
  if (kind.startsWith("comment")) token = "comment";
  else if (
    /^(constant|boolean|number|type|attribute|property|tag|namespace|enum|variant|emphasis)/.test(
      kind,
    )
  )
    token = "constant";
  else if (/^(function|constructor|title)/.test(kind)) token = "function";
  else if (/^(keyword|operator|label|preproc)/.test(kind)) token = "keyword";
  else if (kind === "variable-parameter") token = "parameter";
  else if (kind.startsWith("punctuation")) token = "punctuation";
  else if (kind.startsWith("string") || kind === "text-literal") token = "string";
  else if (kind.startsWith("link")) token = "link";
  else if (kind === "embedded") token = "string-expression";
  return token ? `var(--octc-syntax-token-${token}, ${colors[token]})` : foreground;
}

function escapeCode(value: string): string {
  return value.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
}

/** Synchronous after the containing module's lazy WebAssembly initialization. */
export function highlightWithHighlights(code: string, language: string): string | null {
  if (
    ["text", "plain", "plaintext", "txt"].includes(language.toLowerCase()) ||
    !isSupportedLanguage(language)
  )
    return null;
  const result = codeToTokens(code, { lang: language, theme, cssVariablePrefix: prefix });
  const lines = result.tokens
    .map((line) => {
      const content = line
        .map(
          (token) =>
            `<span style="color:${tokenColor(token.color)}">${escapeCode(token.content)}</span>`,
        )
        .join("");
      return `<span class="line">${content}</span>`;
    })
    .join("\n");
  return `<pre class="ox-highlight css-variables" data-highlighter="highlights" style="background-color:var(--octc-syntax-background, #0d1117);color:${foreground}" tabindex="0"><code>${lines}</code></pre>`;
}
