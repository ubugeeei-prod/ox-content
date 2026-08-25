import { expect, test } from "@playwright/test";
import type { Page, TestInfo } from "@playwright/test";
import { localizeNavGroups } from "../../src/locale-nav";
import { buildThemeNavItems, generateHtmlPage } from "../../src/ssg";
import { resolveTheme } from "../../src/theme";
import type { SidebarItem } from "../../src/theme";

const origin = "http://sidebar-i18n.test";
const sidebar: SidebarItem[] = [
  {
    text: { en: "Guide", ja: "ガイド" },
    collapsed: true,
    stickyCollapsed: true,
    items: [
      {
        text: { en: "Built-in features", ja: "組み込み機能" },
        link: "/built-in.md",
        items: [{ text: { en: "Cards", ja: "カード" }, link: "/cards.md" }],
      },
    ],
  },
];

const longSidebar: SidebarItem[] = [
  {
    text: "Guide",
    collapsed: false,
    stickyCollapsed: true,
    items: Array.from({ length: 34 }, (_, index) => ({
      text: `Navigation item ${String(index + 1).padStart(2, "0")}`,
      link: `/long-${index + 1}.md`,
    })),
  },
];

const pageRefs = [
  { path: "built-in", href: "/built-in.html" },
  { path: "cards", href: "/cards.html" },
  { path: "ja/built-in", href: "/ja/built-in.html" },
  { path: "ja/cards", href: "/ja/cards.html" },
];

async function render(locale: "en" | "ja") {
  const localized = localizeNavGroups(buildThemeNavItems(sidebar, "/", ".html"), {
    locale,
    locales: [{ code: "en" }, { code: "ja" }],
    defaultLocale: "en",
    hideDefaultLocale: true,
    pages: pageRefs,
    base: "/",
  });
  const path = locale === "ja" ? "ja/built-in" : "built-in";
  return generateHtmlPage(
    {
      title: locale === "ja" ? "組み込み機能" : "Built-in features",
      content: `<h1>${locale === "ja" ? "組み込み機能" : "Built-in features"}</h1>`,
      toc: [],
      frontmatter: {},
      path,
      href: `/${path}.html`,
    },
    localized,
    "Sidebar i18n fixture",
    "/",
    undefined,
    resolveTheme({ sidebar }),
    locale,
    [
      { code: "en", name: "English" },
      { code: "ja", name: "日本語" },
    ],
  );
}

async function renderLongMenu() {
  const path = "long-1";
  return generateHtmlPage(
    {
      title: "Long navigation",
      content:
        '<h1>Long navigation</h1><p>Mobile navigation stress fixture.</p><div style="height:1600px"></div>',
      toc: [],
      frontmatter: {},
      path,
      href: `/${path}.html`,
    },
    buildThemeNavItems(longSidebar, "/", ".html"),
    "Long menu fixture",
    "/",
    undefined,
    resolveTheme({ sidebar: longSidebar }),
  );
}

async function routePages(page: Page) {
  const pages = {
    "/built-in.html": await render("en"),
    "/ja/built-in.html": await render("ja"),
  };
  await page.route(`${origin}/**`, async (route) => {
    const pathname = new URL(route.request().url()).pathname;
    const body = pages[pathname as keyof typeof pages];
    await route.fulfill(
      body
        ? { contentType: "text/html", body }
        : { status: 404, contentType: "text/plain", body: "not found" },
    );
  });
}

async function routeLongMenuPage(page: Page) {
  const html = await renderLongMenu();
  await page.route(`${origin}/**`, async (route) => {
    const pathname = new URL(route.request().url()).pathname;
    await route.fulfill(
      pathname === "/long-1.html"
        ? { contentType: "text/html", body: html }
        : { status: 404, contentType: "text/plain", body: "not found" },
    );
  });
}

async function openMobileMenu(page: Page) {
  if (page.viewportSize()!.width >= 768) return;
  const menu = page.locator("[data-mobile-menu]");
  await menu.focus();
  await page.keyboard.press("Enter");
  await expect(page.locator("#ox-sidebar")).toHaveClass(/open/);
}

async function attachScreenshot(page: Page, testInfo: TestInfo, name: string) {
  await testInfo.attach(name, {
    body: await page.screenshot({ animations: "disabled", fullPage: true }),
    contentType: "image/png",
  });
}

