import { readFile } from "node:fs/promises";
import path from "node:path";
import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import { clearTweetCache } from "../../src/plugins/twitter/fetch";
import { transformFetchedTweets } from "../../src/plugins/twitter/transform";

const SSG_PLUGINS = path.join(import.meta.dirname, "../../../../crates/ox_content_ssg/src/plugins");
const FIXTURE_ID = "1543404742411698176";
const FIXTURE_PERMALINK = `https://x.com/i/web/status/${FIXTURE_ID}`;
const originalFetch = globalThis.fetch;

const STYLE_SOURCES = [
  "social.css",
  "social-tweet-full-isolation.css",
  "social-tweet-full.css",
  "social-tweet-full-media.css",
];

const PROSE_CSS = `
.prose {
  color: #374151;
  max-width: 65ch;
}

.prose a {
  color: #111827;
  padding: 0.75rem 1.25rem;
  font-size: 1.25em;
  font-weight: 500;
  line-height: 2;
  text-decoration: underline;
}
`;

test.afterEach(() => {
  globalThis.fetch = originalFetch;
  clearTweetCache();
});

test.describe("Twitter full-card actions", () => {
  for (const scheme of ["light", "dark"] as const) {
    for (const width of [550, 380]) {
      test(`matches sveltweet action geometry (${scheme}, ${width}px)`, async ({ page }) => {
        await page.setViewportSize({ width: width === 550 ? 680 : width, height: 900 });
        await page.emulateMedia({ colorScheme: scheme });
        await renderFixture(page, { width: width === 550 ? 550 : undefined });

        const geometry = await actionGeometry(page);
        expect(geometry.rowHeight).toBeGreaterThanOrEqual(37);
        expect(geometry.rowHeight).toBeLessThan(45);
        expect(geometry.repliesHeight).toBeGreaterThanOrEqual(32);
        expect(Math.abs(geometry.repliesWidth - geometry.repliesParentWidth)).toBeLessThanOrEqual(
          1,
        );
        expect(geometry.copyHref).toBe(FIXTURE_PERMALINK);
        expect(geometry.copyUrl).toBe(FIXTURE_PERMALINK);

        for (const action of geometry.actions) {
          expect(action.height).toBeGreaterThanOrEqual(32);
          expect(action.width).toBeGreaterThanOrEqual(32);
          expect(action.fontSize).toBe("16px");
          expect(action.lineHeight).toBe("20px");
          expect(action.paddingLeft).toBe("0px");
          expect(action.paddingRight).toBe("0px");
          expect(action.wrapperHeight).toBe(32);
          expect(action.wrapperWidth).toBe(32);
          expect(action.iconHeight).toBeGreaterThanOrEqual(20);
          expect(action.iconWidth).toBeGreaterThanOrEqual(20);
        }
      });
    }
  }
});

async function renderFixture(page: Page, options: { width?: number } = {}): Promise<void> {
  const [css, card] = await Promise.all([concatStyles(), renderFullFixture()]);
  const width = options.width
    ? ` style="width: ${options.width}px; max-width: ${options.width}px"`
    : "";
  await page.setContent(
    `<!doctype html>
<html>
  <head>
    <meta charset="utf-8">
    <style>${css}</style>
    <style>${PROSE_CSS}</style>
    <style>body { margin: 0; padding: 16px; }</style>
  </head>
  <body>
    <article class="prose"${width}>
      ${card}
    </article>
  </body>
</html>`,
    { waitUntil: "load" },
  );
}

async function renderFullFixture(): Promise<string> {
  globalThis.fetch = async (input) => {
    const url = typeof input === "string" ? input : input instanceof URL ? input.href : input.url;
    if (url.startsWith("https://cdn.syndication.twimg.com/")) {
      return {
        ok: true,
        json: async () => ({
          id_str: FIXTURE_ID,
          text: "Thank you JavaScript.",
          created_at: "2022-07-02T23:10:00.000Z",
          favorite_count: 12_345,
          conversation_count: 134,
          retweet_count: 678,
          quote_count: 9,
          user: {
            name: "Action Geometry",
            screen_name: "action_geometry",
            is_blue_verified: true,
          },
        }),
      } as Response;
    }
    return { ok: true, arrayBuffer: async () => new Uint8Array([1, 2, 3]).buffer } as Response;
  };

  return transformFetchedTweets(`<XPost id="${FIXTURE_ID}" />`, {
    fetch: true,
    cache: false,
    appearance: "full",
  });
}

async function concatStyles(): Promise<string> {
  const parts = await Promise.all(
    STYLE_SOURCES.map((file) => readFile(path.join(SSG_PLUGINS, file), "utf8")),
  );
  return parts.join("\n");
}

async function actionGeometry(page: Page): Promise<{
  rowHeight: number;
  repliesHeight: number;
  repliesParentWidth: number;
  repliesWidth: number;
  copyHref: string | null;
  copyUrl: string | null;
  actions: Array<{
    height: number;
    width: number;
    fontSize: string;
    lineHeight: string;
    paddingLeft: string;
    paddingRight: string;
    wrapperHeight: number;
    wrapperWidth: number;
    iconHeight: number;
    iconWidth: number;
  }>;
}> {
  return page.locator(".ox-tweet--full").evaluate((card) => {
    const row = card.querySelector(".ox-tweet__actions") as HTMLElement;
    const replies = card.querySelector(".ox-tweet__replies-link") as HTMLElement;
    const repliesParent = card.querySelector(".ox-tweet__replies") as HTMLElement;
    const copy = card.querySelector("[data-ox-tweet-copy]") as HTMLAnchorElement;
    const rect = (node: Element) => node.getBoundingClientRect();

    return {
      rowHeight: rect(row).height,
      repliesHeight: rect(replies).height,
      repliesParentWidth: rect(repliesParent).width,
      repliesWidth: rect(replies).width,
      copyHref: copy.getAttribute("href"),
      copyUrl: copy.getAttribute("data-ox-tweet-copy-url"),
      actions: Array.from(row.querySelectorAll(".ox-tweet__action")).map((action) => {
        const target = action as HTMLElement;
        const wrapper = target.querySelector(".ox-tweet__action-icon") as HTMLElement;
        const icon = target.querySelector(".ox-tweet__icon") as HTMLElement;
        const style = getComputedStyle(target);
        return {
          height: rect(target).height,
          width: rect(target).width,
          fontSize: style.fontSize,
          lineHeight: style.lineHeight,
          paddingLeft: style.paddingLeft,
          paddingRight: style.paddingRight,
          wrapperHeight: rect(wrapper).height,
          wrapperWidth: rect(wrapper).width,
          iconHeight: rect(icon).height,
          iconWidth: rect(icon).width,
        };
      }),
    };
  });
}
