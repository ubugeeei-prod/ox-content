import { expect, test } from "@playwright/test";
import { isMulticolorIcon, renderIconsCss, type ResolvedIcon } from "../../src/icons-css";

const MONOCHROME_ICON_BODIES = {
  linkedin: [
    '<path fill="currentColor" d="M4 8h4v12H4z">',
    '<animate attributeName="opacity" values="0;1" dur=".2s" fill="freeze"/>',
    "</path>",
  ].join(""),
  twitter: [
    '<g fill="none" stroke="currentColor" stroke-width="2">',
    '<path d="M4 7c4 5 9 8 16 8">',
    '<animate attributeName="stroke-dashoffset" values="12;0" dur=".4s" fill="freeze"/>',
    "</path>",
    "</g>",
  ].join(""),
  rss: [
    '<mask id="line-md-rss-mask">',
    '<path fill="#fff" d="M3 3h18v18H3z"/>',
    '<path fill="#000" d="M6 6h3v3H6z"/>',
    "</mask>",
    '<path fill="currentColor" mask="url(#line-md-rss-mask)" d="M4 4h16v16H4z"/>',
  ].join(""),
  "download-outline": [
    '<g fill="none" stroke="currentColor" stroke-width="2">',
    '<path d="M12 3v12m0 0 4-4m-4 4-4-4">',
    '<animate attributeName="opacity" values="1;1" dur=".2s" fill="remove"/>',
    "</path>",
    "</g>",
  ].join(""),
  "github-loop": [
    '<mask id="line-md-github-mask">',
    '<circle cx="12" cy="12" r="9" fill="#fff"/>',
    "</mask>",
    '<path fill="currentColor" mask="url(#line-md-github-mask)" d="M4 12a8 8 0 1 0 16 0a8 8 0 0 0-16 0">',
    '<animateTransform attributeName="transform" type="rotate" from="0 12 12" to="360 12 12" dur="1s" fill="freeze"/>',
    "</path>",
  ].join(""),
} as const;

test("animated monochrome self-hosted icons follow light and dark currentColor", async ({
  page,
}) => {
  await page.setViewportSize({ width: 380, height: 190 });
  await page.setContent(renderProbe(), { waitUntil: "domcontentloaded" });

  const monoStyles = await page.locator("[data-kind='mono']").evaluateAll((nodes) =>
    nodes.map((node) => {
      const element = node as HTMLElement;
      const style = getComputedStyle(element);
      return {
        name: element.dataset.name,
        scheme: element.closest<HTMLElement>("[data-scheme]")?.dataset.scheme,
        color: style.color,
        backgroundColor: style.backgroundColor,
        backgroundImage: style.backgroundImage,
        maskImage: style.maskImage,
        webkitMaskImage: style.webkitMaskImage,
      };
    }),
  );

  expect(monoStyles).toHaveLength(Object.keys(MONOCHROME_ICON_BODIES).length * 2);
  for (const style of monoStyles) {
    expect(style.backgroundColor).toBe(style.color);
    expect(style.backgroundImage).toBe("none");
    expect(style.maskImage !== "none" || style.webkitMaskImage !== "none").toBe(true);
  }

  const palette = page.locator("[data-kind='multicolor']");
  await expect(palette).toHaveCSS("background-color", "rgba(0, 0, 0, 0)");
  await expect(palette).not.toHaveCSS("background-image", "none");
  await expect(page.locator(".icons-probe")).toHaveScreenshot(
    "self-hosted-icons-currentcolor.png",
    {
      animations: "disabled",
      maxDiffPixels: 0,
    },
  );
});

function renderProbe(): string {
  return `<!doctype html>
    <style>
      ${renderIconsCss(icons())}
      * { box-sizing: border-box; }
      body { margin: 0; background: #e5e7eb; }
      .icons-probe {
        display: grid;
        gap: 8px;
        width: 352px;
        padding: 12px;
        background: #e5e7eb;
      }
      .surface {
        display: flex;
        gap: 10px;
        padding: 10px;
      }
      .surface[data-scheme="light"] {
        color: #111827;
        background: #f9fafb;
      }
      .surface[data-scheme="dark"] {
        color: #ffffff;
        background: #111827;
      }
      .surface[data-scheme="palette"] {
        color: #ffffff;
        background: #111827;
      }
      .icon {
        flex: 0 0 auto;
        font-size: 32px;
        line-height: 1;
      }
    </style>
    <main class="icons-probe">
      ${renderIconRow("light")}
      ${renderIconRow("dark")}
      <section class="surface" data-scheme="palette">
        <span class="icon icon-[logos--palette]" data-kind="multicolor" aria-hidden="true"></span>
      </section>
    </main>`;
}

function renderIconRow(scheme: "light" | "dark"): string {
  return `<section class="surface" data-scheme="${scheme}">
    ${Object.keys(MONOCHROME_ICON_BODIES)
      .map(
        (name) =>
          `<span class="icon icon-[line-md--${name}]" data-kind="mono" data-name="${name}" aria-hidden="true"></span>`,
      )
      .join("")}
  </section>`;
}

function icons(): ResolvedIcon[] {
  const mono = Object.entries(MONOCHROME_ICON_BODIES).map(([name, body]) => ({
    prefix: "line-md",
    name,
    body,
    width: 24,
    height: 24,
    multicolor: isMulticolorIcon(body),
  }));
  return [
    ...mono,
    {
      prefix: "logos",
      name: "palette",
      body: '<path fill="#f43f5e" d="M3 3h8v8H3z"/><path fill="#22c55e" d="M13 13h8v8h-8z"/>',
      width: 24,
      height: 24,
      multicolor: true,
    },
  ];
}
