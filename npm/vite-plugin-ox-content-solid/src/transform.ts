import {
  applyIslandSsrHtml,
  discoverDocumentMdxIslands,
  resolveContentRootPath,
  resolveMdxForFilePath,
  transformMarkdown as baseTransformMarkdown,
} from "@ox-content/vite-plugin";
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
  const mdx = resolveMdxForFilePath(id, options.mdx);
  const baseOptions = createBaseOptions(options, mdx);

  if (mdx) {
    const transformed = await baseTransformMarkdown(markdownContent, id, baseOptions);
    const discovered = await discoverDocumentMdxIslands({
      source: markdownContent,
      html: transformed.html,
      components: options.components,
      imports: transformed.imports,
      documentPath: id,
      contentRoot: resolveContentRootPath({
        srcDir: options.srcDir,
        root: options.root,
      }),
      srcDir: options.srcDir,
    });
    const html = options.renderIsland
      ? await applyIslandSsrHtml(
          transformed.html,
          options.renderIsland,
          id,
          discovered.usedComponents,
        )
      : transformed.html;
    return {
      code: generateSolidModule(
        html,
        discovered.usedComponents,
        discovered.usedComponents,
        frontmatter,
        options,
        id,
        discovered.localBindings,
      ),
      map: null,
      usedComponents: discovered.usedComponents,
      frontmatter,
    };
  }

  const scanned = scanComponents(markdownContent, options.components);
  const transformed = await baseTransformMarkdown(scanned.content, id, baseOptions);
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
  mdx: boolean,
): Parameters<typeof baseTransformMarkdown>[2] {
  return {
    srcDir: options.srcDir,
    outDir: options.outDir,
    base: options.base,
    extensions: options.extensions,
    mdx,
    ssg: {
      enabled: false,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
      pagination: false,
      breadcrumbs: false,
      jsonLd: false,
      readerChrome: false,
      localeSwitcher: false,
      a11y: false,
      pageChrome: false,
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
