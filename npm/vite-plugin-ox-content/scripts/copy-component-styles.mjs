#!/usr/bin/env node

/**
 * Copy crate SSG stylesheets into `dist/styles/` so
 * `@ox-content/vite-plugin/styles/*.css` is the same source the built-in SSG
 * inlines. Feature files that the SSG concatenates stay concatenated here.
 */

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const packageRoot = dirname(fileURLToPath(new URL("../package.json", import.meta.url)));
const ssgSrc = join(packageRoot, "../../crates/ox_content_ssg/src");
const outDir = join(packageRoot, "dist/styles");

/** @typedef {{ name: string, sources: string[] }} StyleEntry */

/** Public feature stylesheets. Filenames are package API. */
export const STYLE_ENTRIES = /** @type {const} */ ([
  { name: "core.css", sources: ["ssg.css"] },
  { name: "magic-links.css", sources: ["plugins/magic-links.css"] },
  {
    name: "social.css",
    sources: [
      "plugins/social.css",
      "plugins/provider-cards.css",
      "plugins/social-twitter-rich.css",
      "plugins/reddit.css",
    ],
  },
  {
    name: "twitter-full.css",
    sources: ["plugins/social-tweet-full.css", "plugins/social-tweet-full-media.css"],
  },
  { name: "ogp.css", sources: ["plugins/ogp.css"] },
  { name: "github.css", sources: ["plugins/github.css"] },
  { name: "markdown-tables.css", sources: ["plugins/markdown-tables.css"] },
  { name: "youtube.css", sources: ["plugins/youtube.css"] },
  { name: "tabs.css", sources: ["plugins/tabs.css"] },
  { name: "mermaid.css", sources: ["plugins/mermaid.css"] },
  { name: "graphviz.css", sources: ["plugins/graphviz.css"] },
  { name: "citations.css", sources: ["plugins/citations.css"] },
  { name: "not-by-ai.css", sources: ["plugins/not-by-ai.css"] },
]);

export const ALL_STYLESHEET = "all.css";

export const STYLE_EXPORT_NAMES = [...STYLE_ENTRIES.map((entry) => entry.name), ALL_STYLESHEET];

function crateCss(relativePath) {
  return readFileSync(join(ssgSrc, relativePath), "utf8");
}

function writeGenerated(name, contents) {
  writeFileSync(join(outDir, name), contents);
}

export function copyComponentStyles() {
  mkdirSync(outDir, { recursive: true });

  for (const entry of STYLE_ENTRIES) {
    writeGenerated(entry.name, entry.sources.map(crateCss).join(""));
  }

  const imports = STYLE_ENTRIES.map((entry) => `@import "./${entry.name}";`).join("\n");
  writeGenerated(ALL_STYLESHEET, `${imports}\n`);

  return {
    outDir,
    files: STYLE_EXPORT_NAMES.map((name) => join(outDir, name)),
  };
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  const { files } = copyComponentStyles();
  console.log(`Wrote ${files.length} component stylesheets to ${outDir}`);
}
