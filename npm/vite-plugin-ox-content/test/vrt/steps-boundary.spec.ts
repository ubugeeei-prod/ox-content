import { expect, test } from "@playwright/test";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

async function render(markdown: string) {
  const result = await transformMarkdown(
    markdown,
    "docs/vrt-steps-boundary.md",
    createDocsResolvedOptions({
      highlight: false,
      steps: { enabled: true },
    }),
  );

  return generateHtmlPage(
    {
      title: "Steps",
      content: result.html,
      toc: result.toc,
      frontmatter: {},
      path: "/vrt-steps-boundary",
      href: "/vrt-steps-boundary/index.html",
    },
    [],
    "Ox Content",
    "/",
  );
}

test("step previews keep a clear boundary before their source example", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.setContent(
    await render(
      [
        "# Steps",
        "",
        "::: steps",
        "",
        "1. Install the CLI",
        "",
        "   ```sh",
        "   npm i -g ox-content",
        "   ```",
        "",
        "2. Run **build**",
        "",
        ":::",
        "",
        "````md",
        "::: steps",
        "",
        "1. Install the CLI",
        "",
        "   ```sh",
        "   npm i -g ox-content",
        "   ```",
        "",
        "2. Run **build**",
        ":::",
        "````",
      ].join("\n"),
    ),
    { waitUntil: "load" },
  );

  const metrics = await page.locator(".content").evaluate((content) => {
    const steps = content.querySelector(".ox-steps");
    const source = content.querySelector(":scope > pre:last-of-type");
    if (!(steps instanceof HTMLElement) || !(source instanceof HTMLElement)) {
      throw new Error("Missing steps preview or source block");
    }

    const stepsRect = steps.getBoundingClientRect();
    const sourceRect = source.getBoundingClientRect();
    const stepsStyle = getComputedStyle(steps);
    const sourceText = source.innerText;

    return {
      borderTopWidth: Number.parseFloat(stepsStyle.borderTopWidth),
      gap: sourceRect.top - stepsRect.bottom,
      hasBackground: stepsStyle.backgroundColor !== "rgba(0, 0, 0, 0)",
      sourceText,
    };
  });

  expect(metrics.borderTopWidth).toBeGreaterThanOrEqual(1);
  expect(metrics.gap).toBeGreaterThan(20);
  expect(metrics.hasBackground).toBe(true);
  expect(metrics.sourceText).toContain("\n:::");
  expect(metrics.sourceText).not.toContain("   :::");
});
