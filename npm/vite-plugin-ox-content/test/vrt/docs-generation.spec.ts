import { readFile } from "node:fs/promises";
import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import { generateMarkdown, resolveDocsOptions } from "../../src/docs";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import type { NavGroup } from "../../src/ssg";
import {
  createDocsFixture,
  createDocsResolvedOptions,
  createDocsSearchIndex,
} from "../fixtures/docs-fixture";

async function readFontDataUrl(fileName: string): Promise<string> {
  const font = await readFile(new URL(`./assets/${fileName}`, import.meta.url));
  return `data:font/woff2;base64,${font.toString("base64")}`;
}

async function installVrtFonts(page: Page) {
  const [sansFont, monoFont] = await Promise.all([
    readFontDataUrl("KaTeX_SansSerif-Regular.woff2"),
    readFontDataUrl("KaTeX_Typewriter-Regular.woff2"),
  ]);

  await page.addStyleTag({
    content: `
      @font-face {
        font-family: "OxContentVrtSans";
        src: url("${sansFont}") format("woff2");
        font-style: normal;
        font-weight: 400;
      }

      @font-face {
        font-family: "OxContentVrtMono";
        src: url("${monoFont}") format("woff2");
        font-style: normal;
        font-weight: 400;
      }

      :root {
        --octc-font-sans: "OxContentVrtSans", sans-serif;
        --octc-font-mono: "OxContentVrtMono", monospace;
      }
    `,
  });

  await page.evaluate(() => {
    document.documentElement.setAttribute("data-theme", "dark");
  });
  await page.waitForFunction(() => document.fonts.status === "loaded");
}

async function createDocsPages() {
  const docs = createDocsFixture();
  const docsOptions = resolveDocsOptions({
    githubUrl: "https://github.com/acme/ox-content",
  })!;
  const markdown = generateMarkdown(docs, docsOptions);
  const transformOptions = createDocsResolvedOptions();
  const navGroups: NavGroup[] = [
    {
      title: "API",
      items: [
        { title: "Overview", path: "/api/index", href: "/api/index" },
        { title: "Math", path: "/api/math", href: "/api/math" },
        { title: "Utils", path: "/api/utils", href: "/api/utils" },
      ],
    },
  ];

  async function renderDoc(fileName: string, pagePath: string, title: string) {
    const source = markdown[fileName];
    if (!source) {
      throw new Error(`Missing generated markdown for ${fileName}`);
    }

    const result = await transformMarkdown(source, `docs/${fileName}`, transformOptions);

    return generateHtmlPage(
      {
        title,
        content: result.html,
        toc: result.toc,
        frontmatter: {},
        path: pagePath,
        href: pagePath,
      },
      navGroups,
      "Ox Content",
      "/",
    );
  }

  return {
    indexHtml: await renderDoc("index.md", "/api/index", "API Documentation"),
    utilsHtml: await renderDoc("utils.md", "/api/utils", "utils.ts"),
  };
}

async function routeDocsSite(page: Page, pages: Awaited<ReturnType<typeof createDocsPages>>) {
  const searchIndex = createDocsSearchIndex();

  await page.route("http://docs.test/**", async (route) => {
    const url = new URL(route.request().url());

    if (url.pathname === "/api/index") {
      await route.fulfill({ contentType: "text/html", body: pages.indexHtml });
      return;
    }

    if (url.pathname === "/api/utils") {
      await route.fulfill({ contentType: "text/html", body: pages.utilsHtml });
      return;
    }

    if (url.pathname === "/search-index.json") {
      await route.fulfill({
        contentType: "application/json",
        body: JSON.stringify(searchIndex),
      });
      return;
    }

    await route.fulfill({ status: 404, body: "not found" });
  });
}

test.describe("docs generation VRT", () => {
  test("renders generated reference pages with overview and accordion details", async ({
    page,
  }) => {
    const pages = await createDocsPages();
    await routeDocsSite(page, pages);

    await page.goto("http://docs.test/api/utils");
    await installVrtFonts(page);
    await page.locator(".ox-api-entry summary").first().click();
    await page.locator(".ox-api-entry[open] .ox-api-entry__body").waitFor();

    await expect(page.locator(".content")).toHaveScreenshot("generated-docs-reference.png", {
      animations: "disabled",
      caret: "hide",
      maxDiffPixelRatio: 0.025,
      scale: "css",
    });
  });

  test("renders scoped search results for generated docs", async ({ page }) => {
    const pages = await createDocsPages();
    await routeDocsSite(page, pages);

    await page.goto("http://docs.test/api/index");
    await installVrtFonts(page);
    await page.locator(".search-button").click();
    await page.locator(".search-input").fill("@api capitalize");
    await page.locator(".search-result-scope").waitFor();

    await expect(page.locator(".search-modal")).toHaveScreenshot("generated-docs-search.png", {
      animations: "disabled",
      caret: "hide",
      maxDiffPixelRatio: 0.015,
      scale: "css",
    });
  });
});
