import { readFile } from "node:fs/promises";
import path from "node:path";
import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";

const TWITTER_FULL_CSS = path.join(
  import.meta.dirname,
  "../../../../crates/ox_content_ssg/src/plugins/social-tweet-full.css",
);

test.describe("Twitter full-card themes", () => {
  test("uses explicit class dark tokens on a light OS", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "light" });
    await renderCard(page, { className: "dark" });

    await expect(cardColors(page)).resolves.toEqual({
      background: "rgb(21, 32, 43)",
      color: "rgb(247, 249, 249)",
    });
  });

  test("keeps data-theme dark tokens", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "light" });
    await renderCard(page, { dataTheme: "dark" });

    await expect(cardColors(page)).resolves.toEqual({
      background: "rgb(21, 32, 43)",
      color: "rgb(247, 249, 249)",
    });
  });

  test("uses OS dark fallback without an explicit light theme", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });
    await renderCard(page);

    await expect(cardColors(page)).resolves.toEqual({
      background: "rgb(21, 32, 43)",
      color: "rgb(247, 249, 249)",
    });
  });

  test("lets explicit light themes override a dark OS", async ({ page }) => {
    await page.emulateMedia({ colorScheme: "dark" });

    for (const attrs of [{ className: "light" }, { dataTheme: "light" }] as const) {
      await renderCard(page, attrs);
      await expect(cardColors(page)).resolves.toEqual({
        background: "rgb(255, 255, 255)",
        color: "rgb(15, 20, 25)",
      });
    }
  });
});

async function renderCard(
  page: Page,
  attrs: { className?: string; dataTheme?: "dark" | "light" } = {},
): Promise<void> {
  const css = await readFile(TWITTER_FULL_CSS, "utf8");
  const htmlClass = attrs.className ? ` class="${attrs.className}"` : "";
  const dataTheme = attrs.dataTheme ? ` data-theme="${attrs.dataTheme}"` : "";
  await page.setContent(
    `<!doctype html>
<html${htmlClass}${dataTheme}>
  <head>
    <meta charset="utf-8">
    <style>${css}</style>
  </head>
  <body>
    <figure class="ox-tweet ox-tweet--fetched ox-tweet--full">
      <p class="ox-tweet__body">Theme probe</p>
    </figure>
  </body>
</html>`,
    { waitUntil: "load" },
  );
}

async function cardColors(page: Page): Promise<{ background: string; color: string }> {
  return page.locator(".ox-tweet--full").evaluate((card) => {
    const style = getComputedStyle(card);
    return {
      background: style.backgroundColor,
      color: style.color,
    };
  });
}
