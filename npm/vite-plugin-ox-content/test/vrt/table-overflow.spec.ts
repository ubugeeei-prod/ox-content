import { expect, test } from "@playwright/test";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

async function render(markdown: string) {
  const result = await transformMarkdown(
    markdown,
    "docs/vrt-table-overflow.md",
    createDocsResolvedOptions({ highlight: false }),
  );
  return generateHtmlPage(
    {
      title: "Tables",
      content: result.html,
      toc: result.toc,
      frontmatter: {},
      path: "/vrt-table-overflow",
      href: "/vrt-table-overflow/index.html",
    },
    [],
    "Ox Content",
    "/",
  );
}

test("narrow mobile tables do not draw an empty trailing column", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.setContent(
    await render(
      ["# Tables", "", "| A | B | C |", "| --- | --- | --- |", "| `x` | `y` | `z` |"].join("\n"),
    ),
    { waitUntil: "load" },
  );

  const metrics = await page.locator(".content table").evaluate((table) => {
    const lastCell = table.querySelector("tr:first-child > :last-child");
    const content = table.closest(".content");
    if (!(lastCell instanceof HTMLElement) || !(content instanceof HTMLElement)) {
      throw new Error("Missing table geometry targets");
    }
    const tableRect = table.getBoundingClientRect();
    const cellRect = lastCell.getBoundingClientRect();
    const contentRect = content.getBoundingClientRect();
    return {
      contentWidth: contentRect.width,
      tableWidth: tableRect.width,
      trailingGap: tableRect.right - cellRect.right,
    };
  });

  expect(metrics.tableWidth).toBeLessThan(metrics.contentWidth);
  expect(metrics.trailingGap).toBeLessThan(2);
  await expect(page.locator(".content table")).not.toHaveAttribute("tabindex", "0");
});

test("wide mobile tables keep horizontal overflow inside the content gutter", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.setContent(
    await render(
      [
        "# Tables",
        "",
        "| Option | Type | Default |",
        "| --- | --- | --- |",
        "| `veryLongOptionNameWithoutBreaks` | `ExtremelyLongTypeNameWithoutBreaks` | `false` |",
      ].join("\n"),
    ),
    { waitUntil: "load" },
  );

  const metrics = await page.locator(".content table").evaluate((table) => {
    const content = table.closest(".content");
    if (!(table instanceof HTMLElement) || !(content instanceof HTMLElement)) {
      throw new Error("Missing table geometry targets");
    }
    return {
      clientWidth: table.clientWidth,
      contentWidth: content.getBoundingClientRect().width,
      scrollWidth: table.scrollWidth,
      tableWidth: table.getBoundingClientRect().width,
    };
  });

  expect(metrics.tableWidth).toBeLessThanOrEqual(metrics.contentWidth + 1);
  expect(metrics.scrollWidth).toBeGreaterThan(metrics.clientWidth);
});

test("wide mobile tables are keyboard-scrollable when they overflow", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.setContent(
    await render(
      [
        "# Tables",
        "",
        "| Option | Type | Default |",
        "| --- | --- | --- |",
        "| `veryLongOptionNameWithoutBreaks` | `ExtremelyLongTypeNameWithoutBreaks` | `false` |",
      ].join("\n"),
    ),
    { waitUntil: "load" },
  );

  const table = page.locator(".content table");
  await expect(table).toHaveAttribute("tabindex", "0");
  await expect(table).toHaveAttribute("aria-label", "Scrollable table");
  await table.focus();
  await expect(table).toBeFocused();

  const before = await table.evaluate((node) => node.scrollLeft);
  await page.keyboard.press("ArrowRight");
  await expect.poll(() => table.evaluate((node) => node.scrollLeft)).toBeGreaterThan(before);
});
