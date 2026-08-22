import { describe, expect, it } from "vite-plus/test";
import { highlightCode } from "./highlight";

describe("highlightCode", () => {
  it("caches highlighters per theme instead of reusing the first one", async () => {
    const html = '<pre><code class="language-ts">const value = 1;</code></pre>';

    const githubDark = await highlightCode(html, "github-dark");
    const vitesseDark = await highlightCode(html, "vitesse-dark");

    expect({ githubDark, vitesseDark }).toMatchSnapshot();
  });

  it("highlights standalone language-tagged code inline", async () => {
    const html =
      '<p><code class="ox-api-entry__signature ox-api-entry__signature--highlighted language-typescript">function capitalize(value: string): string</code></p>';

    const highlighted = await highlightCode(html, "vitesse-dark");

    expect(highlighted).toMatchSnapshot();
  });
});

describe("css-variables theme", () => {
  it("renders token colors as --octc-shiki-* properties so they follow the scheme", async () => {
    const html = '<pre><code class="language-ts">const a = "x"; // c</code></pre>';

    const highlighted = await highlightCode(html);

    expect(highlighted).toContain("var(--octc-shiki-token-string");
    expect(highlighted).toContain("var(--octc-shiki-token-comment");
    // No baked hex, or the colors could not track light/dark from one build.
    expect(highlighted).not.toMatch(/color:\s*#[0-9a-f]{6}/i);
  });

  it("bakes GitHub Dark values in as var() fallbacks for sites with no scheme", async () => {
    const html = '<pre><code class="language-ts">const a = "x";</code></pre>';

    const highlighted = await highlightCode(html);

    expect(highlighted).toContain("var(--octc-shiki-token-keyword, #ff7b72)");
  });

  it("still honours an explicitly requested bundled theme", async () => {
    const html = '<pre><code class="language-ts">const a = "x";</code></pre>';

    const highlighted = await highlightCode(html, "vitesse-dark");

    expect(highlighted).not.toContain("--octc-shiki-");
    expect(highlighted).toMatch(/color:\s*#[0-9a-f]{6}/i);
  });
});

describe("native tree-sitter engine", () => {
  // The engine only emits `--octc-shiki-*` properties, so it can serve the
  // css-variables theme and nothing else. Every existing highlighting
  // snapshot happens to pin a bundled theme, so without these the native
  // path would have had no coverage at all.
  const block = (lang: string, code: string) =>
    `<pre><code class="language-${lang}">${code}</code></pre>`;

  it("highlights supported languages, splitting whitespace into its own run", async () => {
    const highlighted = await highlightCode(block("ts", "const value = 1;"));

    expect(highlighted).toContain("var(--octc-shiki-token-keyword");
    expect(highlighted).toContain("var(--octc-shiki-token-constant");
    // tree-sitter emits the gaps between tokens as their own spans; the
    // TextMate engine folded them into the neighbouring token. This is the
    // cheapest signal that the native path actually ran.
    expect(highlighted).toContain('var(--octc-shiki-foreground, #e6edf3)"> </span>');
  });

  it("leaves an explicitly requested bundled theme to Shiki", async () => {
    // Regression: routing every block through the native engine ignored the
    // requested theme and emitted CSS variables for it anyway.
    const highlighted = await highlightCode(block("ts", "const value = 1;"), "github-dark");

    expect(highlighted).not.toContain("--octc-shiki-");
    expect(highlighted).toContain("#F97583");
  });

  it("falls back to Shiki for a language it has no grammar for", async () => {
    // `vue` is in the bundled Shiki set but not the native one, so it must
    // still come out highlighted rather than plain.
    const highlighted = await highlightCode(block("vue", "<template><p>hi</p></template>"));

    expect(highlighted).toContain("var(--octc-shiki-");
    expect(highlighted).not.toContain('var(--octc-shiki-foreground, #e6edf3)"> </span>');
  });

  it("leaves a language neither engine knows unhighlighted", async () => {
    const highlighted = await highlightCode(block("brainfuck", "+[-]"));

    expect(highlighted).not.toContain("--octc-shiki-token");
    expect(highlighted).toContain("+[-]");
  });
});
