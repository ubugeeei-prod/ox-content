#!/usr/bin/env node
// Regenerates the color-scheme preview blocks in the Theme Presets docs.

import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = join(HERE, "..", "..");
const START = "<!-- ox-content:theme-color-previews:start -->";
const END = "<!-- ox-content:theme-color-previews:end -->";
const { palettes } = JSON.parse(readFileSync(join(HERE, "palettes.json"), "utf-8"));

const DOCS = [
  {
    file: join(ROOT, "docs/content/theme-presets.md"),
    mode: { light: "Light", dark: "Dark" },
    summary: () =>
      `Preview every color package as paired light and dark surfaces. Each card samples the page background, text, link accent, code background, and syntax accent set for one scheme; open the gallery to try it with any skin.`,
    label: (title) => `${title} color scheme preview`,
  },
  {
    file: join(ROOT, "docs/content/ja/theme-presets.md"),
    mode: { light: "ライト", dark: "ダーク" },
    summary: () =>
      `すべての配色パッケージを、ライトとダークの対で見られる preview です。各カードはページ背景、本文、リンクアクセント、コード背景、シンタックスアクセントを抜き出しています。組み合わせはギャラリーで試せます。`,
    label: (title) => `${title} 配色 preview`,
  },
];

const html = (value) =>
  String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");

const STYLE = `<style>
.theme-color-previews{display:grid;grid-template-columns:repeat(auto-fit,minmax(min(100%,230px),1fr));gap:10px;margin:1.25rem 0 1.5rem}
.theme-color-preview{display:grid;grid-template-rows:auto minmax(0,1fr);overflow:hidden;text-decoration:none;color:inherit;border:1px solid var(--octc-color-border,#d7deea);border-radius:8px;background:var(--octc-color-bg,#fff);min-width:0}
.theme-color-preview__head{display:block;padding:.68rem .78rem;border-bottom:1px solid var(--octc-color-border,#d7deea);min-width:0}
.theme-color-preview__name{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:.9rem;line-height:1.25;color:var(--octc-color-text,currentColor)}
.theme-color-preview__pkg{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;margin-top:.18rem;font-size:.68rem;line-height:1.35;color:var(--octc-color-text-muted,#64708a)}
.theme-color-preview__modes{display:grid;grid-template-columns:repeat(2,minmax(0,1fr));min-height:122px}
.theme-color-preview__mode{display:grid;gap:.45rem;align-content:start;padding:.7rem;background:linear-gradient(135deg,var(--p-bg) 0%,var(--p-bg) 58%,var(--p-bg-alt) 58%,var(--p-bg-alt) 100%);color:var(--p-text);min-width:0}
.theme-color-preview__mode-head{display:flex;align-items:center;justify-content:space-between;gap:.5rem}
.theme-color-preview__mode-label{font-size:.73rem;line-height:1.1}
.theme-color-preview__dot,.theme-color-preview__swatch{display:block;width:15px;height:15px;border:1px solid var(--p-border);background:var(--swatch)}
.theme-color-preview__dot{border-radius:999px;background:var(--p-primary)}
.theme-color-preview__lines{display:grid;gap:.26rem}
.theme-color-preview__line{display:block;height:7px;border-radius:999px;background:var(--p-text);opacity:.9}
.theme-color-preview__line--muted{width:62%;background:var(--p-muted)}
.theme-color-preview__line--main{width:88%}
.theme-color-preview__swatches{display:flex;gap:3px;flex-wrap:wrap;margin-top:auto}
.theme-color-preview__swatch{border-radius:4px}
.theme-color-preview__code{display:block;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;border:1px solid var(--p-border);border-radius:6px;background:var(--p-code-bg);color:var(--p-code-text);padding:.28rem .36rem;font:600 .68rem/1.35 ui-monospace,SFMono-Regular,Menlo,monospace}
@media (max-width:520px){.theme-color-preview__modes{grid-template-columns:1fr}}
</style>`;

