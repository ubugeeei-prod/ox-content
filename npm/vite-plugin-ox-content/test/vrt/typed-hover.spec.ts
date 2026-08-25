import { expect, test } from "@playwright/test";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

test("typed hover remains interactive after syntax highlighting and reader chrome", async ({
  page,
}) => {
  const markdown = ["# Typed Hover", "", "```ts twoslash", "const value = 1;", "```"].join("\n");
  const result = await transformMarkdown(
    markdown,
    "docs/vrt-typed-hover.md",
    createDocsResolvedOptions({
      highlight: true,
      typedHover: { enabled: true, languages: ["ts", "tsx"] },
    }),
  );
  const html = await generateHtmlPage(
    {
      title: "Typed Hover",
      content: result.html,
      toc: result.toc,
      frontmatter: {},
      path: "/vrt-typed-hover",
      href: "/vrt-typed-hover/index.html",
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
  const token = page.locator(".ox-typed-hover-token", { hasText: "value" }).first();

  await expect(token).toBeVisible();
  await expect(page.locator(".ox-code .ox-typed-hover-token")).toHaveCount(1);

  await token.hover();
  await expect(page.locator(".ox-typed-hover-overlay")).toContainText("number");

  await token.focus();
  await expect(page.locator(".ox-typed-hover-overlay")).toBeVisible();

  await page.keyboard.press("Escape");
  await expect(page.locator(".ox-typed-hover-overlay")).toBeHidden();
});
