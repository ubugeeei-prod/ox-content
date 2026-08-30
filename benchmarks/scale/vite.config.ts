/**
 * Config for the scale benchmark.
 *
 * `run.mjs` drives this through `OX_SCALE_*` so the config stays tracked and
 * reviewable while one process can measure many option sets. Every flag
 * defaults to off, so a suite states exactly what it is paying for.
 */
import { defineConfig } from "vite-plus";
import { oxContent } from "@ox-content/vite-plugin";

const env = (globalThis as { process?: { env?: Record<string, string | undefined> } }).process?.env;

const on = (name: string) => env?.[`OX_SCALE_${name}`] === "1";

export default defineConfig({
  plugins: [
    oxContent({
      srcDir: "content",
      outDir: env?.OX_SCALE_OUT_DIR ?? "dist",
      ssg: { enabled: true, bare: !on("THEME") },
      highlight: on("HIGHLIGHT"),
      gfm: true,
      tables: true,
      footnotes: true,
      taskLists: on("TASK_LISTS"),
      strikethrough: on("STRIKETHROUGH"),
      autolinks: on("AUTOLINKS"),
      mdx: on("MDX"),
      math: on("MATH"),
      attrs: on("ATTRS"),
      wikiLinks: on("WIKI_LINKS"),
      emojiShortcodes: on("EMOJI"),
      containers: on("CONTAINERS"),
      images: on("IMAGES"),
      imageGalleries: on("IMAGE_GALLERIES"),
      timelines: on("TIMELINES"),
      definitionLists: on("DEFINITION_LISTS"),
      keyboardKeys: on("KEYBOARD_KEYS"),
      abbreviations: on("ABBREVIATIONS"),
      magicLinks: on("MAGIC_LINKS"),
      badges: on("BADGES"),
      cards: on("CARDS"),
      steps: on("STEPS"),
      codeGroups: on("CODE_GROUPS"),
      fileTree: on("FILE_TREE"),
      dataTables: on("DATA_TABLES"),
      conditionalBlocks: on("CONDITIONAL_BLOCKS"),
      cjkEmphasis: on("CJK_EMPHASIS"),
      codeAnnotations: on("CODE_ANNOTATIONS"),
      semanticFootnotes: on("SEMANTIC_FOOTNOTES"),
      search: on("SEARCH"),
      siteMaps: on("SITE_MAPS"),
      feeds: on("FEEDS"),
      ogImage: on("OG_IMAGE"),
      ogImageOptions: env?.OX_SCALE_OG_CONCURRENCY
        ? { concurrency: Number(env.OX_SCALE_OG_CONCURRENCY) }
        : undefined,
    }),
  ],
  build: {
    outDir: env?.OX_SCALE_OUT_DIR ?? "dist",
    minify: false,
  },
  logLevel: "error",
});
