import { expect, test } from "@playwright/test";
import type { BrowserContext, Page } from "@playwright/test";
import { importNapiModule } from "../../src/napi";
import { buildThemeNavItems, generateHtmlPage } from "../../src/ssg";
import type { SsgPageData } from "../../src/ssg";
import { resolveTheme } from "../../src/theme";
import type { SidebarItem } from "../../src/theme";

const origin = "http://page-layout.test";
const linkCount = 90;
const sidebar: SidebarItem[] = [
  {
    text: "Guide",
    items: Array.from({ length: linkCount }, (_, index) => ({
      text: `Chapter ${index + 1}`,
      link: `/page-${index + 1}.md`,
    })),
  },
];
const theme = resolveTheme({ sidebar });
const navigation = buildThemeNavItems(sidebar, "/", ".html");

// Sidebar links resolve `/page-1.md` to `/page-1/index.html`.
const url = (path: string) => `/${path}/index.html`;

function pageData(path: string, extra: Partial<SsgPageData> = {}): SsgPageData {
  return {
    title: path,
    content: `<h1>${path}</h1><p>Body</p>`,
    toc: [],
    frontmatter: {},
    path,
    href: url(path),
    ...extra,
  };
}

const render = (data: SsgPageData) =>
  generateHtmlPage(
    data,
    navigation,
    "Layout fixture",
    "/",
    undefined,
    theme,
    undefined,
    undefined,
    false,
    false,
    false,
    false,
    undefined,
    false,
    undefined,
    // `ssg.pageChrome`, so `sidebar: false` in page chrome flags applies.
    true,
  );

// A tall source image, like the 512x702 logo from the report.
const tallImage =
  '<svg xmlns="http://www.w3.org/2000/svg" width="512" height="702" viewBox="0 0 512 702"><rect width="512" height="702" fill="#c33"/></svg>';

/**
 * Serves production output: shared CSS/JS extracted to files, the core script
 * loaded with `defer` and held back so it cannot hide pre-paint work.
 */
async function serve(page: Page, sources: SsgPageData[]) {
  const generated = await Promise.all(
    sources.map(async (data) => ({
      inputPath: `${data.path}.md`,
      outputPath: `/site${url(data.path)}`,
      html: await render(data),
    })),
  );
  const { pages, assets } = (await importNapiModule()).externalizeSsgAssets(
    generated,
    "/site",
    "/",
  );
  const html = new Map(pages.map((item) => [item.outputPath.slice("/site".length), item.html]));
  const files = new Map(assets.map((asset) => [asset.publicPath, asset.content]));

  await page.route(`${origin}/**`, async (route) => {
    const { pathname } = new URL(route.request().url());
    if (pathname === "/tall.svg") {
      await route.fulfill({ contentType: "image/svg+xml", body: tallImage });
      return;
    }
    const asset = files.get(pathname);
    if (asset !== undefined) {
      if (pathname.endsWith(".js")) await new Promise((resolve) => setTimeout(resolve, 400));
      await route.fulfill({
        contentType: pathname.endsWith(".css") ? "text/css" : "text/javascript",
        body: asset,
      });
      return;
    }
    const body = html.get(pathname);
    await route.fulfill(
      body
        ? { contentType: "text/html", headers: { "cache-control": "no-store" }, body }
        : { status: 404, contentType: "text/plain", body: "not found" },
    );
  });
}

/** Records the sidebar scroll offset in the first frame that has a sidebar. */
async function probeFirstSidebarFrame(context: BrowserContext) {
  await context.addInitScript(() => {
    const probe = () => {
      const element = document.getElementById("ox-sidebar");
      if (!element) {
        requestAnimationFrame(probe);
        return;
      }
      (window as { firstSidebarScrollTop?: number }).firstSidebarScrollTop = element.scrollTop;
    };
    requestAnimationFrame(probe);
  });
}

async function firstFrame(page: Page): Promise<number> {
  const handle = await page.waitForFunction(() => {
    const value = (window as { firstSidebarScrollTop?: number }).firstSidebarScrollTop;
    return value === undefined ? undefined : { value };
  });
  return (await handle.jsonValue())!.value;
}

const linkVisibility = (page: Page, href: string) =>
  page.evaluate((target) => {
    const element = document.getElementById("ox-sidebar")!;
    const link = element.querySelector(`a[href="${target}"]`)!.getBoundingClientRect();
    const box = element.getBoundingClientRect();
    return { top: link.top - box.top, bottom: box.bottom - link.bottom };
  }, href);

