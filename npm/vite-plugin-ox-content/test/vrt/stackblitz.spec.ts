import { expect, test } from "@playwright/test";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

test("stackblitz markdown renders as an iframe embed", async ({ page }) => {
  const result = await transformMarkdown(
    '<StackBlitz url="https://stackblitz.com/edit/vitejs-vite" />',
    "docs/vrt-stackblitz.md",
    createDocsResolvedOptions({
      embeds: {
        github: false,
        openGraph: false,
        pm: false,
        spotify: false,
        appleMusic: false,
        speakerDeck: false,
        stackBlitz: true,
        twitter: false,
        bluesky: false,
        webContainer: false,
      },
    }),
  );
  const html = await generateHtmlPage(
    {
      title: "StackBlitz",
      content: result.html,
      toc: result.toc,
      frontmatter: {},
      path: "/vrt-stackblitz",
      href: "/vrt-stackblitz/index.html",
    },
    [],
    "Ox Content",
    "/",
  );

  await page.setViewportSize({ width: 390, height: 844 });
  await page.setContent(html, { waitUntil: "domcontentloaded" });

  const iframe = page.locator("iframe.ox-stackblitz");
  await expect(iframe).toHaveAttribute("src", "https://stackblitz.com/edit/vitejs-vite?embed=1");
  await expect(iframe).toHaveAttribute("sandbox", /allow-scripts/);
  await expect(iframe).toHaveAttribute("loading", "lazy");
  await expect(page.locator("body")).not.toContainText("</StackBlitz>");
});
