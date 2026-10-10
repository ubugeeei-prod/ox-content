import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import { generateHtmlPage } from "../../src/ssg";
import type { NavGroup } from "../../src/ssg";
import { resolveTheme } from "../../src/theme";

const origin = "http://theme-toggle.test";

async function servePage(page: Page) {
  const navigation: NavGroup[] = [
    { title: "Guide", items: [{ title: "Alpha", path: "alpha", href: "/alpha.html" }] },
  ];
  const html = await generateHtmlPage(
    {
      title: "Alpha",
      content: "<h1>Alpha</h1><p>Toggle fixture</p>",
      toc: [],
      frontmatter: {},
      path: "alpha",
      href: "/alpha.html",
    },
    navigation,
    "Toggle fixture",
    "/",
    undefined,
    // Sites name the header so it stays still across navigation.
    resolveTheme({
      toggleTransition: "circle",
      css: ".header { view-transition-name: site-header; }",
    }),
  );
  await page.route(`${origin}/**`, (route) =>
    route.fulfill(
      new URL(route.request().url()).pathname === "/alpha.html"
        ? { contentType: "text/html", body: html }
        : { status: 404, contentType: "text/plain", body: "not found" },
    ),
  );
}

test("the circle reveal to light ends on the new theme and carries named elements", async ({
  page,
}) => {
  await servePage(page);
  await page.goto(`${origin}/alpha.html`);
  await page.evaluate(() => localStorage.setItem("theme", "dark"));
  await page.reload();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  await page.locator(".theme-toggle").click();
  const during = await page.evaluate(async () => {
    const root = document.documentElement;
    let clip: Animation | undefined;
    while (!clip) {
      await new Promise(requestAnimationFrame);
      clip = root
        .getAnimations({ subtree: true })
        .find(
          (animation) =>
            (animation.effect as KeyframeEffect | null)?.pseudoElement ===
            "::view-transition-old(root)",
        );
    }
    const headerName = getComputedStyle(document.querySelector(".header")!).viewTransitionName;
    // Jump to the end of the reveal: this is the frame before teardown.
    clip.finish();
    return {
      headerName,
      clipAtEnd: getComputedStyle(root, "::view-transition-old(root)").clipPath,
    };
  });

  expect(during.headerName).toBe("none");
  expect(during.clipAtEnd).toMatch(/^circle\(0px/);

  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  await expect(page.locator("html")).not.toHaveAttribute("data-ox-theme-transition");
  expect(
    await page.locator(".header").evaluate((header) => getComputedStyle(header).viewTransitionName),
  ).toBe("site-header");
});
