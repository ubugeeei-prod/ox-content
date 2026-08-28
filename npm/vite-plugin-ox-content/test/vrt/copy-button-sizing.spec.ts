import { expect, test, type Page } from "@playwright/test";
import {
  applyReaderChromeHtml,
  renderReaderChromeAttributes,
  renderReaderChromeStyleTag,
} from "../../src/reader-chrome";
import { generateHtmlPage } from "../../src/ssg";
import { transformMarkdown } from "../../src/transform";
import { createDocsResolvedOptions } from "../fixtures/docs-fixture";

const COPY_SIZE_MARKDOWN = [
  "# Copy Sizing",
  "",
  "```ts:line-numbers=117 :wrap [src/components/really-long-reader-chrome-copy-button-layout-example.ts]",
  "export async function loadUserSession(request: Request, cache: Map<string, Promise<UserSession>>) {",
  '  const authorization = request.headers.get("authorization") ?? request.headers.get("x-legacy-auth-token");',
  '  const key = authorization?.replace(/^Bearer\\s+/i, "") ?? "anonymous-session-without-token";',
  "  return cache.get(key) ?? fetchAndStoreUserSession(cache, key, authorization);",
  "}",
  "```",
].join("\n");

const COPY_SIZE_EXPECTATIONS = {
  default: { control: 28, icon: 13, inset: 8 },
  custom: { control: 40, icon: 16, inset: 12 },
} as const;

type CopySizeMode = keyof typeof COPY_SIZE_EXPECTATIONS;
type CopyColorScheme = "light" | "dark";
type CopyDirection = "ltr" | "rtl";

test.describe("reader chrome copy sizing tokens", () => {
  for (const scheme of ["light", "dark"] as const) {
    for (const mode of ["default", "custom"] as const) {
      test(`keep code controls clear (${scheme}, desktop, ${mode})`, async ({ page }) => {
        await page.setViewportSize({ width: 760, height: 720 });
        await assertCopySizing(page, { scheme, mode, direction: "ltr" });
      });
    }
  }
});

test.describe("reader chrome copy sizing tokens on touch layouts", () => {
  test.use({ hasTouch: true, isMobile: true, viewport: { width: 390, height: 844 } });

  for (const scheme of ["light", "dark"] as const) {
    for (const mode of ["default", "custom"] as const) {
      test(`keep code controls clear (${scheme}, touch, ${mode})`, async ({ page }) => {
        await assertCopySizing(page, {
          scheme,
          mode,
          direction: scheme === "dark" && mode === "custom" ? "rtl" : "ltr",
        });
      });
    }
  }
});

test("custom host can resize copy controls with public tokens only", async ({ page }) => {
  const markdown = ["# Copy", "", "```ts", "const value = 1;", "```"].join("\n");
  const result = await transformMarkdown(
    markdown,
    "docs/vrt-custom-copy-size.md",
    createDocsResolvedOptions({ highlight: true }),
  );
  const article = applyReaderChromeHtml(result.html, {
    copy: true,
    externalLinks: false,
    backToTop: false,
  });

  await page.setContent(
    `<!DOCTYPE html>
<html>
<head>
  <style>
    :root {
      --octc-color-primary: #4f6fae;
      --octc-color-text: #131a30;
      --octc-color-bg: #ffffff;
      --octc-color-bg-alt: #f5f7fb;
      --octc-color-code-text: #e5e9f0;
      --octc-color-code-bg: #121a2f;
      --octc-color-code-frame-border: #44506a;
    }
    .content {
      --ox-copy-control-size: 2.25rem;
      --ox-copy-icon-size: 1rem;
      --ox-copy-inset: 0.625rem;
      max-width: 680px;
      margin: 0 auto;
    }
    .content pre { margin: 2rem 0; padding: 3rem; background: var(--octc-color-code-bg); color: var(--octc-color-code-text); }
    .content button { width: 14rem; min-height: 5rem; padding: 2rem; font-size: 3rem; }
  </style>
  ${renderReaderChromeStyleTag({ copy: true, externalLinks: false, backToTop: false })}
</head>
<body>
  <article class="content"${renderReaderChromeAttributes({ copy: true, externalLinks: false, backToTop: false })}>${article}</article>
</body>
</html>`,
    { waitUntil: "load" },
  );

  const metrics = await readCopyMetrics(page);
  expectClose(metrics.controlWidth, 36);
  expectClose(metrics.controlHeight, 36);
  expectClose(metrics.iconWidth, 16);
  expectClose(metrics.iconHeight, 16);
  expectClose(metrics.insetBlockStart, 10);
  expectClose(metrics.insetInlineEnd, 10);
});

