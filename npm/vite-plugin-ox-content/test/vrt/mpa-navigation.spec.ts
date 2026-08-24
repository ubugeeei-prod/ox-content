import { expect, test } from "@playwright/test";
import type { BrowserContext, Page, TestInfo } from "@playwright/test";
import { generateHtmlPage } from "../../src/ssg";
import { resolveTheme } from "../../src/theme";
import type { NavGroup } from "../../src/ssg";

const origin = "http://docs.test";

async function buildPages(viewTransitions = true) {
  const navigation: NavGroup[] = [
    {
      title: "Guide",
      items: [
        { title: "Alpha", path: "alpha", href: "/alpha.html" },
        { title: "Beta", path: "beta", href: "/beta.html" },
      ],
    },
  ];
  const theme = resolveTheme({
    viewTransitions,
    embed: { head: '<link rel="stylesheet" href="/delayed.css">' },
  });

  const render = (title: string, path: string, content: string) =>
    generateHtmlPage(
      {
        title,
        content: `<h1>${title}</h1><p>${content}</p>`,
        toc: [],
        frontmatter: {},
        path,
        href: `/${path}.html`,
      },
      navigation,
      "Transition fixture",
      "/",
      undefined,
      theme,
    );

  return {
    "/alpha.html": await render("Alpha", "alpha", "First generated page"),
    "/beta.html": await render("Beta", "beta", "Second generated page"),
  };
}

async function installRevealProbe(context: BrowserContext) {
  await context.addInitScript(() => {
    addEventListener("pagereveal", (event) => {
      try {
        sessionStorage.setItem(
          "ox-content:last-pagereveal-transition",
          String(Boolean((event as PageRevealEvent).viewTransition)),
        );
      } catch {
        // A failed probe must not change navigation behavior.
      }
    });
  });
}

async function routeFixture(page: Page, pages: Awaited<ReturnType<typeof buildPages>>) {
  await page.route(`${origin}/**`, async (route) => {
    const url = new URL(route.request().url());

    if (url.pathname === "/delayed.css") {
      await new Promise((resolve) => setTimeout(resolve, 120));
      await route.fulfill({
        contentType: "text/css",
        headers: { "cache-control": "no-store" },
        body: "html { --fixture-stylesheet-loaded: 1; }",
      });
      return;
    }

    const body = pages[url.pathname as keyof typeof pages];
    if (body) {
      await route.fulfill({
        contentType: "text/html",
        headers: { "cache-control": "no-store" },
        body,
      });
      return;
    }

    await route.fulfill({ status: 404, body: "not found" });
  });
}

async function setStoredTheme(page: Page, theme: "light" | "dark" | "system") {
  await page.evaluate((value) => localStorage.setItem("theme", value), theme);
}

async function expectTheme(page: Page, theme: "light" | "dark" | "system") {
  if (theme === "system") {
    await expect(page.locator("html")).not.toHaveAttribute("data-theme");
  } else {
    await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
  }
}

async function attachFrame(page: Page, testInfo: TestInfo, name: string) {
  await testInfo.attach(name, {
    body: await page.screenshot({ animations: "disabled", fullPage: true }),
    contentType: "image/png",
  });
}

test.describe("built-in theme MPA navigation", () => {
  test("keeps dark and light preferences through links and browser history", async ({
    context,
    page,
  }) => {
    await installRevealProbe(context);
    await routeFixture(page, await buildPages());
    await page.goto(`${origin}/alpha.html`);

    for (const theme of ["dark", "light"] as const) {
      await setStoredTheme(page, theme);
      await page.reload();
      await expectTheme(page, theme);

      await page.locator('a[href="/beta.html"]').click();
      await expect(page).toHaveURL(`${origin}/beta.html`);
      await expectTheme(page, theme);
      await expect
        .poll(() =>
          page.evaluate(() => sessionStorage.getItem("ox-content:last-pagereveal-transition")),
        )
        .toBe("true");

      await page.goBack();
      await expect(page).toHaveURL(`${origin}/alpha.html`);
      await expectTheme(page, theme);
      await page.goForward();
      await expect(page).toHaveURL(`${origin}/beta.html`);
      await expectTheme(page, theme);
      await page.goto(`${origin}/alpha.html`);
    }
  });

  test("uses the live system scheme without pinning a data attribute", async ({ page }) => {
    await routeFixture(page, await buildPages());
    await page.emulateMedia({ colorScheme: "dark" });
    await page.goto(`${origin}/alpha.html`);
    await setStoredTheme(page, "system");
    await page.reload();

    await expectTheme(page, "system");
    await expect
      .poll(() => page.evaluate(() => getComputedStyle(document.body).backgroundColor))
      .toBe("rgb(6, 8, 22)");

    await page.emulateMedia({ colorScheme: "light" });
    await expect
      .poll(() => page.evaluate(() => getComputedStyle(document.body).backgroundColor))
      .toBe("rgb(255, 255, 255)");
  });

  test("disables cross-document motion for reduced motion and explicit opt-out", async ({
    context,
    page,
  }) => {
    await installRevealProbe(context);
    await page.emulateMedia({ reducedMotion: "reduce" });
    await routeFixture(page, await buildPages());
    await page.goto(`${origin}/alpha.html`);
    await page.locator('a[href="/beta.html"]').click();
    await expect(page).toHaveURL(`${origin}/beta.html`);
    await expect
      .poll(() =>
        page.evaluate(() => sessionStorage.getItem("ox-content:last-pagereveal-transition")),
      )
      .toBe("false");

    await page.unrouteAll({ behavior: "wait" });
    await page.emulateMedia({ reducedMotion: "no-preference" });
    await routeFixture(page, await buildPages(false));
    await page.goto(`${origin}/alpha.html`);
    await page.locator('a[href="/beta.html"]').click();
    await expect(page).toHaveURL(`${origin}/beta.html`);
    await expect
      .poll(() =>
        page.evaluate(() => sessionStorage.getItem("ox-content:last-pagereveal-transition")),
      )
      .toBe("false");
  });

  test("captures stable desktop and mobile frames after a delayed stylesheet", async ({
    context,
    page,
  }, testInfo) => {
    await installRevealProbe(context);
    await routeFixture(page, await buildPages());
    await page.goto(`${origin}/alpha.html`);
    await setStoredTheme(page, "dark");
    await page.reload();
    await page.locator('a[href="/beta.html"]').click();
    await expect(page).toHaveURL(`${origin}/beta.html`);
    await expectTheme(page, "dark");
    await expect
      .poll(() =>
        page.evaluate(() =>
          getComputedStyle(document.documentElement).getPropertyValue(
            "--fixture-stylesheet-loaded",
          ),
        ),
      )
      .toBe("1");
    await attachFrame(page, testInfo, "mpa-navigation-desktop.png");

    await page.setViewportSize({ width: 390, height: 844 });
    await page.goBack();
    await expect(page).toHaveURL(`${origin}/alpha.html`);
    await expectTheme(page, "dark");
    await attachFrame(page, testInfo, "mpa-navigation-mobile.png");
  });
});
