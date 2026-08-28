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
const SVELTWEET_ACTION_PATHS = {
  like: "M20.884 13.19c-1.351 2.48-4.001 5.12-8.379 7.67l-.503.3-.504-.3c-4.379-2.55-7.029-5.19-8.382-7.67-1.36-2.5-1.41-4.86-.514-6.67.887-1.79 2.647-2.91 4.601-3.01 1.651-.09 3.368.56 4.798 2.01 1.429-1.45 3.146-2.1 4.796-2.01 1.954.1 3.714 1.22 4.601 3.01.896 1.81.846 4.17-.514 6.67z",
  reply:
    "M1.751 10c0-4.42 3.584-8 8.005-8h4.366c4.49 0 8.129 3.64 8.129 8.13 0 2.96-1.607 5.68-4.196 7.11l-8.054 4.46v-3.69h-.067c-4.49.1-8.183-3.51-8.183-8.01z",
  copy: "M18.36 5.64c-1.95-1.96-5.11-1.96-7.07 0L9.88 7.05 8.46 5.64l1.42-1.42c2.73-2.73 7.16-2.73 9.9 0 2.73 2.74 2.73 7.17 0 9.9l-1.42 1.42-1.41-1.42 1.41-1.41c1.96-1.96 1.96-5.12 0-7.07zm-2.12 3.53l-7.07 7.07-1.41-1.41 7.07-7.07 1.41 1.41zm-12.02.71l1.42-1.42 1.41 1.42-1.41 1.41c-1.96 1.96-1.96 5.12 0 7.07 1.95 1.96 5.11 1.96 7.07 0l1.41-1.41 1.42 1.41-1.42 1.42c-2.73 2.73-7.16 2.73-9.9 0-2.73-2.74-2.73-7.17 0-9.9z",
  check: "M9.64 18.952l-5.55-4.861 1.317-1.504 3.951 3.459 8.459-10.948L19.4 6.32 9.64 18.952z",
} as const;

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
        expect(
          Math.abs(geometry.repliesOffsetTop - geometry.actionsOffsetBottom),
        ).toBeLessThanOrEqual(1);
        expect(geometry.repliesHeight).toBeGreaterThanOrEqual(32);
        expect(Math.abs(geometry.repliesWidth - geometry.repliesParentWidth)).toBeLessThanOrEqual(
          1,
        );
        const bottomInset = width === 550 ? 13 : 11;
        expect(
          Math.abs(geometry.cardHeight - geometry.repliesOffsetBottom - bottomInset),
        ).toBeLessThanOrEqual(1);
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

      test(`matches sveltweet action icons and labels (${scheme}, ${width}px)`, async ({
        page,
      }) => {
        await page.setViewportSize({ width: width === 550 ? 680 : width, height: 900 });
        await page.emulateMedia({ colorScheme: scheme });
        await renderFixture(page, { width: width === 550 ? 550 : undefined });

        const defaults = await actionVisualContract(page);
        expect(defaults).toMatchObject({
          likeLabel: "Like. This Tweet has 12.3K likes",
          replyLabel: "Reply to this Tweet on Twitter",
          copyLabel: "Copy link",
          likePath: SVELTWEET_ACTION_PATHS.like,
          replyPath: SVELTWEET_ACTION_PATHS.reply,
          copyPath: SVELTWEET_ACTION_PATHS.copy,
        });

        await page
          .locator("[data-ox-tweet-copy]")
          .evaluate((copy) => copy.setAttribute("data-ox-tweet-copied", ""));

        const copied = await actionVisualContract(page);
        expect(copied.copyPath).toBe(SVELTWEET_ACTION_PATHS.check);
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
  actionsOffsetBottom: number;
  cardHeight: number;
  rowHeight: number;
  repliesHeight: number;
  repliesOffsetBottom: number;
  repliesOffsetTop: number;
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
    const cardRect = rect(card);
    const rowRect = rect(row);
    const repliesParentRect = rect(repliesParent);

    return {
      actionsOffsetBottom: rowRect.bottom - cardRect.top,
      cardHeight: cardRect.height,
      rowHeight: rowRect.height,
      repliesHeight: rect(replies).height,
      repliesOffsetBottom: repliesParentRect.bottom - cardRect.top,
      repliesOffsetTop: repliesParentRect.top - cardRect.top,
      repliesParentWidth: repliesParentRect.width,
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

async function actionVisualContract(page: Page): Promise<{
  copyLabel: string | null;
  copyPath: string | null;
  likeLabel: string | null;
  likePath: string | null;
  replyLabel: string | null;
  replyPath: string | null;
}> {
  return page.locator(".ox-tweet--full").evaluate((card) => {
    const iconPath = (icon: HTMLElement): string | null => {
      const style = getComputedStyle(icon) as CSSStyleDeclaration & { webkitMaskImage?: string };
      const image = style.webkitMaskImage || style.maskImage;
      const url = image.match(/^url\(["']?(.*?)["']?\)$/)?.[1];
      if (!url?.startsWith("data:image/svg+xml,")) return null;
      const svg = decodeURIComponent(url.slice("data:image/svg+xml,".length));
      return (
        new DOMParser()
          .parseFromString(svg, "image/svg+xml")
          .querySelector("path")
          ?.getAttribute("d") ?? null
      );
    };
    const like = card.querySelector(".ox-tweet__action--like") as HTMLAnchorElement;
    const reply = card.querySelector(".ox-tweet__action--reply") as HTMLAnchorElement;
    const copy = card.querySelector("[data-ox-tweet-copy]") as HTMLAnchorElement;
    const likeIcon = card.querySelector(".ox-tweet__icon--like") as HTMLElement;
    const replyIcon = card.querySelector(".ox-tweet__icon--reply") as HTMLElement;
    const copyIcon = card.querySelector(".ox-tweet__icon--copy") as HTMLElement;

    return {
      copyLabel: copy.getAttribute("aria-label"),
      copyPath: iconPath(copyIcon),
      likeLabel: like.getAttribute("aria-label"),
      likePath: iconPath(likeIcon),
      replyLabel: reply.getAttribute("aria-label"),
      replyPath: iconPath(replyIcon),
    };
  });
}
