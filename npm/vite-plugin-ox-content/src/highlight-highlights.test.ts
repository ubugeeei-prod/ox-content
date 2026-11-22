import { describe, expect, it } from "vite-plus/test";
import { tokenNames } from "@pierre/highlights";
import { highlightsTheme } from "./highlight-highlights";
import { highlightCode } from "./highlight";
import { getTextContent } from "./highlight-native";
import { parseHtmlFragment } from "./html-fragment";
import { resolveOptions } from "./resolve-options";
import { transformMarkdown } from "./transform";

const options = { engine: "highlights" } as const;

describe("Highlights backend", () => {
  it("uses existing theme-color variables and preserves Unicode, whitespace and escaped HTML", async () => {
    const code = 'const 雪 = "🌸 < > & \\"";\n\t// whitespace\n\n';
    const escaped = code.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;");
    const result = await highlightCode(
      `<pre><code class="language-ts">${escaped}</code></pre>`,
      options,
    );
    expect(result).toContain('data-highlighter="highlights"');
    expect(result).toContain("var(--octc-syntax-token-keyword, #ff7b72)");
    expect(result).toContain("var(--octc-syntax-token-string, #a5d6ff)");
    expect(getTextContent(parseHtmlFragment(result))).toBe(code);
    expect(result).not.toContain("🌸 < > &");
  });

  it("keeps earlier results valid across subsequent Wasm calls", async () => {
    const first = await highlightCode(
      '<pre><code class="language-js">const first = 1;</code></pre>',
      options,
    );
    await highlightCode('<pre><code class="language-python">print("second")</code></pre>', options);
    expect(getTextContent(parseHtmlFragment(first))).toBe("const first = 1;");
  });

  it("highlights tagged inline code while retaining its original classes", async () => {
    const result = await highlightCode(
      '<p><code class="signature language-ts">const value = 1</code></p>',
      options,
    );
    expect(result).toContain("signature language-ts ox-highlight-inline");
    expect(result).not.toContain("<pre");
    expect(getTextContent(parseHtmlFragment(result))).toBe("const value = 1");
  });

  it.each(["text", "plain", "plaintext", "txt"])(
    "wraps %s fences with the native plain-text theme",
    async (language) => {
      const result = await highlightCode(
        `<pre><code class="language-${language}">const 雪 = 1;\n</code></pre>`,
        options,
      );
      expect(result).toContain("ox-highlight css-variables");
      expect(result).toContain("--octc-syntax-background");
      expect(result).not.toContain("--octc-syntax-token-");
      expect(result).toContain('tabindex="0"');
      expect(getTextContent(parseHtmlFragment(result))).toBe("const 雪 = 1;\n");
    },
  );

  it("preserves explicit tabindex and metadata on plain code blocks", async () => {
    const result = await highlightCode(
      '<pre tabindex="-1" class="example" data-note="keep"><code class="language-text">plain</code></pre>',
      options,
    );
    expect(result).toContain('tabindex="-1"');
    expect(result).toContain('class="ox-highlight css-variables example"');
    expect(result).toContain('data-note="keep"');
    expect(result).toContain("--octc-syntax-background");
  });

  it("falls back to Tree-sitter for fish blocks and inline code", async () => {
    const source = "function greet; echo hello; end";
    const result = await highlightCode(
      `<pre><code class="language-fish">${source}</code></pre><p><code class="language-fish">${source}</code></p>`,
      options,
    );
    expect(result).toContain("ox-highlight css-variables");
    expect(result).toContain("ox-highlight-inline");
    expect(result).toContain("--octc-syntax-token-keyword");
    expect(result).not.toContain('data-highlighter="highlights"');
    expect(getTextContent(parseHtmlFragment(result))).toBe(source + source);
  });

  it("colors the body of diff lines and specialized variable and selector tokens", async () => {
    for (const [language, code, color, text] of [
      ["diff", "-old\n+new", "keyword", "old"],
      ["diff", "-old\n+new", "string", "new"],
      ["zig", "pub fn a(self: *@This()) void { _ = self; }", "constant", "_"],
      ["html", "&lt;style>a:hover {color:red}&lt;/style>", "constant", ":hover"],
    ]) {
      const result = await highlightCode(
        `<pre><code class="language-${language}">${code}</code></pre>`,
        options,
      );
      expect(result).toMatch(new RegExp(`--octc-syntax-token-${color}, [^)]*\\)">${text}`));
    }
  });

  it("assigns each upstream token an explicit scope or a themed parent", () => {
    for (const token of tokenNames) {
      if (["none", "background", "foreground"].includes(token)) continue;
      const parts = token.split(".");
      while (parts.length && !Object.hasOwn(highlightsTheme.style.syntax, parts.join(".")))
        parts.pop();
      expect(parts.length, token).toBeGreaterThan(0);
    }
  });

  it("leaves unsupported languages unhighlighted", async () => {
    const result = await highlightCode(
      '<pre><code class="language-brainfuck">+[&lt;]&gt;</code></pre>',
      options,
    );
    expect(result).not.toContain("ox-highlight");
    expect(getTextContent(parseHtmlFragment(result))).toBe("+[<]>");
  });

  it("preserves annotations, line numbers and anchors through the full Markdown pipeline", async () => {
    const result = await transformMarkdown(
      '```ts:line-numbers=27 :line-links=sample {1} [sample.ts]\nconst first = "雪🌸";\nconst second = 2;\n```\n',
      "sample.md",
      resolveOptions({
        highlight: options,
        codeAnnotations: { notation: "both" },
        docs: false,
        search: false,
        resources: false,
        ssg: false,
      }),
    );
    expect(result.html).toContain('data-highlighter="highlights"');
    expect(result.html).toContain("ox-code-line--highlight");
    expect(result.html).toContain('id="sample-L27"');
    expect(result.html).toContain('data-line-anchor="sample-L27"');
    expect(result.html).toContain("ox-code-block--line-numbers");
    expect(result.html).toContain("雪🌸");
  });

  it("retains boolean defaults and resolves the opt-in backend separately", () => {
    expect(resolveOptions({}).highlight).toBe(false);
    expect(resolveOptions({ highlight: true }).highlightOptions).toBeUndefined();
    const resolved = resolveOptions({ highlight: options });
    expect(resolved.highlight).toBe(true);
    expect(resolved.highlightOptions).toEqual(options);
  });
});
