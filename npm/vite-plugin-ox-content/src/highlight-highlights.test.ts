import { describe, expect, it } from "vite-plus/test";
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
    "keeps %s fences as plain code",
    async (language) => {
      const result = await highlightCode(
        `<pre><code class="language-${language}">const 雪 = 1;\n</code></pre>`,
        options,
      );
      expect(result).not.toContain("ox-highlight");
      expect(getTextContent(parseHtmlFragment(result))).toBe("const 雪 = 1;\n");
    },
  );

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