async function assertCopySizing(
  page: Page,
  options: { direction: CopyDirection; mode: CopySizeMode; scheme: CopyColorScheme },
) {
  await page.emulateMedia({ colorScheme: options.scheme });
  await renderCopySizingFixture(page, options);

  const button = page.locator(".ox-code").first().locator(".ox-copy");
  const expected = COPY_SIZE_EXPECTATIONS[options.mode];
  const rest = await readCopyMetrics(page);
  expectCopyMetrics(rest, expected);
  expect(rest.lineNumberWidth).toBeGreaterThan(0);
  expect(rest.preScrollWidth).toBeLessThanOrEqual(rest.preClientWidth + 1);
  expect(rest.prePaddingInlineEnd).toBeGreaterThanOrEqual(expected.control + expected.inset - 1);
  expect(rest.titlePaddingInlineEnd).toBeGreaterThanOrEqual(expected.control + expected.inset - 1);
  expect(Math.abs(rest.titleMarginInlineEnd)).toBeGreaterThanOrEqual(
    rest.titlePaddingInlineEnd - 1,
  );

  await page.locator(".ox-code").first().hover();
  await button.hover({ force: true });
  expectSameCopyLayout(rest, await readCopyMetrics(page));

  await button.focus();
  expectSameCopyLayout(rest, await readCopyMetrics(page));

  await button.evaluate((node) => node.setAttribute("data-copy-state", "copied"));
  expectSameCopyLayout(rest, await readCopyMetrics(page));

  await button.evaluate((node) => node.setAttribute("data-copy-state", "failed"));
  expectSameCopyLayout(rest, await readCopyMetrics(page));
}

async function renderCopySizingFixture(
  page: Page,
  options: { direction: CopyDirection; mode: CopySizeMode; scheme: CopyColorScheme },
) {
  const result = await transformMarkdown(
    COPY_SIZE_MARKDOWN,
    "docs/vrt-copy-sizing.md",
    createDocsResolvedOptions({
      highlight: true,
      codeAnnotations: {
        enabled: true,
        notation: "vitepress",
        metaKey: "annotate",
        defaultLineNumbers: false,
      },
    }),
  );
  const html = await generateHtmlPage(
    {
      title: "Copy Sizing",
      content: result.html,
      toc: result.toc,
      frontmatter: {},
      path: "/vrt-copy-sizing",
      href: "/vrt-copy-sizing/index.html",
    },
    [],
    "Ox Content",
    "/",
    undefined,
    undefined,
    undefined,
    undefined,
    false,
    { copy: true, externalLinks: false, backToTop: false },
  );

  await page.setContent(html, { waitUntil: "load" });
  await page.evaluate(({ direction, scheme }) => {
    document.documentElement.setAttribute("dir", direction);
    document.documentElement.setAttribute("data-theme", scheme);
    document.querySelector(".content")?.classList.add("copy-sizing-fixture");
  }, options);
  await page.addStyleTag({
    content: `
      .copy-sizing-fixture {
        max-width: 420px;
        margin: 0 auto;
        font-size: 16px;
      }
      .copy-sizing-fixture pre {
        font-size: 16px;
      }
      .copy-sizing-fixture.copy-sizing-fixture--custom {
        --ox-copy-control-size: 2.5rem;
        --ox-copy-icon-size: 1rem;
        --ox-copy-inset: 0.75rem;
      }
    `,
  });
  if (options.mode === "custom") {
    await page.locator(".content").evaluate((content) => {
      content.classList.add("copy-sizing-fixture--custom");
    });
  }
}

