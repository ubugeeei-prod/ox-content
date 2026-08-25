#!/usr/bin/env node
// Regenerates every npm/theme-color-* package from palettes.json.
//
// A color package is pure data: fifteen hand-picked colors per mode, expanded
// here into the full --octc-* token surface so all schemes stay consistent with
// one another and a new scheme costs one JSON entry instead of a package.
//
// Usage: node scripts/theme-colors/generate.mjs

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { execFileSync } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const HERE = dirname(fileURLToPath(import.meta.url));
const ROOT = join(HERE, "..", "..");
const { palettes } = JSON.parse(readFileSync(join(HERE, "palettes.json"), "utf-8"));
const VERSION = JSON.parse(
  readFileSync(join(ROOT, "npm", "vite-plugin-ox-content", "package.json"), "utf-8"),
).version;

import { ensureContrast, tokensFor } from "./derive.mjs";
import { updateDocsPreview } from "./update-docs-preview.mjs";

const record = (obj, indent) =>
  Object.entries(obj)
    .map(([k, v]) => `${indent}"${k}": ${JSON.stringify(v)},`)
    .join("\n");

const colors = (c) => ({
  primary: c.primary,
  primaryHover: c.primaryHover,
  background: c.bg,
  backgroundAlt: c.bgAlt,
  text: c.text,
  textMuted: c.muted,
  border: c.border,
  codeBackground: c.codeBg,
  codeBackgroundTop: c.codeBg,
  codeText: c.codeText,
});

function indexTs(p, exportName) {
  return `import type { ThemeConfig } from "@ox-content/vite-plugin";

/**
 * ${p.title} — ${p.description}.
 *
 * Color only: no layout, no texture, no typography. Compose it under any
 * \`@ox-content/theme-*\` skin, or use it on its own over the default theme.
 *
 * Generated from \`scripts/theme-colors/palettes.json\`; edit that file and run
 * \`node scripts/theme-colors/generate.mjs\` rather than editing this by hand.
 */
export const ${exportName}: ThemeConfig = {
  name: "${p.id}",
  colors: {
${record(colors(p.light), "    ")}
  },
  darkColors: {
${record(colors(p.dark), "    ")}
  },
  tokens: {
${record(tokensFor(p.light, p.dark, "light"), "    ")}
  },
  darkTokens: {
${record(tokensFor(p.dark, p.light, "dark"), "    ")}
  },
};

export default ${exportName};
`;
}

function packageJson(p) {
  return {
    name: `@ox-content/theme-color-${p.id}`,
    version: VERSION,
    description: `${p.description} — an Ox Content color scheme that composes with any @ox-content/theme-* skin`,
    keywords: [
      "ox-content",
      "theme",
      "color-scheme",
      "palette",
      "ssg",
      "markdown",
      "documentation",
      p.id,
    ],
    license: "MIT",
    author: "ubugeeei",
    repository: {
      type: "git",
      url: "https://github.com/ubugeeei-prod/ox-content.git",
      directory: `npm/theme-color/${p.id}`,
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

function readme(p, exportName) {
  return `# @ox-content/theme-color-${p.id}

${p.title} — ${p.description} — for [Ox Content](https://github.com/ubugeeei-prod/ox-content).

**Color only.** Light and dark \`--octc-*\` tokens and nothing else: no layout, no
texture, no typography. That is what lets it drop under any skin package.

\`\`\`bash
npm install @ox-content/theme-color-${p.id}
\`\`\`

\`\`\`ts
// vite.config.ts
import { defineConfig } from "vite";
import { oxContent } from "@ox-content/vite-plugin";
import ${exportName} from "@ox-content/theme-color-${p.id}";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "docs",
      ssg: { siteName: "My Docs", theme: ${exportName} },
    }),
  ],
});
\`\`\`

Stack a skin on top to change the form as well — layers compose left to right,
so anything you append wins:

\`\`\`ts
import pixel from "@ox-content/theme-pixel";

theme: [pixel, ${exportName}, { colors: { primary: "#ff5f56" } }];
\`\`\`

## License

MIT
`;
}

const camel = (id) => id.replace(/-([a-z0-9])/g, (_, c) => c.toUpperCase());

// Link text and muted body copy must clear WCAG AA against their own page, so
// the authored accents are tightened here rather than in 23 hand-edited files.
for (const p of palettes) {
  for (const mode of ["light", "dark"]) {
    const c = p[mode];
    // Body text first: everything else is decoration if this fails.
    c.text = ensureContrast(c.text, c.bg, 4.5);
    c.codeText = ensureContrast(c.codeText, c.codeBg, 4.5);
    c.primary = ensureContrast(c.primary, c.bg, 4.5);
    c.primaryHover = ensureContrast(c.primaryHover, c.bg, 4.5);
    c.muted = ensureContrast(c.muted, c.bg, 4.5);
  }
}

for (const p of palettes) {
  const dir = join(ROOT, "npm", "theme-color", p.id);
  const exportName = camel(p.id);
  mkdirSync(join(dir, "src"), { recursive: true });
  writeFileSync(join(dir, "src", "index.ts"), indexTs(p, exportName));
  writeFileSync(join(dir, "package.json"), JSON.stringify(packageJson(p), null, 2) + "\n");
  writeFileSync(join(dir, "tsconfig.json"), JSON.stringify(TSCONFIG, null, 2) + "\n");
  writeFileSync(join(dir, "vite.config.ts"), VITE_CONFIG);
  writeFileSync(join(dir, "README.md"), readme(p, exportName));
}

updateDocsPreview();

// The repository formats its sources, and generated files are sources too.
// Formatting here rather than leaving it to the caller means a regeneration can
// never land unformatted — which is how this broke CI once already.
try {
  execFileSync("npx", ["vp", "fmt"], { cwd: ROOT, stdio: "ignore" });
} catch {
  console.warn("  (could not run `vp fmt` — format the output before committing)");
}

console.log(`Generated ${palettes.length} color packages into npm/theme-color-*`);
