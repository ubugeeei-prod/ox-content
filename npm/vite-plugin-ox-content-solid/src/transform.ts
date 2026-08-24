import { transformMarkdown as baseTransformMarkdown } from "@ox-content/vite-plugin";
import { generateSolidModule } from "./codegen";
import { extractFrontmatter, injectIslandMarkers, scanComponents } from "./markdown";
import type { ResolvedSolidOptions, SolidTransformResult } from "./types";

export async function transformMarkdownWithSolid(
  code: string,
  id: string,
  options: ResolvedSolidOptions,
): Promise<SolidTransformResult> {
  // `frontmatter: false` means the body is taken verbatim, so a leading `---`
  // block stays in the rendered document instead of becoming module exports.
  const { content: markdownContent, frontmatter } = options.frontmatter
    ? extractFrontmatter(code)
    : { content: code, frontmatter: {} };
  const scanned = scanComponents(markdownContent, options.components);

  const transformed = await baseTransformMarkdown(scanned.content, id, createBaseOptions(options));

  const htmlWithIslands = injectIslandMarkers(transformed.html, scanned.islands);

  return {
    code: generateSolidModule(
      htmlWithIslands,
      scanned.usedComponents,
      scanned.islands,
      frontmatter,
      options,
      id,
    ),
    map: null,
    usedComponents: scanned.usedComponents,
    frontmatter,
  };
}

/**
 * Options handed to the core Markdown transform.
 *
 * The site-level features (SSG, search, OG images, highlighting) are turned off
 * here: this path only produces the HTML that gets embedded in a Solid module,
 * and the host app owns everything around it. Frontmatter is stripped before
 * this point, so the core parser sees a body-only document.
 */
function createBaseOptions(
  options: ResolvedSolidOptions,
): Parameters<typeof baseTransformMarkdown>[2] {
  return {
    srcDir: options.srcDir,
    outDir: options.outDir,
    base: options.base,
    extensions: options.extensions,
    ssg: {
      enabled: false,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
      pagination: false,
      breadcrumbs: false,
      readerChrome: false,
      localeSwitcher: false,
    },
    gfm: options.gfm,
    frontmatter: false,
    toc: options.toc,
    tocMaxDepth: options.tocMaxDepth,
    codeAnnotations: options.codeAnnotations,
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    autolinks: options.autolinks,
    highlight: false,
    mermaid: false,
    ogImage: false,
    ogImageOptions: {
      vuePlugin: "vitejs",
      width: 1200,
      height: 630,
      cache: true,
      concurrency: 1,
    },
    transformers: [],
    docs: false,
    ogViewer: false,
    search: {
      enabled: false,
      limit: 10,
      prefix: true,
      placeholder: "Search...",
      hotkey: "k",
    },
    embeds: options.embeds,
    i18n: false,
  } as unknown as Parameters<typeof baseTransformMarkdown>[2];
}
