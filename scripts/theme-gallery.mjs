#!/usr/bin/env node
// Builds a self-contained preview gallery for every skin x color-scheme pair.
//
// The preview is not a mock-up: it inlines the real SSG base stylesheet, the
// real generated skin CSS, and the same color variables the Rust renderer
// emits, then renders the page inside an iframe so cascade and specificity
// behave exactly as they do in a built site.
//
// Usage: node scripts/theme-gallery.mjs [outFile]

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = join(HERE, "..");
// Lives in the docs site's static assets so the gallery ships with the docs
// rather than being a one-off build artifact.
const OUT = process.argv[2] ?? join(ROOT, "docs", "public", "theme-gallery.html");

const SSG_CSS = readFileSync(join(ROOT, "crates/ox_content_ssg/src/ssg.css"), "utf-8");
const ENTRY_CSS = readFileSync(join(ROOT, "crates/ox_content_ssg/src/entry.css"), "utf-8");

const skinsManifest = JSON.parse(readFileSync(join(HERE, "theme-skins/skins.json"), "utf-8"));
const palettes = JSON.parse(
  readFileSync(join(HERE, "theme-colors/palettes.json"), "utf-8"),
).palettes;

const camel = (id) => id.replace(/-([a-z0-9])/g, (_, c) => c.toUpperCase());

function pascal(id) {
  return id
    .split("-")
    .filter(Boolean)
    .map((part) => `${part[0]?.toUpperCase() ?? ""}${part.slice(1)}`)
    .join(" ");
}

/** Mirrors `generate_theme_css` in crates/ox_content_ssg/src/html/theme_css.rs. */
const COLOR_VARS = {
  primary: "color-primary",
  primaryHover: "color-primary-hover",
  background: "color-bg",
  backgroundAlt: "color-bg-alt",
  text: "color-text",
  textMuted: "color-text-muted",
  border: "color-border",
  codeBackground: "color-code-bg",
  codeBackgroundTop: "color-code-bg-top",
  codeText: "color-code-text",
};

const decls = (pairs, indent) =>
  pairs.map(([name, value]) => `${indent}--octc-${name}: ${value};`).join("\n");

function paletteCss(theme) {
  const light = [
    ...Object.entries(theme.colors).map(([k, v]) => [COLOR_VARS[k], v]),
    ...Object.entries(theme.tokens),
  ];
  const dark = [
    ...Object.entries(theme.darkColors).map(([k, v]) => [COLOR_VARS[k], v]),
    ...Object.entries(theme.darkTokens),
  ];
  // All three blocks, exactly as the Rust renderer emits them. The media-query
  // form is not optional: `:root:not([data-theme="light"])` outranks a bare
  // `[data-theme="dark"]`, so omitting it lets the base stylesheet's own dark
  // block win on an OS set to dark, and every scheme renders identically.
  return [
    `:root {\n${decls(light, "  ")}\n}`,
    `[data-theme="dark"] {\n${decls(dark, "  ")}\n}`,
    `@media (prefers-color-scheme: dark) {\n  :root:not([data-theme="light"]) {\n${decls(dark, "    ")}\n  }\n}`,
  ].join("\n");
}

