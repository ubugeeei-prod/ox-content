import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import { paper } from "../../../theme/paper/src/index";
import { receipt } from "../../../theme/receipt/src/index";
import { editorial } from "../../../theme/editorial/src/index";
import { blurGlass } from "../../../theme/blur-glass/src/index";
import { liquidGlass } from "../../../theme/liquid-glass/src/index";
import { tokyoNight } from "../../../theme-color/tokyo-night/src/index";
import { generateHtmlPage } from "../../src/ssg";
import { resolveTheme, type ThemeConfig } from "../../src/theme";

const content = `
<h1>Theme Polish</h1>
<p><a href="#spacing">Readable links</a> and prose.</p>
<h2 id="spacing">Spacing</h2>
<p>Code blocks need enough internal spacing.</p>
<pre><code>const compact = true;
console.log(compact);</code></pre>
<h2 id="second">Second</h2>
<p>Right outline spacing should not overlap its own rule.</p>
`;

const toc = [
  { depth: 2, text: "Spacing", slug: "spacing" },
  { depth: 2, text: "Second", slug: "second" },
];

async function render(theme: ThemeConfig | ThemeConfig[]) {
  const layers = Array.isArray(theme) ? theme : [theme];
  return generateHtmlPage(
    {
      title: "Theme Polish",
      content,
      toc,
      frontmatter: {},
      path: "/theme-polish",
      href: "/theme-polish/index.html",
    },
    [{ title: "Guide", items: [{ title: "Theme Polish", path: "/theme-polish", href: "." }] }],
    "Ox Content",
    "/",
    undefined,
    resolveTheme([...layers, { aside: true }]),
  );
}

async function headerBackdropFilters(page: Page) {
  return page.locator(".header").evaluate((header) => {
    const style = getComputedStyle(header);
    return {
      backdrop: style.backdropFilter,
      webkitBackdrop: style.getPropertyValue("-webkit-backdrop-filter"),
    };
  });
}

test("theme hover states keep text posture stable", async ({ page }) => {
  await page.setViewportSize({ width: 1520, height: 900 });
  await page.setContent(await render([editorial, tokyoNight]), { waitUntil: "load" });

  const targets = [".nav-link", ".toc-link", ".content a"] as const;
  for (const selector of targets) {
    const locator = page.locator(selector).first();
    await locator.hover();
    await expect(locator).toHaveCSS("font-style", "normal");
  }
});

test("glass-like theme headers do not blur page content", async ({ page }) => {
  for (const theme of [blurGlass, liquidGlass]) {
    await page.setContent(await render([theme, tokyoNight]), { waitUntil: "load" });
    const filters = await headerBackdropFilters(page);
    expect(filters.backdrop).toBe("none");
    expect(["", "none"]).toContain(filters.webkitBackdrop);
  }
});

test("paper and receipt code blocks stay flat and readable", async ({ page }) => {
  await page.setContent(await render([paper, tokyoNight]), { waitUntil: "load" });

  const paperPre = page.locator(".content pre").first();
  await expect(paperPre).toHaveCSS("box-shadow", "none");
  await expect(paperPre).toHaveCSS("background-image", "none");

  await page.setContent(await render([receipt, tokyoNight]), { waitUntil: "load" });

  const receiptPadding = await page
    .locator(".content pre")
    .first()
    .evaluate((pre) => {
      const style = getComputedStyle(pre);
      return {
        left: Number.parseFloat(style.paddingLeft),
        right: Number.parseFloat(style.paddingRight),
      };
    });
  expect(receiptPadding.left).toBeGreaterThanOrEqual(12);
  expect(receiptPadding.right).toBeGreaterThanOrEqual(12);
});

test("right toc reserves space between the rule and links", async ({ page }) => {
  await page.setViewportSize({ width: 1520, height: 900 });
  await page.setContent(await render([paper, tokyoNight]), { waitUntil: "load" });

  const metrics = await page.locator(".toc").evaluate((tocElement) => {
    const tocRect = tocElement.getBoundingClientRect();
    const firstLink = tocElement.querySelector(".toc-link");
    if (!(firstLink instanceof HTMLElement)) {
      throw new Error("missing toc link");
    }
    const linkRect = firstLink.getBoundingClientRect();
    return {
      tocWidth: tocRect.width,
      ruleGap: linkRect.left - tocRect.left,
    };
  });

  expect(metrics.tocWidth).toBeGreaterThanOrEqual(260);
  expect(metrics.ruleGap).toBeGreaterThanOrEqual(20);
});