for (const viewport of [
  { name: "desktop", width: 1280, height: 800 },
  { name: "mobile", width: 390, height: 844 },
] as const) {
  test(`localizes nested labels and preserves keyboard collapse state on ${viewport.name}`, async ({
    page,
  }, testInfo) => {
    await page.setViewportSize(viewport);
    await routePages(page);
    await page.goto(`${origin}/built-in.html`);
    await openMobileMenu(page);

    const details = page.locator('details[data-ox-nav-state-key="group:0"]');
    const summary = details.locator(":scope > summary");
    await expect(summary).toHaveText("Guide");
    await summary.focus();
    await page.keyboard.press("Enter");
    await expect(details).toHaveAttribute("open", "");
    await expect
      .poll(() => page.evaluate(() => localStorage.getItem("ox-content:nav:/:group:0")))
      .toBe("open");
    await attachScreenshot(page, testInfo, `${viewport.name}-sidebar-en.png`);

    await page.goto(`${origin}/ja/built-in.html`);
    await openMobileMenu(page);
    const japaneseDetails = page.locator('details[data-ox-nav-state-key="group:0"]');
    await expect(japaneseDetails).toHaveAttribute("open", "");
    await expect(japaneseDetails.locator(":scope > summary")).toHaveText("ガイド");
    await expect(japaneseDetails).toContainText("組み込み機能");
    await expect(japaneseDetails).toContainText("カード");
    await expect(page.locator('a[href="/ja/built-in.html"]')).toHaveClass(/active/);
    await attachScreenshot(page, testInfo, `${viewport.name}-sidebar-ja.png`);
  });
}

test("contains a long mobile menu between the fixed header and footer", async ({
  page,
}, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await routeLongMenuPage(page);
  await page.goto(`${origin}/long-1.html`);
  await openMobileMenu(page);

  const boxes = await page.evaluate(() => {
    const sidebar = document.querySelector<HTMLElement>("#ox-sidebar");
    const header = document.querySelector<HTMLElement>(".header");
    const footer = document.querySelector<HTMLElement>(".mobile-footer");
    const overlay = document.querySelector<HTMLElement>(".overlay");
    if (!sidebar || !header || !footer || !overlay) throw new Error("missing mobile chrome");

    const sidebarBox = sidebar.getBoundingClientRect();
    const headerBox = header.getBoundingClientRect();
    const footerBox = footer.getBoundingClientRect();
    const overlayBox = overlay.getBoundingClientRect();
    const overlayStyle = getComputedStyle(overlay);
    return {
      headerBottom: headerBox.bottom,
      footerTop: footerBox.top,
      overlayBackground: overlayStyle.backgroundColor,
      overlayBottom: overlayBox.bottom,
      overlayTop: overlayBox.top,
      sidebarBottom: sidebarBox.bottom,
      sidebarClientHeight: sidebar.clientHeight,
      sidebarScrollHeight: sidebar.scrollHeight,
      sidebarTop: sidebarBox.top,
    };
  });

  expect(boxes.sidebarTop).toBeGreaterThanOrEqual(Math.floor(boxes.headerBottom) - 1);
  expect(boxes.sidebarBottom).toBeLessThanOrEqual(Math.ceil(boxes.footerTop) + 1);
  expect(boxes.sidebarScrollHeight).toBeGreaterThan(boxes.sidebarClientHeight);
  expect(boxes.overlayTop).toBeGreaterThanOrEqual(Math.floor(boxes.headerBottom) - 1);
  expect(boxes.overlayBottom).toBeLessThanOrEqual(Math.ceil(boxes.footerTop) + 1);
  expect(boxes.overlayBackground).not.toBe("rgba(0, 0, 0, 0)");

  const bodyScrollBefore = await page.evaluate(() => window.scrollY);
  await page.locator("#ox-sidebar").hover();
  await page.mouse.wheel(0, 900);
  await expect.poll(() => page.evaluate(() => window.scrollY)).toBe(bodyScrollBefore);
  await expect
    .poll(() => page.locator("#ox-sidebar").evaluate((element) => element.scrollTop))
    .toBeGreaterThan(0);
  await attachScreenshot(page, testInfo, "mobile-long-sidebar-contained.png");
});
