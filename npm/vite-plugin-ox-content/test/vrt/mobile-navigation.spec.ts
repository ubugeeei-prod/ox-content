import { expect, test } from "@playwright/test";
import * as fs from "node:fs/promises";
import * as os from "node:os";
import * as path from "node:path";
import { buildSsg } from "../../src/ssg";
import { resolveOptions } from "../../src/resolve-options";

const origin = "http://mobile-navigation.test";

for (const width of [320, 390, 768]) {
  test(`prioritizes navigation, search and language at ${width}px`, async ({ page }, testInfo) => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "ox-mobile-navigation-"));
    try {
      await fs.mkdir(path.join(root, "content", "ja"), { recursive: true });
      for (const [file, title] of [
        ["guide.md", "Guide"],
        ["ja/guide.md", "ガイド"],
      ]) {
        await fs.writeFile(
          path.join(root, "content", file),
          `# ${title}\n\n${"Reading content.\n\n".repeat(60)}`,
        );
      }
      const result = await buildSsg(
        resolveOptions({
          srcDir: "content",
          docs: false,
          search: false,
          ogViewer: false,
          i18n: {
            enabled: true,
            defaultLocale: "en",
            hideDefaultLocale: true,
            locales: [
              { code: "en", name: "English" },
              { code: "ja", name: "日本語" },
            ],
          },
          ssg: {
            localeSwitcher: true,
            theme: { socialLinks: { github: "https://github.com/ox-content" } },
          },
        }),
        root,
      );
      expect(result.errors).toEqual([]);
      await page.route(`${origin}/**`, async (route) => {
        const pathname = new URL(route.request().url()).pathname;
        try {
          const body = await fs.readFile(
            path.join(root, "dist", pathname.endsWith("/") ? `${pathname}index.html` : pathname),
          );
          await route.fulfill({
            body,
            contentType: pathname.endsWith(".css")
              ? "text/css"
              : pathname.endsWith(".js")
                ? "text/javascript"
                : "text/html",
          });
        } catch {
          await route.fulfill({ status: 404, body: "not found" });
        }
      });
      await page.setViewportSize({ width, height: 844 });
      await page.goto(`${origin}/guide/index.html`);
      const bar = page.locator(".mobile-footer");
      await expect(bar).toBeVisible();
      await expect(bar.getByRole("button", { name: "Menu", exact: true })).toBeVisible();
      await expect(bar.getByRole("button", { name: "Search", exact: true })).toBeVisible();
      await expect(bar.locator("[data-mobile-theme], a[target='_blank']")).toHaveCount(0);
      const language = bar.getByRole("button", { name: "Change language" });
      await expect(language).toContainText("English");
      await language.click();
      const japanese = bar.getByRole("link", { name: "日本語", exact: true });
      await expect(japanese).toBeVisible();
      const bounds = await japanese.boundingBox();
      expect(bounds?.height).toBeGreaterThanOrEqual(44);
      expect(bounds!.x).toBeGreaterThanOrEqual(0);
      expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(width);
      await page.keyboard.press("Escape");
      await expect(language).toBeFocused();
      await expect(japanese).toBeHidden();
      await language.click();
      await testInfo.attach(`mobile-language-${width}.png`, {
        body: await page.screenshot({ path: testInfo.outputPath(`mobile-language-${width}.png`) }),
        contentType: "image/png",
      });
      await japanese.click();
      await expect(page).toHaveURL(`${origin}/ja/guide/index.html`);
      await expect(page.locator("h1")).toHaveText("ガイド");
      await expect(language).toContainText("日本語");
      const theme = page.locator(".mobile-preferences [data-mobile-theme]");
      await theme.scrollIntoViewIfNeeded();
      const before = await page.evaluate(
        () =>
          document.documentElement.getAttribute("data-theme") ??
          (matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light"),
      );
      await theme.click();
      await expect(page.locator("html")).toHaveAttribute(
        "data-theme",
        before === "dark" ? "light" : "dark",
      );
      await page.setViewportSize({ width: 1280, height: 900 });
      await expect(bar).toBeHidden();
      await expect(page.locator(".header-actions .ox-locale-switcher")).toBeVisible();
      await expect(page.locator(".header-actions .theme-toggle")).toBeVisible();
    } finally {
      await fs.rm(root, { recursive: true, force: true });
    }
  });
}
