import { expect, test } from "@playwright/test";
import { transformAllPlugins } from "../../src/plugins";
import { resetTabGroupCounter } from "../../src/plugins/tabs";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

async function render(markdown: string) {
  resetTabGroupCounter();
  const result = await transformMarkdown(
    markdown,
    "docs/vrt-tabs-polish.md",
    createDocsResolvedOptions({
      embeds: {
        github: false,
        openGraph: false,
        pm: true,
        spotify: false,
        stackBlitz: false,
        twitter: false,
        bluesky: false,
        webContainer: false,
      },
      highlight: false,
    }),
  );
  const content = await transformAllPlugins(result.html, {
    github: false,
    openGraph: false,
    pm: true,
    spotify: false,
    stackBlitz: false,
    twitter: false,
    bluesky: false,
    webContainer: false,
  });

  return generateHtmlPage(
    {
      title: "Tabs",
      content,
      toc: result.toc,
      frontmatter: {},
      path: "/vrt-tabs-polish",
      href: "/vrt-tabs-polish/index.html",
    },
    [],
    "Ox Content",
    "/",
  );
}

test("package-manager tabs render code as an integrated panel on mobile", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.setContent(await render("<pm>npm install -D @ox-content/vite-plugin</pm>"), {
    waitUntil: "load",
  });

  const metrics = await page
    .locator(".ox-tabs")
    .first()
    .evaluate((tabs) => {
      const panel = tabs.querySelector(".ox-tab-panel[data-tab='0']");
      const pre = panel?.querySelector("pre");
      const header = tabs.querySelector(".ox-tabs-header");
      if (
        !(tabs instanceof HTMLElement) ||
        !(panel instanceof HTMLElement) ||
        !(pre instanceof HTMLElement)
      ) {
        throw new Error("Missing tabs geometry targets");
      }
      const tabsRect = tabs.getBoundingClientRect();
      const panelRect = panel.getBoundingClientRect();
      const preRect = pre.getBoundingClientRect();
      const headerRect = header?.getBoundingClientRect();
      const panelStyle = getComputedStyle(panel);
      const preStyle = getComputedStyle(pre);
      return {
        headerBottom: headerRect?.bottom ?? 0,
        panelPaddingTop: Number.parseFloat(panelStyle.paddingTop),
        preBorderRadius: Number.parseFloat(preStyle.borderTopLeftRadius),
        preLeftGap: preRect.left - tabsRect.left,
        preTopGap: preRect.top - panelRect.top,
        tabsWidth: tabsRect.width,
      };
    });

  expect(metrics.tabsWidth).toBeLessThanOrEqual(390);
  expect(metrics.panelPaddingTop).toBe(0);
  expect(metrics.preBorderRadius).toBe(0);
  expect(metrics.preLeftGap).toBeLessThan(2);
  expect(metrics.preTopGap).toBeLessThan(2);
  expect(metrics.headerBottom).toBeGreaterThan(0);
});

test("generic tabs keep a long header scrollable without widening prose", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.setContent(
    await render(
      [
        "<tabs>",
        '  <tab label="Vite config"><pre><code>plugins: [oxContent({ srcDir: "content" })]</code></pre></tab>',
        '  <tab label="Markdown source"><pre><code># Hello</code></pre></tab>',
        '  <tab label="Static build"><pre><code>pnpm vite build</code></pre></tab>',
        '  <tab label="Preview route"><pre><code>/docs/getting-started</code></pre></tab>',
        '  <tab label="Deploy target"><pre><code>dist/client</code></pre></tab>',
        "</tabs>",
      ].join("\n"),
    ),
    { waitUntil: "load" },
  );

  const metrics = await page
    .locator(".ox-tabs")
    .first()
    .evaluate((tabs) => {
      const content = tabs.closest(".content");
      const header = tabs.querySelector(".ox-tabs-header");
      if (
        !(tabs instanceof HTMLElement) ||
        !(content instanceof HTMLElement) ||
        !(header instanceof HTMLElement)
      ) {
        throw new Error("Missing tabs header targets");
      }
      return {
        contentWidth: content.getBoundingClientRect().width,
        headerClientWidth: header.clientWidth,
        headerScrollWidth: header.scrollWidth,
        tabsWidth: tabs.getBoundingClientRect().width,
      };
    });

  expect(metrics.tabsWidth).toBeLessThanOrEqual(metrics.contentWidth + 1);
  expect(metrics.headerScrollWidth).toBeGreaterThan(metrics.headerClientWidth);
});