function skinCss(skin) {
  const mod = readFileSync(join(ROOT, "npm", "theme", skin.id, "src", "skin.ts"), "utf-8");
  const start = mod.indexOf("export const css = `") + "export const css = `".length;
  const end = mod.lastIndexOf("`;");
  return mod
    .slice(start, end)
    .replace(/\\`/g, "`")
    .replace(/\\\$\{/g, "${")
    .replace(/\\\\/g, "\\");
}

/**
 * Builds a skin's WebGL backdrop from the same sources the package generator
 * uses, rather than scraping it back out of the generated module. Parsing a
 * emitted TypeScript file by string offsets broke the moment the formatter
 * rewrapped it, and there is no reason to read the output when the input is
 * right here.
 */
function skinJs(id) {
  const path = join(HERE, "theme-skins", "js", `${id}.js`);
  if (!existsSync(path)) return "";
  const runtime = readFileSync(join(HERE, "theme-skins", "js", "runtime.js"), "utf-8");
  return `(()=>{try{\n${runtime}\n${readFileSync(path, "utf-8")}\n}catch(e){console.warn("[ox-content] theme backdrop disabled:",e)}})();`;
}

function skinVars(skin) {
  const motion = Object.entries(skin.motion).map(([k, v]) => [`motion-${k}`, v]);
  return `:root {\n${decls(motion, "  ")}\n  --octc-font-sans: ${
    skinsManifest.fontStacks[skin.sans]
  };\n  --octc-font-mono: ${skinsManifest.fontStacks[skin.mono]};\n  --octc-sidebar-width: ${
    skin.layout.sidebarWidth
  };\n  --octc-header-height: ${skin.layout.headerHeight};\n  --octc-max-content-width: ${
    skin.layout.maxContentWidth
  };\n}`;
}

function readExportedTheme(src, exportName) {
  const marker = `export const ${exportName}`;
  const start = src.indexOf(marker);
  if (start < 0) throw new Error(`Missing theme export: ${exportName}`);

  const objectStart = src.indexOf("{", start);
  if (objectStart < 0) throw new Error(`Missing object literal for theme export: ${exportName}`);

  let depth = 0;
  let quote = "";
  let escaped = false;

  for (let i = objectStart; i < src.length; i += 1) {
    const ch = src[i];

    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (ch === "\\") {
        escaped = true;
      } else if (ch === quote) {
        quote = "";
      }
      continue;
    }

    if (ch === '"' || ch === "'" || ch === "`") {
      quote = ch;
      continue;
    }

    if (ch === "{") {
      depth += 1;
      continue;
    }

    if (ch === "}") {
      depth -= 1;
      if (depth === 0) {
        const literal = src.slice(objectStart, i + 1);
        return new Function(`return (${literal})`)();
      }
    }
  }

  throw new Error(`Unclosed object literal for theme export: ${exportName}`);
}

const NAV = [
  ["Getting started", ["Introduction", "Installation", "Quick start"]],
  ["Guide", ["Markdown", "Theming", "Plugins", "Deployment"]],
  ["Reference", ["Config", "CLI", "API"]],
];

const navHtml = NAV.map(
  ([title, items], gi) =>
    `<div class="nav-section"><div class="nav-title">${title}</div><ul class="nav-list">${items
      .map(
        (t, i) =>
          `<li class="nav-item"><a class="nav-link${gi === 1 && i === 1 ? " active" : ""}" href="#">${t}</a></li>`,
      )
      .join("")}</ul></div>`,
).join("");

const FEATURES = [
  ["Blazing fast", "Arena-allocated parser with zero-copy parsing."],
  ["mdast compatible", "Run existing remark and unified transforms."],
  ["Zero JavaScript", "Motion and transitions ship as declarative CSS."],
  ["Composable themes", "Pick a skin, pick a scheme, ship."],
];

const featuresHtml = FEATURES.map(
  ([t, d]) =>
    `<div class="feature-card"><div class="feature-body"><h3 class="feature-title">${t}</h3><p class="feature-details">${d}</p></div></div>`,
).join("");

// Uses the same --octc-syntax-* properties the real pipeline emits, so the
// preview shows the scheme's actual syntax colours rather than a stand-in.
const tok = (name, text) => `<span style="color:var(--octc-syntax-${name})">${text}</span>`;
const CODE = [
  `${tok("token-keyword", "import")} { oxContent } ${tok("token-keyword", "from")} ${tok("token-string-expression", '"@ox-content/vite-plugin"')};`,
  `${tok("token-keyword", "import")} ${tok("token-constant", "pixel")} ${tok("token-keyword", "from")} ${tok("token-string-expression", '"@ox-content/theme-pixel"')};`,
  `${tok("token-keyword", "import")} ${tok("token-constant", "tokyoNight")} ${tok("token-keyword", "from")} ${tok("token-string-expression", '"@ox-content/theme-color-tokyo-night"')};`,
  "",
  `${tok("token-comment", "// One skin, one colour scheme. Layers compose left to right.")}`,
  `${tok("token-keyword", "export default")} {`,
  `  plugins: [${tok("token-function", "oxContent")}({ ssg: { theme: [pixel, tokyoNight] } })],`,
  `};`,
].join("\n");

const shell = (body, extraClass = "") =>
  `<header class="header"><button class="menu-toggle" aria-label="Menu"><svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M3 12h18M3 6h18M3 18h18"/></svg></button><a href="#" class="header-title">Ox Content</a><div class="header-actions"><a class="social-link" href="#" aria-label="GitHub"><svg width="18" height="18" viewBox="0 0 24 24" fill="currentColor"><path d="M12 .5a12 12 0 0 0-3.8 23.4c.6.1.8-.3.8-.6v-2c-3.3.7-4-1.6-4-1.6-.6-1.4-1.4-1.8-1.4-1.8-1-.7 0-.7 0-.7 1.2.1 1.8 1.2 1.8 1.2 1 1.8 2.8 1.3 3.5 1 .1-.8.4-1.3.7-1.6-2.7-.3-5.5-1.3-5.5-5.9 0-1.3.5-2.4 1.2-3.2 0-.4-.5-1.6.1-3.2 0 0 1-.3 3.3 1.2a11.5 11.5 0 0 1 6 0C17.999 4.7 19 5 19 5c.6 1.6.2 2.8.1 3.2.8.8 1.2 1.9 1.2 3.2 0 4.6-2.8 5.6-5.5 5.9.4.4.8 1.1.8 2.2v3.3c0 .3.2.7.8.6A12 12 0 0 0 12 .5Z"/></svg></a><button class="search-button"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg><span>Search</span><kbd>&#8984;K</kbd></button><button class="theme-toggle" aria-label="Theme"><svg class="icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg><svg class="icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg></button></div></header><div class="layout${extraClass}">${body}</div>`;

const landing = shell(
  `<main class="main"><div class="entry-content"><section class="hero"><div class="hero-content"><h1 class="hero-name">Ox Content</h1><p class="hero-text">High-performance Markdown toolkit</p><p class="hero-tagline">A Rust-powered Markdown engine and static site generator for the JavaScript ecosystem.</p><div class="hero-actions"><a class="hero-action hero-action-brand" href="#">Get started</a><a class="hero-action hero-action-alt" href="#">GitHub</a></div></div></section><section class="features"><div class="features-grid">${featuresHtml}</div></section></div></main>`,
);

const article = shell(
  `<aside class="sidebar"><nav>${navHtml}</nav></aside><main class="main main--with-toc"><div class="content"><h1>Theming</h1><p>Ox Content splits appearance into two independent axes. A <strong>skin</strong> owns form — geometry, texture, typography and motion. A <strong>color scheme</strong> owns nothing but color. Compose them in either order.</p><h2>Installation</h2><p>Install one of each and list them as layers. Later layers win, so your own overrides go last.</p><pre><code>${CODE}</code></pre><h2>How composition works</h2><p>Every skin is written against <code>--octc-*</code> custom properties, so it never names a hue. Every scheme emits those properties and nothing else.</p><blockquote><p>Layers compose left to right. Object fields merge key-by-key; <code>css</code> concatenates.</p></blockquote><h3>Token reference</h3><table><thead><tr><th>Token</th><th>Owner</th><th>Purpose</th></tr></thead><tbody><tr><td><code>--octc-color-primary</code></td><td>Scheme</td><td>Accent for links and active states</td></tr><tr><td><code>--octc-motion-ease</code></td><td>Skin</td><td>Signature easing curve</td></tr><tr><td><code>--octc-surface-glass</code></td><td>Scheme</td><td>Translucent panel fill</td></tr></tbody></table><p>See the <a href="#">full reference</a> for every token.</p><hr/><p>Scroll to watch the reveal choreography, and use the browser back button to see the cross-document transition.</p></div></main><aside class="toc"><div class="toc-title">On this page</div><ul class="toc-list"><li class="toc-item"><a class="toc-link" href="#">Installation</a></li><li class="toc-item"><a class="toc-link" href="#">How composition works</a></li><li class="toc-item"><a class="toc-link toc-link--depth-3" href="#">Token reference</a></li></ul></aside>`,
);

const skins = skinsManifest.skins.map((s) => ({
  id: s.id,
  title: s.title,
  description: s.description,
  css: `${skinVars(s)}\n${skinCss(s)}`,
  js: skinJs(s.id),
  // Mirrors the skin's `embed.head`, so a preview that needs a webfont gets it.
  head: s.embedHead ?? "",
}));

// Color modules are TypeScript, so read the exported object literals directly
// rather than pulling a TS loader into this script. A package can expose named
// variants from the same module; each variant deserves its own gallery row.
const schemeData = palettes.flatMap((p) => {
  const src = readFileSync(join(ROOT, "npm", "theme-color", p.id, "src", "index.ts"), "utf-8");
  const entries = [
    {
      id: p.id,
      title: p.title,
      description: p.description,
      light: p.light,
      dark: p.dark,
      exportName: camel(p.id),
    },
    ...(p.variants ?? []).map((variant) => ({
      id: variant.name ?? `${p.id}-${variant.id}`,
      title: `${p.title} ${pascal(variant.id)}`,
      description: variant.description,
      light: p.light,
      dark: variant.dark,
      exportName: variant.exportName,
    })),
  ];

  return entries.map((entry) => {
    const theme = readExportedTheme(src, entry.exportName);
    return {
      id: entry.id,
      title: entry.title,
      description: entry.description,
      light: entry.light,
      dark: entry.dark,
      css: paletteCss(theme),
    };
  });
});

const DATA = { skins, schemes: schemeData, base: `${SSG_CSS}\n${ENTRY_CSS}`, landing, article };

const html = `<title>Ox Content Theme Gallery</title>
<style>${readFileSync(join(HERE, "theme-gallery.css"), "utf-8")}</style>
<div id="app"></div>
<script type="application/json" id="data">${JSON.stringify(DATA).replace(/</g, "\\u003c")}</script>
<script>${readFileSync(join(HERE, "theme-gallery.js"), "utf-8")}</script>
`;

const changed = !existsSync(OUT) || readFileSync(OUT, "utf-8") !== html;
if (changed) {
  writeFileSync(OUT, html);
}
console.log(
  `${changed ? "Wrote" : "Skipped unchanged"} ${OUT} (${(html.length / 1024).toFixed(0)} kB, ${skins.length} skins x ${schemeData.length} schemes = ${skins.length * schemeData.length} combinations)`,
);
