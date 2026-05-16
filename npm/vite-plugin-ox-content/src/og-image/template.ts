/**
 * Default OG image template.
 *
 * Uses inline HTML/CSS for a flat, low-color brand card with title,
 * description, siteName, and tags. No external dependencies required.
 */

import type { OgImageTemplateFn, OgImageTemplateProps } from "./types";

/**
 * Escapes HTML special characters.
 */
function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function normalizeBrandValue(str: string): string {
  return str.replace(/\s+/g, "").toLowerCase();
}

function renderWordmarkSvg(): string {
  return `<svg width="430" height="102" viewBox="0 0 270 64" fill="none" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="ogWordmarkGradient" x1="286" y1="10" x2="320" y2="54" gradientUnits="userSpaceOnUse">
      <stop offset="0%" stop-color="#355cff"/>
      <stop offset="100%" stop-color="#74c7ff"/>
    </linearGradient>
  </defs>
  <text
    x="2"
    y="43"
    fill="#eff6ff"
    font-family="IBM Plex Sans, IBM Plex Mono, Avenir Next, Segoe UI, sans-serif"
    font-size="34"
    font-weight="700"
    letter-spacing="-1.4"
  >
    OXCONTENT
  </text>
  <text
    x="213"
    y="43.5"
    fill="#eff6ff"
    font-family="IBM Plex Sans, IBM Plex Mono, Avenir Next, Segoe UI, sans-serif"
    font-size="40"
    font-weight="400"
  >
    (
  </text>
  <g transform="translate(216 9) scale(0.089) rotate(-7 256 256)">
    <path
      d="M161 96H286C298 96 309 101 318 110L352 144C361 153 366 164 366 176V386C366 399 355 410 342 410H161C148 410 138 399 138 386V120C138 107 148 96 161 96Z"
      fill="url(#ogWordmarkGradient)"
    />
  </g>
  <text
    x="252"
    y="43.5"
    fill="#eff6ff"
    font-family="IBM Plex Sans, IBM Plex Mono, Avenir Next, Segoe UI, sans-serif"
    font-size="40"
    font-weight="400"
  >
    )
  </text>
</svg>`;
}

/**
 * Returns the built-in default template function.
 */
export function getDefaultTemplate(): OgImageTemplateFn {
  return function defaultTemplate(props: OgImageTemplateProps): string {
    const { title, description, siteName } = props;
    const rawBrand = siteName?.trim() ? siteName : "Ox Content";
    const isBrandCard = normalizeBrandValue(title) === normalizeBrandValue(rawBrand);

    const heroTitle = isBrandCard ? "cargo doc for JavaScript" : title;
    const heroDescription = isBrandCard
      ? "Rust-powered docs and high-performance Markdown tooling."
      : description && description.trim().length > 0
        ? description
        : "Rust-powered docs and Markdown tooling.";
    const descriptionHtml =
      heroDescription.trim().length > 0
        ? `<p style="max-width:760px;font-size:28px;color:#93a4c3;line-height:1.45;margin:0;overflow:hidden;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;">${escapeHtml(heroDescription)}</p>`
        : "";

    return `<div style="width:100%;height:100%;position:relative;overflow:hidden;box-sizing:border-box;padding:56px 64px 52px;background:#0b1220;font-family:'IBM Plex Sans','Avenir Next','Segoe UI',system-ui,sans-serif;color:#eff6ff;border:1px solid #223252;border-top:4px solid #4f6fae;">
  <div style="position:relative;z-index:1;display:flex;flex-direction:column;height:100%;">
    <div style="display:flex;align-items:flex-start;">${renderWordmarkSvg()}</div>
    <div style="display:flex;flex-direction:column;justify-content:center;gap:24px;max-width:860px;flex:1;padding:22px 0 0;">
      <h1 style="font-size:78px;font-weight:700;color:#eff6ff;line-height:1.02;letter-spacing:-0.055em;margin:0;overflow:hidden;display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;">${escapeHtml(heroTitle)}</h1>
      ${descriptionHtml}
    </div>
  </div>
</div>`;
  };
}
