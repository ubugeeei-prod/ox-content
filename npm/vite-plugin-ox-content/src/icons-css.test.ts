import { describe, expect, it } from "vite-plus/test";
import { isMulticolorIcon, renderIconsCss, type ResolvedIcon } from "./icons-css";

const LINE_MD_REPRO_BODIES = {
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

describe("isMulticolorIcon", () => {
  it("treats animated line-md monochrome SVGs as mask-safe", () => {
    for (const body of Object.values(LINE_MD_REPRO_BODIES)) {
      expect(isMulticolorIcon(body)).toBe(false);
    }
  });

  it("keeps visible fixed paint and animated paint values multicolor", () => {
    expect(isMulticolorIcon('<path fill="#f43f5e" d="M0 0h24v24H0z"/>')).toBe(true);
    expect(
      isMulticolorIcon(
        '<animate attributeName="fill" values="currentColor;#22c55e" dur=".2s" fill="freeze"/>',
      ),
    ).toBe(true);
  });
});

describe("renderIconsCss", () => {
  it("emits animated monochrome line-md icons as currentColor masks", () => {
    const css = renderIconsCss(lineMdIcons());
    for (const name of Object.keys(LINE_MD_REPRO_BODIES)) {
      const rule = iconRule(css, `line-md--${name}`);
      expect(rule).toContain("background-color:currentColor");
      expect(rule).toContain("mask-image:");
      expect(rule).not.toContain("background-image:");
    }
    expect(css).toContain("fill='freeze'");
    expect(css).toContain("fill='remove'");
  });

  it("emits fixed-color icons as background images", () => {
    const css = renderIconsCss([
      {
        prefix: "logos",
        name: "palette",
        body: '<path fill="#f43f5e" d="M3 3h8v8H3z"/><path fill="#22c55e" d="M13 13h8v8h-8z"/>',
        width: 24,
        height: 24,
        multicolor: true,
      },
    ]);

    const rule = iconRule(css, "logos--palette");
    expect(rule).toContain("background-color:transparent");
    expect(rule).toContain("background-image:");
    expect(rule).not.toContain("mask-image:");
  });
});

function lineMdIcons(): ResolvedIcon[] {
  return Object.entries(LINE_MD_REPRO_BODIES).map(([name, body]) => ({
    prefix: "line-md",
    name,
    body,
    width: 24,
    height: 24,
    multicolor: isMulticolorIcon(body),
  }));
}

function iconRule(css: string, name: string): string {
  const selector = `.icon-\\[${name}\\]`;
  const start = css.indexOf(selector);
  expect(start).toBeGreaterThanOrEqual(0);
  const end = css.indexOf("\n", start);
  return css.slice(start, end === -1 ? css.length : end);
}
