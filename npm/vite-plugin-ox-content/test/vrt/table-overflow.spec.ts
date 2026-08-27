import { readFile } from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { expect, test } from "@playwright/test";
import { transformWithOxc } from "vite";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

const packageRoot = fileURLToPath(new URL("../..", import.meta.url));
const repoRoot = fileURLToPath(new URL("../../../..", import.meta.url));
const markdownTablesPath = path.join(packageRoot, "src/markdown-tables.ts");
const markdownTablesCssPath = path.join(
  repoRoot,
  "crates/ox_content_ssg/src/plugins/markdown-tables.css",
);

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

async function renderCustomHost(markdown: string) {
  const result = await transformMarkdown(
    markdown,
    "docs/vrt-custom-host-tables.md",
    createDocsResolvedOptions({ highlight: false }),
  );
  const tableCss = await readFile(markdownTablesCssPath, "utf8");
  const tableScript = await markdownTablesBrowserScript();

  return `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <style>
      body {
        margin: 0;
        background: rgb(250, 251, 252);
        color: rgb(17, 24, 39);
        font-family: Arial, sans-serif;
      }
      .content {
        width: min(832px, calc(100vw - 40px));
        margin: 24px auto;
        line-height: 1.6;
      }
      a { color: rgb(180, 40, 64); }
      blockquote {
        margin: 1rem 0;
        padding: 0.5rem 1rem;
        background: rgb(255, 246, 210);
      }
      th,
      td {
        border: 1px solid rgb(203, 213, 225);
        padding: 0.45rem 0.6rem;
        white-space: nowrap;
      }
      ${tableCss}
    </style>
  </head>
  <body>
    <article class="content">
      <p><a href="/custom">Host link</a></p>
      <blockquote>Host quote chrome</blockquote>
      ${result.html}
    </article>
    <script type="module">
      ${tableScript}
      globalThis.__oxEnhanceMarkdownTables(document, { label: "Scrollable table" });
    </script>
  </body>
</html>`;
}

let cachedTableScript: string | undefined;

async function markdownTablesBrowserScript(): Promise<string> {
  if (cachedTableScript) {
    return cachedTableScript;
  }
  const source = await readFile(markdownTablesPath, "utf8");
  const transformed = await transformWithOxc(
    `${source}\nglobalThis.__oxEnhanceMarkdownTables = enhanceMarkdownTables;`,
    markdownTablesPath,
    { lang: "ts" },
  );
  if (transformed.errors.length > 0) {
    throw new Error(transformed.errors.map((error) => error.message).join("\n"));
  }
  cachedTableScript = transformed.code;
  return cachedTableScript;
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

test("custom hosts can import isolated table styles without theme side effects", async ({
  page,
}) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.setContent(
    await renderCustomHost(
      [
        "# Tables",
        "",
        "| Short | Value |",
        "| --- | --- |",
        "| A | B |",
        "",
        "| Option | Type | Default |",
        "| --- | --- | --- |",
        "| `veryLongOptionNameWithoutBreaks` | `ExtremelyLongTypeNameWithoutBreaks` | `false` |",
      ].join("\n"),
    ),
    { waitUntil: "load" },
  );

  const metrics = await page.evaluate(() => {
    const content = document.querySelector(".content");
    const link = document.querySelector("a");
    const quote = document.querySelector("blockquote");
    const [narrow, wide] = Array.from(document.querySelectorAll("table"));
    if (
      !(content instanceof HTMLElement) ||
      !(link instanceof HTMLElement) ||
      !(quote instanceof HTMLElement) ||
      !(narrow instanceof HTMLElement) ||
      !(wide instanceof HTMLElement)
    ) {
      throw new Error("Missing custom-host targets");
    }
    return {
      bodyBackground: getComputedStyle(document.body).backgroundColor,
      bodyFontFamily: getComputedStyle(document.body).fontFamily,
      contentWidth: content.getBoundingClientRect().width,
      linkColor: getComputedStyle(link).color,
      quoteBackground: getComputedStyle(quote).backgroundColor,
      pageOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
      narrowTabIndex: narrow.getAttribute("tabindex"),
      narrowScrollable: narrow.getAttribute("data-ox-table-scrollable"),
      wideTabIndex: wide.getAttribute("tabindex"),
      wideLabel: wide.getAttribute("aria-label"),
      wideScrollable: wide.getAttribute("data-ox-table-scrollable"),
      wideClientWidth: wide.clientWidth,
      wideScrollWidth: wide.scrollWidth,
      wideRenderedWidth: wide.getBoundingClientRect().width,
    };
  });

  expect(metrics.bodyBackground).toBe("rgb(250, 251, 252)");
  expect(metrics.bodyFontFamily).toContain("Arial");
  expect(metrics.linkColor).toBe("rgb(180, 40, 64)");
  expect(metrics.quoteBackground).toBe("rgb(255, 246, 210)");
  expect(metrics.contentWidth).toBeCloseTo(350, 0);
  expect(metrics.pageOverflow).toBeLessThanOrEqual(0);
  expect(metrics.narrowTabIndex).toBeNull();
  expect(metrics.narrowScrollable).toBeNull();
  expect(metrics.wideTabIndex).toBe("0");
  expect(metrics.wideLabel).toBe("Scrollable table");
  expect(metrics.wideScrollable).toBe("");
  expect(metrics.wideScrollWidth).toBeGreaterThan(metrics.wideClientWidth);
  expect(metrics.wideRenderedWidth).toBeLessThanOrEqual(metrics.contentWidth + 1);
});
