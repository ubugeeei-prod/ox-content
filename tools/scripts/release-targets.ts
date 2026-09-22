import * as fs from "node:fs";
import * as path from "node:path";

const ROOT = path.resolve(import.meta.dirname, "../..");

// Packages to publish (relative to root)
export const NPM_PACKAGES = [
  "crates/ox_content_napi",
  "npm/ox-content-islands",
  "npm/ox-content-code-play",
  "npm/unplugin-ox-content",
  "npm/vite-plugin-ox-content",
  "npm/vite-plugin-ox-content-react",
  "npm/vite-plugin-ox-content-solid",
  "npm/vite-plugin-ox-content-svelte",
  "npm/vite-plugin-ox-content-vue",
  "npm/vscode-ox-content",
  // Theme presets are generated, so enumerate them rather than keep ~70 paths.
  ...["theme", "theme-color"].flatMap((g) =>
    fs.readdirSync(path.join(ROOT, "npm", g)).map((n) => `npm/${g}/${n}`),
  ),
];

export const CARGO_PUBLISH_PACKAGES = [
  "ox_content_allocator",
  "ox_content_ast",
  "ox_content_profiler",
  "ox_content_parser",
  "ox_content_mdast",
  "ox_content_renderer",
  "ox_content_incremental",
  "ox_content_og_image",
  "ox_content_transform",
  "ox_content_search",
  "ox_content_ssg",
  "ox_content_docs",
  "ox_content_vite",
];
