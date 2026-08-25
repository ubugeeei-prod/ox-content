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
  expect(metrics.iconWidth).toBeLessThanOrEqual(14);
  expect(metrics.iconHeight).toBeLessThanOrEqual(14);
  expect(metrics.opacity).toBeLessThan(0.9);
  expect(metrics.backgroundColor).not.toBe("rgb(255, 255, 255)");
});
