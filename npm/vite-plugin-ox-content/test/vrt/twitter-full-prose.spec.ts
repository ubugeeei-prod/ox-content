import { readFile } from "node:fs/promises";
import path from "node:path";
import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";

const SSG_PLUGINS = path.join(import.meta.dirname, "../../../../crates/ox_content_ssg/src/plugins");

/** The published `twitter-full.css`, in the order the package concatenates it. */
const TWITTER_FULL_SOURCES = [
  "social-tweet-full-isolation.css",
  "social-tweet-full.css",
  "social-tweet-full-media.css",
];

/** The published `social.css` entry, trimmed to what a full card needs. */
const SOCIAL_SOURCES = ["social.css"];

/**
 * The `@tailwindcss/typography` rules a full card lands in.
 *
 * Copied from the plugin's own output rather than paraphrased, so the test
 * fails if the isolation stops covering what a real `.prose` article does.
 */
const PROSE_CSS = `
.prose { color: #374151; max-width: 65ch; }
.prose a {
  color: #111827;
  padding: 0.75rem 1.25rem;
  font-size: 1.25em;
  font-weight: 500;
  line-height: 2;
  text-decoration: underline;
}
.prose strong { font-weight: 600; }
.prose ol, .prose ul { margin-top: 1.25em; margin-bottom: 1.25em; padding-inline-start: 1.625em; }
.prose ol { list-style-type: decimal; }
.prose ul { list-style-type: disc; }
.prose img { margin-top: 2em; margin-bottom: 2em; }
.prose video { margin-top: 2em; margin-bottom: 2em; }
.prose figure { margin-top: 2em; margin-bottom: 2em; }
.prose figure > * { margin-top: 0; margin-bottom: 0; }
.prose p { margin-top: 1.25em; margin-bottom: 1.25em; }
.prose code { color: #111827; font-weight: 600; font-size: 0.875em; }
.prose code::before { content: "\`"; }
.prose code::after { content: "\`"; }
.prose blockquote {
  font-weight: 500;
  font-style: italic;
  color: #111827;
  border-inline-start-width: 0.25rem;
  border-inline-start-color: #e5e7eb;
  quotes: "\\201C""\\201D""\\2018""\\2019";
  margin-top: 1.6em;
  margin-bottom: 1.6em;
  padding-inline-start: 1em;
}
.prose blockquote p:first-of-type::before { content: open-quote; }
.prose blockquote p:last-of-type::after { content: close-quote; }
`;

const CARD_HTML = `
<figure class="ox-tweet ox-tweet--fetched ox-tweet--full">
  <div class="ox-tweet__header">
    <a class="ox-tweet__avatar-link" href="https://x.com/probe">
      <img class="ox-tweet__avatar" src="data:image/gif;base64,R0lGODlhAQABAAAAACw=" alt="" width="48" height="48">
    </a>
    <div class="ox-tweet__author">
      <a class="ox-tweet__author-name" href="https://x.com/probe">Probe</a>
      <div class="ox-tweet__author-meta"><span class="ox-tweet__author-handle">@probe</span></div>
    </div>
  </div>
  <p class="ox-tweet__body">Body text with <code>code</code> in it.</p>
  <div class="ox-tweet__media" data-count="1">
    <img class="ox-tweet__media-item" src="data:image/gif;base64,R0lGODlhAQABAAAAACw=" alt="">
  </div>
  <blockquote class="ox-tweet__quote">
    <div class="ox-tweet__quote-header">
      <img class="ox-tweet__avatar" src="data:image/gif;base64,R0lGODlhAQABAAAAACw=" alt="" width="20" height="20">
      <span class="ox-tweet__author-handle">@quoted</span>
    </div>
    <p class="ox-tweet__quote-body">Quoted post text.</p>
  </blockquote>
</figure>`;

test.describe("Twitter full cards inside prose", () => {
  for (const scheme of ["light", "dark"] as const) {
    for (const width of [1280, 380]) {
      test(`stays isolated from article prose rules (${scheme}, ${width}px)`, async ({ page }) => {
        await page.setViewportSize({ width, height: 900 });
        await page.emulateMedia({ colorScheme: scheme });
        await renderInProse(page);

        // Avatars keep the 48px header geometry rather than growing by the
        // article's image margins.
        await expect(margins(page, ".ox-tweet__header .ox-tweet__avatar")).resolves.toEqual({
          top: "0px",
          bottom: "0px",
        });
        await expect(margins(page, ".ox-tweet__quote .ox-tweet__avatar")).resolves.toEqual({
          top: "0px",
          bottom: "0px",
        });
        await expect(margins(page, ".ox-tweet__media-item")).resolves.toEqual({
          top: "0px",
          bottom: "0px",
        });

        // The quoted post keeps full-card typography, not article quotation
        // styling, and gets no generated quotation marks.
        const quote = await page.locator(".ox-tweet__quote").evaluate((node) => {
          const style = getComputedStyle(node);
          return { fontStyle: style.fontStyle, fontWeight: style.fontWeight, quotes: style.quotes };
        });
        expect(quote.fontStyle).toBe("normal");
        expect(quote.fontWeight).toBe("400");
        expect(quote.quotes).toBe("none");

        for (const pseudo of ["::before", "::after"]) {
          const content = await page
            .locator(".ox-tweet__quote-body")
            .evaluate((node, which) => getComputedStyle(node, which).content, pseudo);
          expect(content).toBe("none");
        }

        // The card keeps its own outer margin rather than the article's
        // figure spacing.
        await expect(margins(page, ".ox-tweet--full")).resolves.toEqual({
          top: "24px",
          bottom: "24px",
        });

        // Body paragraphs and inline code do not pick up article spacing.
        await expect(margins(page, ".ox-tweet__body")).resolves.toEqual({
          top: "0px",
          bottom: "0px",
        });
        const codeContent = await page
          .locator(".ox-tweet__body code")
          .evaluate((node) => getComputedStyle(node, "::before").content);
        expect(codeContent).toBe("none");
      });
    }
  }

  test("keeps the 48px header geometry a prose article would inflate", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await renderInProse(page);

    const header = await page.locator(".ox-tweet__header").evaluate((node) => {
      const avatar = node.querySelector(".ox-tweet__avatar") as HTMLElement;
      return {
        header: node.getBoundingClientRect().height,
        avatar: avatar.getBoundingClientRect().height,
      };
    });

    expect(header.avatar).toBe(48);
    // 48px avatar plus the card's own line-height, nowhere near the 108px a
    // 30px-per-side prose image margin produced.
    expect(header.header).toBeLessThanOrEqual(60);
  });
});

async function renderInProse(page: Page): Promise<void> {
  const componentCss = await concat([...SOCIAL_SOURCES, ...TWITTER_FULL_SOURCES]);
  await page.setContent(
    `<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <style>${componentCss}</style>
    <style>${PROSE_CSS}</style>
  </head>
  <body>
    <article class="prose">
      <p>Article prose before the card.</p>
      ${CARD_HTML}
      <p>Article prose after the card.</p>
    </article>
  </body>
</html>`,
    { waitUntil: "load" },
  );
}

/** Host stylesheets last, which is the order that used to lose. */
async function concat(files: string[]): Promise<string> {
  const parts = await Promise.all(
    files.map((file) => readFile(path.join(SSG_PLUGINS, file), "utf8")),
  );
  return parts.join("\n");
}

async function margins(page: Page, selector: string): Promise<{ top: string; bottom: string }> {
  return page
    .locator(selector)
    .first()
    .evaluate((node) => {
      const style = getComputedStyle(node);
      return { top: style.marginTop, bottom: style.marginBottom };
    });
}
