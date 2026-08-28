import { readFile } from "node:fs/promises";
import path from "node:path";
import { expect, test } from "@playwright/test";

const runtimePath = path.join(
  import.meta.dirname,
  "../../../../crates/ox_content_ssg/src/html/plugins/twitter.js",
);
const permalink = "https://x.com/i/web/status/1543404742411698176";

test("tweet client copies links and resets accessible state", async ({ page }) => {
  const runtime = await readFile(runtimePath, "utf8");
  await page.setContent(pageHtml(), { waitUntil: "load" });
  await page.addScriptTag({ content: `${runtime}\ninitTweetCards(document, { copiedMs: 50 });` });
  await page.evaluate(() => {
    const writes: string[] = [];
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText(value: string) {
          writes.push(value);
          return Promise.resolve();
        },
      },
    });
    (window as typeof window & { __tweetWrites?: string[] }).__tweetWrites = writes;
  });

  await page.locator("[data-ox-tweet-copy]").focus();
  await page.keyboard.press("Enter");

  await expect
    .poll(() =>
      page.evaluate(() => (window as typeof window & { __tweetWrites?: string[] }).__tweetWrites),
    )
    .toEqual([permalink]);
  await expect(page.locator("[data-ox-tweet-copy]")).toHaveAttribute("data-ox-tweet-copied", "");
  await expect(page.locator("[data-ox-tweet-copy]")).toHaveAttribute("aria-label", "Copied!");
  await expect(page.locator("[data-ox-tweet-copy-status]")).toHaveText("Copied!");
  await expect(page.locator("[data-ox-tweet-copy]")).not.toHaveAttribute(
    "data-ox-tweet-copied",
    "",
  );
  await expect(page.locator("[data-ox-tweet-copy-status]")).toHaveText("");
});

test("tweet client preserves fallback anchors without Clipboard API", async ({ page }) => {
  const runtime = await readFile(runtimePath, "utf8");
  await page.setContent(pageHtml(), { waitUntil: "load" });
  await page.evaluate(() => {
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: undefined,
    });
  });
  await page.addScriptTag({ content: `${runtime}\ninitTweetCards(document);` });
  const prevented = await page.locator("[data-ox-tweet-copy]").evaluate((anchor) => {
    let defaultPrevented: boolean | undefined;
    document.addEventListener("click", (event) => {
      defaultPrevented = event.defaultPrevented;
    });
    anchor.dispatchEvent(new MouseEvent("click", { bubbles: true, cancelable: true }));
    return defaultPrevented ?? false;
  });

  expect(prevented).toBe(false);
  await expect(page.locator("[data-ox-tweet-copy]")).toHaveAttribute("href", permalink);
});

test("tweet client delegates to inserted cards and avoids duplicate writes", async ({ page }) => {
  const runtime = await readFile(runtimePath, "utf8");
  await page.setContent('<main id="root"></main>', { waitUntil: "load" });
  await page.addScriptTag({ content: runtime });
  await page.evaluate((cardHtml) => {
    const writes: string[] = [];
    Object.defineProperty(navigator, "clipboard", {
      configurable: true,
      value: {
        writeText(value: string) {
          writes.push(value);
          return Promise.resolve();
        },
      },
    });
    const root = document.querySelector("#root");
    const win = window as typeof window & {
      __tweetWrites?: string[];
      initTweetCards?: (root?: Document | Element | null, options?: { copiedMs?: number }) => void;
    };
    win.initTweetCards?.(document, { copiedMs: 5 });
    win.initTweetCards?.(root, { copiedMs: 5 });
    root?.insertAdjacentHTML("beforeend", cardHtml);
    win.__tweetWrites = writes;
  }, tweetCardHtml());

  await page.locator("[data-ox-tweet-copy]").click();

  await expect
    .poll(() =>
      page.evaluate(() => (window as typeof window & { __tweetWrites?: string[] }).__tweetWrites),
    )
    .toEqual([permalink]);
});

function pageHtml() {
  return `<!doctype html>
<html>
  <body>
    <main id="root">${tweetCardHtml()}</main>
    <script>window.__tweetCardHtml = ${JSON.stringify(tweetCardHtml())};</script>
  </body>
</html>`;
}

function tweetCardHtml() {
  return `<a class="ox-tweet__action ox-tweet__action--copy" href="${permalink}" target="_blank" rel="noopener noreferrer" data-ox-tweet-copy data-ox-tweet-copy-url="${permalink}" aria-label="Copy link to post"><span class="ox-tweet__action-text ox-tweet__copy-text">Copy link</span><span class="ox-tweet__action-text ox-tweet__copied-text">Copied!</span><span class="ox-tweet__copy-status" data-ox-tweet-copy-status role="status" aria-live="polite"></span></a>`;
}
