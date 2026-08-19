#!/usr/bin/env node
// Regenerates every npm/theme-* skin package from skins.json + skins/<id>.css.
//
// Skins are authored as real stylesheets so editors can lint and format them;
// this script inlines each one (after the shared motion layer) into a plain
// data module, which is why the published package has no runtime dependency.
//
// Usage: node scripts/theme-skins/generate.mjs

import { existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = join(HERE, "..", "..");
const manifest = JSON.parse(readFileSync(join(HERE, "skins.json"), "utf-8"));
const MOTION_CSS = readFileSync(join(HERE, "motion.css"), "utf-8");
const HERO_CSS = readFileSync(join(HERE, "hero.css"), "utf-8");
const DETAIL_CSS = readFileSync(join(HERE, "details.css"), "utf-8");
const GUARD_CSS = readFileSync(join(HERE, "guards.css"), "utf-8");
const GL_RUNTIME = readFileSync(join(HERE, "js", "runtime.js"), "utf-8");
const VERSION = JSON.parse(
  readFileSync(join(ROOT, "npm", "vite-plugin-ox-content", "package.json"), "utf-8"),
).version;

const camel = (id) => id.replace(/-([a-z0-9])/g, (_, c) => c.toUpperCase());

/** Escapes a stylesheet for embedding in a TypeScript template literal. */
const escapeCss = (css) => css.replace(/\\/g, "\\\\").replace(/`/g, "\\`").replace(/\$\{/g, "\\${");

/** Strips comments and collapses whitespace — themes ship as one inline blob. */
function minify(css) {
  return (
    css
      .replace(/\/\*[\s\S]*?\*\//g, "")
      .replace(/\s*\n\s*/g, "\n")
      .replace(/\n{2,}/g, "\n")
      .replace(/\s*([{};,>])\s*/g, "$1")
      // `:` is deliberately excluded above: the space in `.content :not(pre)` is
      // a descendant combinator, and collapsing it silently rewrites the
      // selector. Only tighten a colon that directly follows a word, which is
      // always a declaration (`color: red`) or a pseudo (`a:hover`).
      .replace(/(\w):\s*/g, "$1:")
      .replace(/;}/g, "}")
      .trim()
  );
}

/**
 * Reads a skin's optional WebGL2 backdrop and wraps it with the shared runtime.
 *
 * The SSG concatenates every theme's `js` into one classic inline script, so the
 * whole thing is scoped in an IIFE and must not rely on module resolution.
 */
function skinJs(skin) {
  const path = join(HERE, "js", `${skin.id}.js`);
  if (!existsSync(path)) return "";
  // The theme's script is appended after the SSG's own, so a throw here cannot
  // take out search or the theme toggle — but it would still surface as a
  // console error on a page that is meant to degrade silently.
  // Failures stay non-fatal — the CSS backdrop underneath is the design — but
  // they are reported. Swallowing them silently is how a backdrop that mounted
  // and never drew went unnoticed.
  return `(()=>{try{\n${GL_RUNTIME}\n${readFileSync(path, "utf-8")}\n}catch(e){console.warn("[ox-content] theme backdrop disabled:",e)}})();`;
}

/**
 * A skin is either `skins/<id>.css` or a `skins/<id>/` directory of parts read
 * in filename order. The directory form exists so a skin with a genuinely large
 * idea can be split along its own seams rather than squeezed under the
 * repository's per-file line limit.
 */
function readSkinCss(id) {
  const dir = join(HERE, "skins", id);
  if (existsSync(dir) && statSync(dir).isDirectory()) {
    return readdirSync(dir)
      .filter((f) => f.endsWith(".css"))
      .sort()
      .map((f) => readFileSync(join(dir, f), "utf-8"))
      .join("\n");
  }
  return readFileSync(join(HERE, "skins", `${id}.css`), "utf-8");
}

function glTs(skin, js) {
  const doc = [
    "/**",
    ` * ${skin.title} WebGL2 hero backdrop.`,
    " *",
    " * Progressive enhancement only: it bails out under prefers-reduced-motion,",
    " * on a missing WebGL2 context, and whenever the hero scrolls out of view.",
    " * The CSS backdrop underneath is the design; this layers on top of it.",
    " *",
    ` * Generated from scripts/theme-skins/js/${skin.id}.js; edit that file and`,
    " * run node scripts/theme-skins/generate.mjs rather than editing this by hand.",
    " */",
  ].join("\n");
  return `${doc}\nexport const js = ${JSON.stringify(js)};\n`;
}

function skinTs(skin, css) {
  return `/**
 * ${skin.title} skin stylesheet.
 *
 * ${skin.description}.
 *
 * Written entirely against \`--octc-*\` custom properties and \`color-mix()\`, plus
 * neutral black/white alpha for depth — no hue is ever named, which is what lets
 * any \`@ox-content/theme-color-*\` package drive it.
 *
 * Generated from \`scripts/theme-skins/skins/${skin.id}.css\`; edit that file and
 * run \`node scripts/theme-skins/generate.mjs\` rather than editing this by hand.
 */
export const css = \`${escapeCss(css)}\`;
`;
}

function indexTs(skin, exportName, fonts, hasJs) {
  const motion = Object.fromEntries(
    Object.entries(skin.motion).map(([k, v]) => [`motion-${k}`, v]),
  );
  const entries = (obj, indent) =>
    Object.entries(obj)
      .map(([k, v]) => `${indent}${JSON.stringify(k)}: ${JSON.stringify(v)},`)
      .join("\n");

  return `import type { ThemeConfig } from "@ox-content/vite-plugin";

import { css } from "./skin";${hasJs ? '\nimport { js } from "./gl";' : ""}

/**
 * ${skin.title} — ${skin.description}.
 *
 * Form only: geometry, texture, typography and motion. It names no colors, so
 * it composes with any \`@ox-content/theme-color-*\` package:
 *
 * \`\`\`ts
 * theme: [${exportName}, tokyoNight]
 * \`\`\`
 */
export const ${exportName}: ThemeConfig = {
  name: "${skin.id}",
  fonts: {
    sans: ${JSON.stringify(fonts.sans)},
    mono: ${JSON.stringify(fonts.mono)},
  },
  layout: {
${entries(skin.layout, "    ")}
  },
  entryPage: { mode: ${JSON.stringify(skin.entryPage ?? "default")} },${
    skin.embedHead ? `\n  embed: { head: ${JSON.stringify(skin.embedHead)} },` : ""
  }
  tokens: {
${entries(motion, "    ")}
  },
  css,${hasJs ? "\n  js," : ""}
};

export default ${exportName};
`;
}

function packageJson(skin) {
  return {
    name: `@ox-content/theme-${skin.id}`,
    version: VERSION,
    description: `${skin.description} — an Ox Content skin that composes with any @ox-content/theme-color-* scheme`,
    keywords: ["ox-content", "theme", "skin", "ssg", "markdown", "documentation", ...skin.keywords],
    license: "MIT",
    author: "ubugeeei",
    repository: {
      type: "git",
      url: "https://github.com/ubugeeei-prod/ox-content.git",
      directory: `npm/theme/${skin.id}`,
    },
    files: ["dist"],
    type: "module",
    main: "./dist/index.cjs",
    types: "./dist/index.d.mts",
    exports: {
      ".": {
        import: "./dist/index.mjs",
        require: "./dist/index.cjs",
        types: "./dist/index.d.mts",
      },
    },
    sideEffects: false,
    publishConfig: { access: "public", provenance: true },
    scripts: { build: "vp pack", dev: "vp pack --watch", typecheck: "tsgo --noEmit" },
    devDependencies: {
      "@ox-content/vite-plugin": "workspace:*",
      "@types/node": "catalog:",
      "@typescript/native-preview": "catalog:",
      typescript: "catalog:",
      "vite-plus": "catalog:",
    },
    peerDependencies: { "@ox-content/vite-plugin": ">=2.84.0" },
    peerDependenciesMeta: { "@ox-content/vite-plugin": { optional: true } },
  };
}

const TSCONFIG = {
  compilerOptions: {
    target: "ES2022",
    module: "ESNext",
    moduleResolution: "bundler",
    strict: true,
    esModuleInterop: true,
    skipLibCheck: true,
    declaration: true,
    declarationMap: true,
    outDir: "dist",
    rootDir: "src",
    types: ["node"],
  },
  include: ["src/**/*.ts"],
  exclude: ["node_modules", "dist"],
};

const VITE_CONFIG = `import { defineConfig } from "vite-plus";
import { defineConfig as definePackConfig } from "vite-plus/pack";

export default defineConfig({
  fmt: {
    ignorePatterns: ["dist/**"],
  },
  pack: definePackConfig({
    entry: ["src/index.ts"],
    format: ["esm", "cjs"],
    dts: true,
    clean: true,
    hash: false,
    deps: {
      neverBundle: ["@ox-content/vite-plugin"],
    },
  }),
});
`;

function readme(skin, exportName, bytes) {
  return `# @ox-content/theme-${skin.id}

${skin.title} — ${skin.description} — for [Ox Content](https://github.com/ubugeeei-prod/ox-content).

**Form only.** Geometry, texture, typography and motion, written entirely
against \`--octc-*\` custom properties. It names no colors, so it pairs with any
\`@ox-content/theme-color-*\` scheme. About ${bytes} of CSS, zero JavaScript, zero
runtime dependencies.

\`\`\`bash
npm install @ox-content/theme-${skin.id} @ox-content/theme-color-tokyo-night
\`\`\`

\`\`\`ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import ${exportName} from "@ox-content/theme-${skin.id}";
import tokyoNight from "@ox-content/theme-color-tokyo-night";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      ssg: { siteName: "My Docs", theme: [${exportName}, tokyoNight] },
    }),
  ],
});
\`\`\`

Layers compose left to right, so anything you append wins:

\`\`\`ts
theme: [${exportName}, tokyoNight, { colors: { primary: "#ff5f56" } }];
\`\`\`

## Motion

Transitions, scroll-driven reveals and cross-document page transitions are all
declarative CSS — no router, no observer, no client bundle. Everything sits
behind \`@supports\` and is switched off under \`prefers-reduced-motion: reduce\`.

Retune the choreography without touching the stylesheet:

\`\`\`ts
theme: [
  ${exportName},
  tokyoNight,
  { tokens: { "motion-base": "200ms", "motion-ease": "linear" } },
];
\`\`\`

## License

MIT
`;
}

let total = 0;
for (const skin of manifest.skins) {
  const exportName = camel(skin.id);
  const dir = join(ROOT, "npm", "theme", skin.id);
  const own = readSkinCss(skin.id);
  const css = minify(`${MOTION_CSS}\n${HERO_CSS}\n${DETAIL_CSS}\n${own}\n${GUARD_CSS}`);
  const bytes = `${(css.length / 1024).toFixed(1)} kB`;
  total += css.length;

  mkdirSync(join(dir, "src"), { recursive: true });
  writeFileSync(join(dir, "src", "skin.ts"), skinTs(skin, css));
  const js = skinJs(skin);
  if (js) {
    writeFileSync(join(dir, "src", "gl.ts"), glTs(skin, js));
  }
  writeFileSync(
    join(dir, "src", "index.ts"),
    indexTs(
      skin,
      exportName,
      { sans: manifest.fontStacks[skin.sans], mono: manifest.fontStacks[skin.mono] },
      Boolean(js),
    ),
  );
  writeFileSync(join(dir, "package.json"), JSON.stringify(packageJson(skin), null, 2) + "\n");
  writeFileSync(join(dir, "tsconfig.json"), JSON.stringify(TSCONFIG, null, 2) + "\n");
  writeFileSync(join(dir, "vite.config.ts"), VITE_CONFIG);
  writeFileSync(join(dir, "README.md"), readme(skin, exportName, bytes));
  console.log(`  ${skin.id.padEnd(14)} ${bytes.padStart(8)}`);
}

// The repository formats its sources, and generated files are sources too.
// Formatting here rather than leaving it to the caller means a regeneration can
// never land unformatted — which is how this broke CI once already.
try {
  execFileSync("npx", ["vp", "fmt"], { cwd: ROOT, stdio: "ignore" });
} catch {
  console.warn("  (could not run `vp fmt` — format the output before committing)");
}

console.log(
  `Generated ${manifest.skins.length} skin packages (${(total / manifest.skins.length / 1024).toFixed(1)} kB average)`,
);
