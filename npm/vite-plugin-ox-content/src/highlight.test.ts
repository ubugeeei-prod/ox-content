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
