import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "@playwright/test";
import {
  applyReaderChromeHtml,
  renderReaderChromeAttributes,
  renderReaderChromeStyleTag,
} from "../../src/reader-chrome";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

const packageRoot = dirname(fileURLToPath(new URL("../../package.json", import.meta.url)));
const readerChromeRuntime = readFileSync(
  join(packageRoot, "../../crates/ox_content_ssg/src/html/reader_chrome_runtime.js"),
  "utf8",
);

test("code copy button stays compact on touch layouts", async ({ page }) => {
  const result = await transformMarkdown(
    ["# Copy", "", "```ts", "const value = 1;", "```"].join("\n"),
    "docs/vrt-copy-button.md",
    createDocsResolvedOptions({ highlight: true }),
  );
  const html = await generateHtmlPage(
    {
      title: "Copy",
      content: result.html,
      toc: result.toc,
      frontmatter: {},
      path: "/vrt-copy-button",
      href: "/vrt-copy-button/index.html",
    },
    [],
    "Ox Content",
    "/",
    undefined,
    undefined,
    undefined,
    undefined,
    false,
    { copy: true, externalLinks: false, backToTop: false },
  );

  await page.setViewportSize({ width: 390, height: 844 });
  await page.setContent(html, { waitUntil: "load" });

  const metrics = await page.locator(".ox-copy").evaluate((button) => {
    if (!(button instanceof HTMLElement)) {
      throw new Error("Missing copy button");
    }
    const style = getComputedStyle(button);
    const icon = getComputedStyle(button, "::before");
    const rect = button.getBoundingClientRect();
    return {
      backgroundColor: style.backgroundColor,
      buttonHeight: rect.height,
      buttonWidth: rect.width,
      iconHeight: Number.parseFloat(icon.height),
      iconWidth: Number.parseFloat(icon.width),
      opacity: Number.parseFloat(style.opacity),
    };
  });

  expect(metrics.buttonWidth).toBeLessThanOrEqual(28);
  expect(metrics.buttonHeight).toBeLessThanOrEqual(28);
  expect(metrics.iconWidth).toBeLessThanOrEqual(13);
  expect(metrics.iconHeight).toBeLessThanOrEqual(13);
  expect(metrics.opacity).toBeLessThanOrEqual(0.72);
  expect(metrics.backgroundColor).toBe("rgba(0, 0, 0, 0)");
});

test("code copy button preserves authored inline directives", async ({ page }) => {
  const markdown = [
    "# Copy",
    "",
    "```ts",
    "// [!code focus:2]",
    "const before = true;",
    "const after = false;",
    "console.log('old') // [!code --]",
    "console.log('new') // [!code ++]",
    "```",
  ].join("\n");
  const result = await transformMarkdown(
    markdown,
    "docs/vrt-copy-source.md",
    createDocsResolvedOptions({
      highlight: true,
      codeAnnotations: {
        enabled: true,
        notation: "vitepress",
        metaKey: "annotate",
        defaultLineNumbers: false,
      },
    }),
  );
  const html = await generateHtmlPage(
    {
      title: "Copy",
      content: result.html,
      toc: result.toc,
      frontmatter: {},
      path: "/vrt-copy-source",
      href: "/vrt-copy-source/index.html",
    },
    [],
    "Ox Content",
    "/",
    undefined,
    undefined,
    undefined,
    undefined,
    false,
    { copy: true, externalLinks: false, backToTop: false },
  );

  await page.setContent(html, { waitUntil: "load" });
  await page.evaluate(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText(value: string) {
          (window as typeof window & { __oxCopied?: string }).__oxCopied = value;
          return Promise.resolve();
        },
      },
    });
  });

  await page.locator(".ox-copy").focus();
  await page.keyboard.press("Enter");

  const authoredSource = `${markdown.split("\n").slice(3, -1).join("\n")}\n`;
  await expect
    .poll(() => page.evaluate(() => (window as typeof window & { __oxCopied?: string }).__oxCopied))
    .toBe(authoredSource);
});

test.describe("custom host reader chrome touch layout", () => {
  test.use({ hasTouch: true, isMobile: true, viewport: { width: 390, height: 844 } });

  test("stays stable across prose styles and repeated init", async ({ page }) => {
    const markdown = [
      "# Copy",
      "",
      "```ts",
      "const value = 1;",
      "```",
      "",
      "```ts",
      "value + 1;",
      "```",
    ].join("\n");
    const result = await transformMarkdown(
      markdown,
      "docs/vrt-custom-copy.md",
      createDocsResolvedOptions({ highlight: true }),
    );
    const article = applyReaderChromeHtml(result.html, {
      copy: true,
      externalLinks: false,
      backToTop: false,
    });

    await page.setContent(
      `<!DOCTYPE html>
<html>
<head>
  <style>
    :root {
      --octc-color-primary: #4f6fae;
      --octc-color-text: #131a30;
      --octc-color-bg: #ffffff;
      --octc-color-bg-alt: #f5f7fb;
      --octc-color-code-text: #e5e9f0;
      --octc-color-code-bg: #121a2f;
      --octc-color-code-frame-border: #44506a;
    }
    .content { max-width: 680px; margin: 0 auto; }
    .content pre { margin: 2rem 0; padding: 3rem; background: var(--octc-color-code-bg); color: var(--octc-color-code-text); }
    .content button { width: 14rem; min-height: 5rem; padding: 2rem; font-size: 3rem; }
  </style>
  ${renderReaderChromeStyleTag({ copy: true, externalLinks: false, backToTop: false })}
</head>
<body>
  <article class="content"${renderReaderChromeAttributes({ copy: true, externalLinks: false, backToTop: false })}>${article}</article>
  <script>${readerChromeRuntime}
window.__initReaderChrome = initReaderChrome;</script>
</body>
</html>`,
      { waitUntil: "load" },
    );

    await page.evaluate(() => {
      const calls: string[] = [];
      Object.defineProperty(navigator, "clipboard", {
        configurable: true,
        value: {
          writeText(value: string) {
            calls.push(value);
            return Promise.resolve();
          },
        },
      });
      const root = document.querySelector("article");
      const win = window as typeof window & {
        __copyCalls?: string[];
        __initReaderChrome?: (root?: Document | Element) => void;
      };
      win.__copyCalls = calls;
      win.__initReaderChrome?.(document);
      win.__initReaderChrome?.(root ?? undefined);
      win.__initReaderChrome?.(root ?? undefined);
    });

    await expect(page.locator(".ox-copy")).toHaveCount(2);
    const metrics = await page
      .locator(".ox-copy")
      .first()
      .evaluate((button) => {
        if (!(button instanceof HTMLElement)) throw new Error("Missing copy button");
        const rect = button.getBoundingClientRect();
        return {
          height: rect.height,
          width: rect.width,
          opacity: Number.parseFloat(getComputedStyle(button).opacity),
        };
      });
    expect(metrics.width).toBeLessThanOrEqual(28);
    expect(metrics.height).toBeLessThanOrEqual(28);
    expect(metrics.opacity).toBeLessThanOrEqual(0.72);

    await page.locator(".ox-copy").first().click();

    await expect
      .poll(() =>
        page.evaluate(() => (window as typeof window & { __copyCalls?: string[] }).__copyCalls),
      )
      .toEqual(["const value = 1;\n"]);
  });
});