const modeVars = (mode) =>
  ["bg", "bg-alt", "text", "muted", "border", "primary", "code-bg", "code-text"]
    .map((key) => `--p-${key}:var(--${mode}-${key})`)
    .join(";");

const swatch = (mode, key) =>
  `<i class="theme-color-preview__swatch" aria-hidden="true" style="--swatch:var(--${mode}-${key})"></i>`;

function modePanel(mode, label) {
  return `<span class="theme-color-preview__mode" style="${modeVars(mode)}"><span class="theme-color-preview__mode-head"><b class="theme-color-preview__mode-label">${html(label)}</b><i class="theme-color-preview__dot" aria-hidden="true"></i></span><span class="theme-color-preview__lines"><i class="theme-color-preview__line theme-color-preview__line--main" aria-hidden="true"></i><i class="theme-color-preview__line theme-color-preview__line--muted" aria-hidden="true"></i></span><span class="theme-color-preview__swatches">${["red", "green", "yellow", "blue", "magenta", "cyan"].map((key) => swatch(mode, key)).join("")}</span><span class="theme-color-preview__code" aria-hidden="true">theme()</span></span>`;
}

function card(palette, doc) {
  const vars = [
    ["l-bg", palette.light.bg],
    ["l-bg-alt", palette.light.bgAlt],
    ["l-text", palette.light.text],
    ["l-muted", palette.light.muted],
    ["l-border", palette.light.border],
    ["l-primary", palette.light.primary],
    ["l-code-bg", palette.light.codeBg],
    ["l-code-text", palette.light.codeText],
    ["l-red", palette.light.red],
    ["l-green", palette.light.green],
    ["l-yellow", palette.light.yellow],
    ["l-blue", palette.light.blue],
    ["l-magenta", palette.light.magenta],
    ["l-cyan", palette.light.cyan],
    ["d-bg", palette.dark.bg],
    ["d-bg-alt", palette.dark.bgAlt],
    ["d-text", palette.dark.text],
    ["d-muted", palette.dark.muted],
    ["d-border", palette.dark.border],
    ["d-primary", palette.dark.primary],
    ["d-code-bg", palette.dark.codeBg],
    ["d-code-text", palette.dark.codeText],
    ["d-red", palette.dark.red],
    ["d-green", palette.dark.green],
    ["d-yellow", palette.dark.yellow],
    ["d-blue", palette.dark.blue],
    ["d-magenta", palette.dark.magenta],
    ["d-cyan", palette.dark.cyan],
  ]
    .map(([key, value]) => `--${key}:${value}`)
    .join(";");
  const pkg = `@ox-content/theme-color-${palette.id}`;

  return `  <a class="theme-color-preview" data-theme-color-preview="${html(palette.id)}" href="/theme-gallery.html" aria-label="${html(doc.label(palette.title))}" style="${vars}">
    <span class="theme-color-preview__head"><strong class="theme-color-preview__name">${html(palette.title)}</strong><small class="theme-color-preview__pkg">${html(pkg)}</small></span>
    <span class="theme-color-preview__modes">${modePanel("l", doc.mode.light)}${modePanel("d", doc.mode.dark)}</span>
  </a>`;
}

function block(doc) {
  return `${START}

${doc.summary(palettes.length)}

${STYLE}

<div class="theme-color-previews" data-theme-color-previews>
${palettes.map((palette) => card(palette, doc)).join("\n")}
</div>

${END}`;
}

function replaceBlock(source, next) {
  const start = source.indexOf(START);
  const end = source.indexOf(END);
  if (start === -1 || end === -1 || end < start) {
    throw new Error("Theme color preview markers are missing.");
  }
  return `${source.slice(0, start)}${next}${source.slice(end + END.length)}`;
}

export function updateDocsPreview() {
  for (const doc of DOCS) {
    const source = readFileSync(doc.file, "utf-8");
    writeFileSync(doc.file, replaceBlock(source, block(doc)));
  }
  console.log(`Updated theme color previews for ${palettes.length} schemes`);
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  updateDocsPreview();
}