async function readCopyMetrics(page: Page) {
  return page
    .locator(".ox-code")
    .first()
    .evaluate((code) => {
      if (!(code instanceof HTMLElement)) throw new Error("Missing code wrapper");
      const button = code.querySelector(".ox-copy");
      const pre = code.querySelector("pre");
      const firstLine = code.querySelector(".line");
      if (!(button instanceof HTMLElement)) throw new Error("Missing copy button");
      if (!(pre instanceof HTMLElement)) throw new Error("Missing code block");
      if (!(firstLine instanceof HTMLElement)) throw new Error("Missing code line");

      const px = (style: CSSStyleDeclaration, property: string) =>
        Number.parseFloat(style.getPropertyValue(property)) || 0;
      const wrapperRect = code.getBoundingClientRect();
      const buttonRect = button.getBoundingClientRect();
      const preStyle = getComputedStyle(pre);
      const titleStyle = getComputedStyle(pre, "::before");
      const iconStyle = getComputedStyle(button, "::before");
      const lineNumberStyle = getComputedStyle(firstLine, "::before");
      const isRtl = getComputedStyle(code).direction === "rtl";

      return {
        controlHeight: buttonRect.height,
        controlWidth: buttonRect.width,
        iconHeight: px(iconStyle, "height"),
        iconWidth: px(iconStyle, "width"),
        insetBlockStart: buttonRect.top - wrapperRect.top,
        insetInlineEnd: isRtl
          ? buttonRect.left - wrapperRect.left
          : wrapperRect.right - buttonRect.right,
        lineNumberWidth: px(lineNumberStyle, "width"),
        preClientWidth: pre.clientWidth,
        prePaddingInlineEnd: px(preStyle, "padding-inline-end"),
        preScrollWidth: pre.scrollWidth,
        titleMarginInlineEnd: px(titleStyle, "margin-inline-end"),
        titlePaddingInlineEnd: px(titleStyle, "padding-inline-end"),
      };
    });
}

function expectCopyMetrics(
  metrics: Awaited<ReturnType<typeof readCopyMetrics>>,
  expected: (typeof COPY_SIZE_EXPECTATIONS)[CopySizeMode],
) {
  expectClose(metrics.controlWidth, expected.control);
  expectClose(metrics.controlHeight, expected.control);
  expectClose(metrics.iconWidth, expected.icon);
  expectClose(metrics.iconHeight, expected.icon);
  expectClose(metrics.insetBlockStart, expected.inset);
  expectClose(metrics.insetInlineEnd, expected.inset);
}

function expectSameCopyLayout(
  before: Awaited<ReturnType<typeof readCopyMetrics>>,
  after: Awaited<ReturnType<typeof readCopyMetrics>>,
) {
  expectClose(after.controlWidth, before.controlWidth);
  expectClose(after.controlHeight, before.controlHeight);
  expectClose(after.iconWidth, before.iconWidth);
  expectClose(after.iconHeight, before.iconHeight);
  expectClose(after.insetBlockStart, before.insetBlockStart);
  expectClose(after.insetInlineEnd, before.insetInlineEnd);
  expectClose(after.prePaddingInlineEnd, before.prePaddingInlineEnd);
  expectClose(after.titlePaddingInlineEnd, before.titlePaddingInlineEnd);
  expect(after.preScrollWidth).toBeLessThanOrEqual(after.preClientWidth + 1);
}

function expectClose(value: number, expected: number) {
  expect(Math.abs(value - expected)).toBeLessThanOrEqual(0.75);
}
