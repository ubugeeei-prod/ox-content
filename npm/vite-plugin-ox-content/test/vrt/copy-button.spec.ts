import { expect, test } from "@playwright/test";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

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
