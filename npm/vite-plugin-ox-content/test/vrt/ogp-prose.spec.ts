import { readFile } from "node:fs/promises";
import path from "node:path";
import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";

const ogpCssPath = path.join(
  import.meta.dirname,
  "../../../../crates/ox_content_ssg/src/plugins/ogp.css",
);

const PROSE_CSS = `
.prose {
  box-sizing: border-box;
  width: min(832px, calc(100vw - 40px));
  margin: 24px auto;
  color: #374151;
  font-family: Arial, sans-serif;
  line-height: 1.65;
}
.prose a {
  color: #111827;
  font-weight: 600;
  text-decoration: underline;
}
.prose img {
  display: block;
  margin-top: 2em;
  margin-bottom: 2em;
  max-width: 100%;
}
`;

const favicon =
  "data:image/svg+xml," +
  encodeURIComponent(
    '<svg xmlns="http://www.w3.org/2000/svg" width="32" height="32"><rect width="32" height="32" fill="#4d699b"/></svg>',
  );

test.describe("Open Graph cards inside prose hosts", () => {
  test("keeps previews visible and margins isolated on desktop rows", async ({ page }) => {
    await page.setViewportSize({ width: 1024, height: 900 });
    await renderInProse(page);
    await waitForPreviewImages(page);

    const metrics = await page.evaluate(measureCards);

    expect(metrics.documentOverflow).toBeLessThanOrEqual(0);
    expect(metrics.cards).toHaveLength(2);
    for (const card of metrics.cards) {
      expect(card.flexDirection).toBe("row");
      expect(card.cardOverflow).toBeLessThanOrEqual(0);
      expect(card.cardWidth).toBeCloseTo(832, 0);
      expect(card.image.marginTop).toBe("0px");
      expect(card.image.marginBottom).toBe("0px");
      expect(card.image.objectFit).toBe("contain");
      expect(card.image.backgroundColor).toBe("rgb(245, 247, 250)");
      expect(card.image.width).toBeCloseTo(200, 0);
      expect(card.favicon.marginTop).toBe("0px");
      expect(card.favicon.marginBottom).toBe("0px");
      expect(card.textDecorationLine).toBe("none");
      expect(card.fontWeight).toBe("400");
    }

    expect(metrics.cards[0]?.image.naturalWidth).toBe(1200);
    expect(metrics.cards[0]?.image.naturalHeight).toBe(600);
    expect(metrics.cards[1]?.image.naturalWidth).toBe(1200);
    expect(metrics.cards[1]?.image.naturalHeight).toBe(630);
  });

  test("keeps the published card layout contained on narrow prose columns", async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 900 });
    await renderInProse(page);
    await waitForPreviewImages(page);

    const metrics = await page.evaluate(measureCards);

    expect(metrics.documentOverflow).toBeLessThanOrEqual(0);
    for (const card of metrics.cards) {
      expect(card.flexDirection).toBe("column");
      expect(card.cardOverflow).toBeLessThanOrEqual(0);
      expect(card.cardWidth).toBeCloseTo(350, 0);
      expect(card.image.width).toBeCloseTo(card.cardWidth, 0);
      expect(card.image.height).toBeCloseTo(160, 0);
      expect(card.image.marginTop).toBe("0px");
      expect(card.image.marginBottom).toBe("0px");
      expect(card.image.objectFit).toBe("contain");
      expect(card.favicon.marginTop).toBe("0px");
      expect(card.favicon.marginBottom).toBe("0px");
    }
  });
});