const pagesFor = (indexes: number[]) => indexes.map((index) => pageData(`page-${index}`));

test.describe("sidebar restoration", () => {
  test.use({ viewport: { width: 1280, height: 720 } });

  test("paints the restored position in the first frame after navigation", async ({
    context,
    page,
  }) => {
    await probeFirstSidebarFrame(context);
    await serve(page, pagesFor([1, 85]));
    await page.goto(`${origin}${url("page-1")}`);
    await page.waitForLoadState("load");

    const saved = await page.evaluate(() => {
      const element = document.getElementById("ox-sidebar")!;
      element.scrollTop = element.scrollHeight;
      return element.scrollTop;
    });
    expect(saved).toBeGreaterThan(1000);
    await expect
      .poll(() => page.evaluate(() => sessionStorage.getItem("sidebarScroll")))
      .toBe(String(saved));

    await page.locator(`#ox-sidebar a[href="${url("page-85")}"]`).click();
    await expect(page).toHaveURL(`${origin}${url("page-85")}`);

    expect(await firstFrame(page)).toBe(saved);
    await page.waitForLoadState("load");
    expect(await page.locator("#ox-sidebar").evaluate((element) => element.scrollTop)).toBe(saved);
  });

  test("reveals the current page on a direct visit", async ({ context, page }) => {
    await probeFirstSidebarFrame(context);
    await serve(page, pagesFor([85]));
    await page.goto(`${origin}${url("page-85")}`);

    expect(await firstFrame(page)).toBeGreaterThan(0);
    const visible = await linkVisibility(page, url("page-85"));
    expect(visible.top).toBeGreaterThan(0);
    expect(visible.bottom).toBeGreaterThan(0);
  });

  test("reveals the current page when the saved position hides it", async ({ page }) => {
    await serve(page, pagesFor([1, 85]));
    await page.goto(`${origin}${url("page-1")}`);
    await page.evaluate(() => sessionStorage.setItem("sidebarScroll", "1"));
    await page.goto(`${origin}${url("page-85")}`);

    const visible = await linkVisibility(page, url("page-85"));
    expect(visible.top).toBeGreaterThan(0);
    expect(visible.bottom).toBeGreaterThan(0);
  });
});

test.describe("hidden sidebar", () => {
  test("starts the main column at the left edge", async ({ page }) => {
    await serve(page, [pageData("landing", { chrome: { sidebar: false } }), pageData("page-1")]);

    await page.goto(`${origin}${url("landing")}`);
    await expect(page.locator("body")).toHaveClass(/\box-no-sidebar\b/);
    await expect(page.locator("#ox-sidebar")).toHaveCount(0);
    expect(await page.locator(".main").evaluate((main) => main.getBoundingClientRect().left)).toBe(
      0,
    );

    await page.goto(`${origin}${url("page-1")}`);
    await expect(page.locator("body")).not.toHaveClass(/\box-no-sidebar\b/);
    expect(await page.locator(".main").evaluate((main) => main.getBoundingClientRect().left)).toBe(
      260,
    );
  });
});

test.describe("entry hero image", () => {
  const entry = (image: Record<string, unknown>) =>
    pageData("index", {
      entryPage: { hero: { name: "Fixture", image: { src: "/tall.svg", alt: "", ...image } } },
    });

  async function heroImageBox(page: Page, image: Record<string, unknown>) {
    await page.unrouteAll();
    await serve(page, [entry(image)]);
    await page.goto(`${origin}${url("index")}`);
    const img = page.locator(".hero-image img");
    await expect(img).toHaveJSProperty("complete", true);
    return img.evaluate((element) => {
      const box = element.getBoundingClientRect();
      return { width: Math.round(box.width), height: Math.round(box.height) };
    });
  }

  test("honors authored width and height", async ({ page }) => {
    expect(await heroImageBox(page, { width: 160, height: 220 })).toEqual({
      width: 160,
      height: 220,
    });
  });

  test("keeps the aspect ratio from a single authored dimension", async ({ page }) => {
    expect(await heroImageBox(page, { width: 160 })).toEqual({ width: 160, height: 219 });
    expect(await heroImageBox(page, { height: 220 })).toEqual({ width: 160, height: 220 });
  });

  test("keeps the natural size without authored dimensions", async ({ page }) => {
    expect(await heroImageBox(page, {})).toEqual({ width: 512, height: 702 });
  });
});
