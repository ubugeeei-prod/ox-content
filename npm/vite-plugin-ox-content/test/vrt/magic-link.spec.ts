import { expect, test } from "@playwright/test";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

test("magic links stay compact inside mobile prose", async ({ page }) => {
  const result = await transformMarkdown(
    "See {link:@ryoppippi} and {link:Oxc}.",
    "docs/vrt-magic-link.md",
    createDocsResolvedOptions({
      magicLinks: {
        enabled: true,
        aliases: {
          Oxc: {
            href: "https://oxc.rs",
            image: "https://github.com/oxc-project.png",
          },
        },
        favicon: false,
        imageOverrides: [],
      },
    }),
  );
  const html = await generateHtmlPage(
    {
      title: "Magic Links",
      content: result.html,
      toc: result.toc,
      frontmatter: {},
      path: "/vrt-magic-link",
      href: "/vrt-magic-link/index.html",
    },
    [],
    "Ox Content",
    "/",
    undefined,
    undefined,
    undefined,
    undefined,
    false,
    { copy: false, externalLinks: true, backToTop: false },
  );

  await page.setViewportSize({ width: 390, height: 844 });
  await page.setContent(html, { waitUntil: "load" });

  const metrics = await page
    .locator(".ox-magic-link")
    .first()
    .evaluate((link) => {
      if (!(link instanceof HTMLElement)) {
        throw new Error("Missing magic link");
      }
      const image = link.querySelector(".ox-magic-link__image");
      const externalIcon = link.querySelector(".ox-external-icon");
      if (!(image instanceof HTMLElement) || !(externalIcon instanceof HTMLElement)) {
        throw new Error("Missing magic-link details");
      }
      const linkStyle = getComputedStyle(link);
      const imageRect = image.getBoundingClientRect();
      const iconRect = externalIcon.getBoundingClientRect();
      const linkRect = link.getBoundingClientRect();
      return {
        borderRadius: Number.parseFloat(linkStyle.borderRadius),
        iconWidth: iconRect.width,
        imageWidth: imageRect.width,
        linkHeight: linkRect.height,
      };
    });

  expect(metrics.linkHeight).toBeLessThanOrEqual(32);
  expect(metrics.borderRadius).toBeLessThan(12);
  expect(metrics.imageWidth).toBeLessThanOrEqual(18);
  expect(metrics.iconWidth).toBeLessThanOrEqual(12);
});