async function renderInProse(page: Page): Promise<void> {
  const componentCss = await readFile(ogpCssPath, "utf8");
  await page.setContent(
    `<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <style>
      :root {
        --octc-color-bg-alt: rgb(245, 247, 250);
        --octc-color-border: rgb(203, 213, 225);
        --octc-color-primary: rgb(77, 105, 155);
        --octc-color-text: rgb(17, 24, 39);
        --octc-color-text-muted: rgb(75, 85, 99);
      }
      body {
        margin: 0;
      }
      ${componentCss}
    </style>
    <style>${PROSE_CSS}</style>
  </head>
  <body>
    <article class="prose">
      ${cardHtml({
        title:
          "ryoppippi/nix-secure-enclave-key keeps a deliberately long Open Graph title so the text column makes the flex row taller",
        description:
          "A long description used to prove that card copy can no longer decide which part of the 1200 by 600 preview remains visible.",
        image: previewImage(1200, 600, "1200x600"),
      })}
      ${cardHtml({
        title: "A compact 1200 by 630 article preview",
        description: "Short metadata still uses the same containment rule.",
        image: previewImage(1200, 630, "1200x630"),
      })}
    </article>
  </body>
</html>`,
    { waitUntil: "load" },
  );
}

async function waitForPreviewImages(page: Page): Promise<void> {
  await page.waitForFunction(() =>
    Array.from(document.querySelectorAll(".ox-ogp-image")).every(
      (node) =>
        node instanceof HTMLImageElement &&
        node.complete &&
        node.naturalWidth > 0 &&
        node.naturalHeight > 0,
    ),
  );
}

function cardHtml(input: { title: string; description: string; image: string }): string {
  return `<a class="ox-ogp-card" href="https://example.com/post" target="_blank" rel="noopener noreferrer">
  <div class="ox-ogp-content">
    <div class="ox-ogp-title">${input.title}</div>
    <div class="ox-ogp-description">${input.description}</div>
    <div class="ox-ogp-meta">
      <img class="ox-ogp-favicon" src="${favicon}" alt="" loading="lazy">
      <span class="ox-ogp-domain">example.com</span>
    </div>
  </div>
  <img class="ox-ogp-image" src="${input.image}" alt="" loading="lazy">
</a>`;
}

function previewImage(width: number, height: number, label: string): string {
  return (
    "data:image/svg+xml," +
    encodeURIComponent(`<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">
  <rect width="${width}" height="${height}" fill="#fef3c7"/>
  <rect x="8" y="8" width="${width - 16}" height="${height - 16}" fill="none" stroke="#b91c1c" stroke-width="16"/>
  <text x="50%" y="50%" dominant-baseline="middle" text-anchor="middle" font-family="Arial" font-size="80" fill="#111827">${label}</text>
</svg>`)
  );
}

function measureCards() {
  return {
    documentOverflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
    cards: Array.from(document.querySelectorAll(".ox-ogp-card")).map((node) => {
      const card = node as HTMLElement;
      const image = card.querySelector(".ox-ogp-image") as HTMLImageElement | null;
      const favicon = card.querySelector(".ox-ogp-favicon") as HTMLImageElement | null;
      if (!image || !favicon) {
        throw new Error("Missing OGP image targets");
      }
      const cardRect = card.getBoundingClientRect();
      const imageRect = image.getBoundingClientRect();
      const cardStyle = getComputedStyle(card);
      const imageStyle = getComputedStyle(image);
      const faviconStyle = getComputedStyle(favicon);
      return {
        cardOverflow: card.scrollWidth - card.clientWidth,
        cardWidth: cardRect.width,
        flexDirection: cardStyle.flexDirection,
        fontWeight: cardStyle.fontWeight,
        textDecorationLine: cardStyle.textDecorationLine,
        image: {
          backgroundColor: imageStyle.backgroundColor,
          height: imageRect.height,
          marginBottom: imageStyle.marginBottom,
          marginTop: imageStyle.marginTop,
          naturalHeight: image.naturalHeight,
          naturalWidth: image.naturalWidth,
          objectFit: imageStyle.objectFit,
          width: imageRect.width,
        },
        favicon: {
          marginBottom: faviconStyle.marginBottom,
          marginTop: faviconStyle.marginTop,
        },
      };
    }),
  };
}
