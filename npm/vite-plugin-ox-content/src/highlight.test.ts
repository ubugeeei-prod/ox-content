import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import { highlightCode } from "./highlight";

const block = (lang: string, code: string) =>
  `<pre><code class="language-${lang}">${code}</code></pre>`;

describe("highlightCode", () => {
  it("highlights a supported ts block with --octc-shiki-* token variables", async () => {
    const highlighted = await highlightCode(block("ts", "const value = 1;"));

    expect(highlighted).toContain("var(--octc-shiki-token-");
    expect(highlighted).toContain("var(--octc-shiki-token-keyword");
    expect(highlighted).toContain("var(--octc-shiki-token-constant");
    // No baked hex token colors — theme-color packages resolve the variables.
    expect(highlighted).not.toMatch(/color:\s*#[0-9a-f]{6}/i);
    // tree-sitter emits the gaps between tokens as their own spans.
    expect(highlighted).toContain('var(--octc-shiki-foreground, #e6edf3)"> </span>');
  });

  it("leaves an unknown language unhighlighted", async () => {
    const highlighted = await highlightCode(block("brainfuck", "+[-]"));

    expect(highlighted).toContain("+[-]");
    expect(highlighted).not.toContain("--octc-shiki-token");
  });

  it("leaves vue unhighlighted when the native engine has no grammar", async () => {
    const highlighted = await highlightCode(
      block("vue", "&lt;template&gt;&lt;p&gt;hi&lt;/p&gt;&lt;/template&gt;"),
    );

    expect(highlighted).toContain("template");
    expect(highlighted).toContain("hi");
    expect(highlighted).not.toContain("--octc-shiki-token");
  });

  it("highlights standalone language-tagged code inline", async () => {
    const html =
      '<p><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">function capitalize(value: string): string</code></p>';

    const highlighted = await highlightCode(html);

    expect(highlighted).toContain("var(--octc-shiki-token-");
    expect(highlighted).toContain("shiki-inline");
    expect(highlighted).toContain("function");
    expect(highlighted).toContain("capitalize");
  });

  it("keeps HTML special characters escaped", async () => {
    const highlighted = await highlightCode(block("ts", 'const s = "a &lt; b &amp; c &gt; d";'));

    expect(highlighted).toContain("&lt;");
    expect(highlighted).toContain("&amp;");
    expect(highlighted).toContain("&gt;");
    expect(highlighted).not.toContain("a < b");
  });
});

describe("plugin package", () => {
  it("does not list shiki as a dependency", () => {
    const pkg = JSON.parse(
      readFileSync(join(dirname(fileURLToPath(import.meta.url)), "../package.json"), "utf8"),
    ) as {
      dependencies?: Record<string, string>;
      devDependencies?: Record<string, string>;
      peerDependencies?: Record<string, string>;
    };

    expect(pkg.dependencies?.shiki).toBeUndefined();
    expect(pkg.devDependencies?.shiki).toBeUndefined();
    expect(pkg.peerDependencies?.shiki).toBeUndefined();
  });
});
