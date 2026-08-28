import { expect, test, type Page } from "@playwright/test";
import { kanagawaDragon } from "../../../theme-color/kanagawa/src/index";
import { renderThemeTokenCss } from "../../src/theme-tokens";

const FIXTURE_CSS = `
body {
  margin: 0;
  background: var(--octc-color-bg);
  color: var(--octc-color-text);
  font-family: Arial, sans-serif;
}
.kanagawa-fixture {
  box-sizing: border-box;
  width: min(880px, calc(100vw - 40px));
  margin: 24px auto;
}
.kanagawa-fixture pre {
  margin: 0;
  padding: 20px;
  border: 1px solid var(--octc-color-border);
  border-radius: 6px;
  overflow: auto;
  background: var(--octc-syntax-background);
  color: var(--octc-syntax-foreground);
  font: 16px/1.7 ui-monospace, SFMono-Regular, Menlo, monospace;
}
.token.comment { color: var(--octc-syntax-token-comment); }
.token.keyword { color: var(--octc-syntax-token-keyword); }
.token.string { color: var(--octc-syntax-token-string); }
.token.constant { color: var(--octc-syntax-token-constant); }
.token.function { color: var(--octc-syntax-token-function); }
.token.parameter { color: var(--octc-syntax-token-parameter); }
.token.punctuation { color: var(--octc-syntax-token-punctuation); }
`;

test.describe("Kanagawa Dragon highlighted fixture", () => {
  test("renders Lotus tokens in light mode", async ({ page }) => {
    await renderFixture(page, "light");

    const colors = await collectColors(page);

    expect(colors.blockBackground).toBe("rgb(231, 219, 160)");
    expect(colors.keyword).toBe("rgb(179, 91, 121)");
    expect(colors.string).toBe("rgb(111, 137, 78)");
    expect(colors.constant).toBe("rgb(119, 113, 63)");
    expect(colors.function).toBe("rgb(77, 105, 155)");
    expect(colors.parameter).toBe("rgb(200, 64, 83)");
  });

  test("renders canonical Dragon syntax tokens in dark mode", async ({ page }) => {
    await renderFixture(page, "dark");

    const colors = await collectColors(page);

    expect(colors.blockBackground).toBe("rgb(24, 22, 22)");
    expect(colors.comment).toBe("rgb(115, 124, 115)");
    expect(colors.punctuation).toBe("rgb(158, 155, 147)");
    expect(colors.keyword).toBe("rgb(137, 146, 167)");
    expect(colors.string).toBe("rgb(138, 154, 123)");
    expect(colors.constant).toBe("rgb(182, 146, 123)");
    expect(colors.function).toBe("rgb(139, 164, 176)");
    expect(colors.parameter).toBe("rgb(166, 166, 156)");
  });
});

async function renderFixture(page: Page, theme: "light" | "dark"): Promise<void> {
  await page.setViewportSize({ width: 960, height: 720 });
  await page.setContent(
    `<!doctype html>
<html data-theme="${theme}">
  <head>
    <meta charset="utf-8">
    <style>${renderThemeTokenCss(kanagawaDragon)}</style>
    <style>
      :root {
        --octc-color-bg: ${kanagawaDragon.colors?.background};
        --octc-color-text: ${kanagawaDragon.colors?.text};
        --octc-color-border: ${kanagawaDragon.colors?.border};
      }
      [data-theme="dark"] {
        --octc-color-bg: ${kanagawaDragon.darkColors?.background};
        --octc-color-text: ${kanagawaDragon.darkColors?.text};
        --octc-color-border: ${kanagawaDragon.darkColors?.border};
      }
      ${FIXTURE_CSS}
    </style>
  </head>
  <body>
    <main class="kanagawa-fixture">
      <pre><code><span class="token comment">// TypeScript</span>
<span class="token keyword">export</span> <span class="token keyword">const</span> palette <span class="token punctuation">=</span> <span class="token function">defineTheme</span><span class="token punctuation">(</span><span class="token parameter">mode</span> <span class="token punctuation">=&gt;</span> <span class="token punctuation">{</span>
  <span class="token keyword">return</span> <span class="token string">"kanagawa"</span> <span class="token punctuation">+</span> <span class="token constant">3</span>
<span class="token punctuation">}</span><span class="token punctuation">)</span>

<span class="token comment">/* CSS */</span>
<span class="token keyword">.code</span> <span class="token punctuation">{</span> <span class="token parameter">color</span><span class="token punctuation">:</span> <span class="token string">var(--octc-syntax-token-function)</span><span class="token punctuation">;</span> <span class="token punctuation">}</span>

<span class="token comment">&lt;!-- HTML --&gt;</span>
<span class="token punctuation">&lt;</span><span class="token keyword">button</span> <span class="token parameter">data-theme</span><span class="token punctuation">=</span><span class="token string">"dragon"</span><span class="token punctuation">&gt;</span><span class="token function">Run</span><span class="token punctuation">&lt;/</span><span class="token keyword">button</span><span class="token punctuation">&gt;</span></code></pre>
    </main>
  </body>
</html>`,
    { waitUntil: "load" },
  );
}

async function collectColors(page: Page) {
  return page.locator(".kanagawa-fixture pre").evaluate((pre) => {
    const value = (selector: string) => {
      const node = pre.querySelector(selector);
      if (!(node instanceof HTMLElement)) {
        throw new Error(`Missing token ${selector}`);
      }
      return getComputedStyle(node).color;
    };

    return {
      blockBackground: getComputedStyle(pre).backgroundColor,
      comment: value(".token.comment"),
      constant: value(".token.constant"),
      function: value(".token.function"),
      keyword: value(".token.keyword"),
      parameter: value(".token.parameter"),
      punctuation: value(".token.punctuation"),
      string: value(".token.string"),
    };
  });
}
