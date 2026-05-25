/**
 * SSG (Static Site Generation) module for ox-content
 */

import * as fs from "fs/promises";
import * as path from "path";
import { transformMarkdown } from "./transform";
import { generateOgImages } from "./og-image";
import type { OgImagePageEntry } from "./og-image";
import { transformAllPlugins } from "./plugins";
import type { TransformAllOptions } from "./plugins";
import { protectMermaidSvgs, restoreMermaidSvgs } from "./plugins/mermaid-protect";
import { transformIslands, hasIslands } from "./island";
import { importNapiModule, importNapiModuleSync } from "./napi";
import { DEFAULT_MARKDOWN_EXTENSIONS } from "./markdown";
import type {
  ResolvedOptions,
  ResolvedSsgOptions,
  SsgOptions,
  SsgNavigationGroup,
  TocEntry,
  HeroConfig,
  FeatureConfig,
  LocaleConfig,
} from "./types";
import { resolveTheme, themeToNapi } from "./theme";
import type { ResolvedThemeConfig, SidebarItem } from "./theme";
import { normalizeVitePressFrontmatter } from "./vitepress";

/**
 * Navigation item for SSG.
 */
export interface SsgNavItem {
  title: string;
  path: string;
  href: string;
  children?: SsgNavItem[];
  collapsed?: boolean;
}

/**
 * Entry page configuration for SSG (passed to Rust).
 */
export interface SsgEntryPageConfig {
  hero?: HeroConfig;
  features?: FeatureConfig[];
}

/**
 * Page data for SSG.
 */
export interface SsgPageData {
  title: string;
  description?: string;
  content: string;
  toc: TocEntry[];
  lastUpdated?: number;
  frontmatter: Record<string, unknown>;
  path: string;
  href: string;
  /** Entry page configuration (if layout: entry) */
  entryPage?: SsgEntryPageConfig;
}

interface SsgRoutePaths {
  outputPath: string;
  urlPath: string;
  href: string;
  ogImagePath: string;
  ogImageUrl: string;
}

/**
 * Deprecated compatibility export for consumers that imported the former
 * TypeScript SSG template. HTML generation is Rust-backed now.
 *
 * @deprecated Use `generateHtmlPage`/`buildSsg` instead.
 */
export const DEFAULT_HTML_TEMPLATE = "<!-- ox-content default HTML template is Rust-backed -->";

/**
 * Resolves SSG options with defaults.
 */
export function resolveSsgOptions(ssg: SsgOptions | boolean | undefined): ResolvedSsgOptions {
  if (ssg === false) {
    return {
      enabled: false,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
    };
  }

  if (ssg === true || ssg === undefined) {
    return {
      enabled: true,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
      theme: resolveTheme(undefined),
    };
  }

  return {
    enabled: ssg.enabled ?? true,
    extension: ssg.extension ?? ".html",
    clean: ssg.clean ?? false,
    bare: ssg.bare ?? false,
    siteName: ssg.siteName,
    ogImage: ssg.ogImage,
    generateOgImage: ssg.generateOgImage ?? false,
    lastUpdated: ssg.lastUpdated ?? false,
    siteUrl: ssg.siteUrl,
    theme: resolveTheme(ssg.theme),
    navigation: ssg.navigation,
  };
}

/**
 * Extracts title from content or frontmatter.
 */
export function extractTitle(content: string, frontmatter: Record<string, unknown>): string {
  return importNapiModuleSync().extractSsgTitle(
    content,
    typeof frontmatter.title === "string" ? frontmatter.title : undefined,
  );
}

/**
 * Generates bare HTML page (no navigation, no styles).
 */
export function generateBareHtmlPage(content: string, title: string): string {
  return importNapiModuleSync().generateSsgBareHtml(content, title);
}

/**
 * Generates HTML page with navigation using Rust NAPI bindings.
 */
export async function generateHtmlPage(
  pageData: SsgPageData,
  navGroups: NavGroup[],
  siteName: string,
  base: string,
  ogImage?: string,
  theme?: ResolvedThemeConfig,
  locale?: string,
  availableLocales?: LocaleConfig[],
): Promise<string> {
  const mod = await importNapiModule();

  // Convert TocEntry to the format expected by Rust
  const toRustTocEntry = (entry: TocEntry): TocEntry => ({
    depth: entry.depth,
    text: entry.text,
    slug: entry.slug,
    children: entry.children?.map(toRustTocEntry) ?? [],
  });
  const tocForRust = pageData.toc.map(toRustTocEntry);

  // Convert NavGroup to the format expected by Rust
  const toRustNavItem = (item: SsgNavItem): SsgNavItem => ({
    title: item.title,
    path: item.path,
    href: item.href,
    children: item.children?.map(toRustNavItem),
    collapsed: item.collapsed,
  });

  const navGroupsForRust = navGroups.map((group) => ({
    title: group.title,
    collapsed: group.collapsed,
    items: group.items.map(toRustNavItem),
  }));

  // Convert theme to NAPI format if provided
  const themeForRust = theme ? themeToNapi(theme) : undefined;

  // Convert entry page to NAPI format if provided
  const entryPageForRust = pageData.entryPage
    ? {
        hero: pageData.entryPage.hero
          ? {
              name: pageData.entryPage.hero.name,
              text: pageData.entryPage.hero.text,
              tagline: pageData.entryPage.hero.tagline,
              notice: pageData.entryPage.hero.notice
                ? {
                    title: pageData.entryPage.hero.notice.title,
                    body: pageData.entryPage.hero.notice.body,
                  }
                : undefined,
              image: pageData.entryPage.hero.image
                ? {
                    src: pageData.entryPage.hero.image.src,
                    lightSrc: pageData.entryPage.hero.image.lightSrc,
                    darkSrc: pageData.entryPage.hero.image.darkSrc,
                    alt: pageData.entryPage.hero.image.alt,
                    width: pageData.entryPage.hero.image.width,
                    height: pageData.entryPage.hero.image.height,
                  }
                : undefined,
              actions: pageData.entryPage.hero.actions?.map((a) => ({
                theme: a.theme,
                text: a.text,
                link: a.link,
              })),
            }
          : undefined,
        features: pageData.entryPage.features?.map((f) => ({
          icon: f.icon,
          title: f.title,
          details: f.details,
          link: f.link,
          linkText: f.linkText,
        })),
      }
    : undefined;

  return mod.generateSsgHtml(
    {
      title: pageData.title,
      description: pageData.description,
      content: pageData.content,
      toc: tocForRust,
      lastUpdated: pageData.lastUpdated,
      path: pageData.path,
      entryPage: entryPageForRust,
    },
    navGroupsForRust,
    {
      siteName,
      base,
      ogImage,
      theme: themeForRust,
      locale,
      availableLocales: availableLocales?.map((l) => ({
        code: l.code,
        name: l.name,
        dir: l.dir ?? "ltr",
      })),
    },
  );
}

interface GeneratedHtmlPage {
  inputPath: string;
  outputPath: string;
  html: string;
}

interface ExternalizedSharedAsset {
  outputPath: string;
  content: string;
}

async function externalizeSharedPageAssets(
  pages: GeneratedHtmlPage[],
  outDir: string,
  base: string,
): Promise<{ pages: GeneratedHtmlPage[]; assets: string[] }> {
  const mod = await importNapiModule();
  const optimized = mod.externalizeSsgAssets(pages, outDir, base) as {
    pages: GeneratedHtmlPage[];
    assets: ExternalizedSharedAsset[];
  };

  await Promise.all(
    optimized.assets.map(async (asset) => {
      await fs.mkdir(path.dirname(asset.outputPath), { recursive: true });
      await fs.writeFile(asset.outputPath, asset.content, "utf-8");
    }),
  );

  return {
    pages: optimized.pages,
    assets: optimized.assets.map((asset) => asset.outputPath),
  };
}

/**
 * Converts a markdown file path to its corresponding HTML output path.
 */
export function getOutputPath(
  inputPath: string,
  srcDir: string,
  outDir: string,
  extension: string,
): string {
  return importNapiModuleSync().getSsgOutputPath(inputPath, srcDir, outDir, extension);
}

/**
 * Converts a markdown file path to a relative URL path.
 */
export function getUrlPath(inputPath: string, srcDir: string): string {
  return importNapiModuleSync().getSsgUrlPath(inputPath, srcDir);
}

/**
 * Converts a markdown file path to an href.
 */
export function getHref(
  inputPath: string,
  srcDir: string,
  base: string,
  extension: string,
): string {
  return importNapiModuleSync().getSsgHref(inputPath, srcDir, base, extension);
}

/**
 * Resolves manual navigation config to the format used by the built-in SSG renderer.
 */
export function resolveNavigationGroups(
  navigation: SsgNavigationGroup[] | undefined,
  base: string,
  extension: string,
): NavGroup[] | undefined {
  if (!navigation) {
    return undefined;
  }

  return importNapiModuleSync().resolveSsgNavigationGroups(navigation, base, extension);
}

export function getPageLocale(urlPath: string, i18n: ResolvedOptions["i18n"]): string | undefined {
  if (!i18n) return undefined;
  return (
    importNapiModuleSync().getSsgPageLocale(
      urlPath,
      i18n.defaultLocale,
      i18n.locales.map((locale) => locale.code),
    ) ?? undefined
  );
}

function getRoutePaths(
  inputPath: string,
  srcDir: string,
  outDir: string,
  base: string,
  extension: string,
  siteUrl?: string,
): SsgRoutePaths {
  return importNapiModuleSync().resolveSsgRoutePaths(
    inputPath,
    srcDir,
    outDir,
    base,
    extension,
    siteUrl,
  );
}

/**
 * Formats a file/dir name as a title.
 */
export function formatTitle(name: string): string {
  return importNapiModuleSync().formatSsgTitle(name);
}

/**
 * Collects all markdown files from the source directory.
 */
export async function collectMarkdownFiles(
  srcDir: string,
  extensions: readonly string[] = DEFAULT_MARKDOWN_EXTENSIONS,
): Promise<string[]> {
  return importNapiModuleSync().collectSsgMarkdownFiles(srcDir, [...extensions]);
}

/**
 * Navigation group for hierarchical navigation.
 */
export interface NavGroup {
  title: string;
  items: SsgNavItem[];
  collapsed?: boolean;
}

/**
 * Builds navigation items from markdown files, grouped by directory.
 */
export function buildNavItems(
  markdownFiles: string[],
  srcDir: string,
  base: string,
  extension: string,
): NavGroup[] {
  return importNapiModuleSync().buildSsgNavItems(markdownFiles, srcDir, base, extension);
}

/**
 * Builds navigation items from an explicit theme sidebar tree.
 */
export function buildThemeNavItems(
  sidebar: SidebarItem[],
  base: string,
  extension: string,
): NavGroup[] {
  return importNapiModuleSync().buildSsgThemeNavItems(sidebar, base, extension);
}

/**
 * Builds all markdown files to static HTML.
 */
export async function buildSsg(
  options: ResolvedOptions,
  root: string,
): Promise<{ files: string[]; errors: string[] }> {
  const ssgOptions = options.ssg;
  if (!ssgOptions.enabled) {
    return { files: [], errors: [] };
  }

  const srcDir = path.resolve(root, options.srcDir);
  const outDir = path.resolve(root, options.outDir);
  const base = options.base.endsWith("/") ? options.base : options.base + "/";
  const generatedFiles: string[] = [];
  const generatedPages: GeneratedHtmlPage[] = [];
  const errors: string[] = [];

  // Clean output directory if requested
  if (ssgOptions.clean) {
    try {
      await fs.rm(outDir, { recursive: true, force: true });
    } catch {
      // Ignore if directory doesn't exist
    }
  }

  // Collect markdown files
  const markdownFiles = await collectMarkdownFiles(srcDir, options.extensions);

  // Build navigation
  const navItems =
    resolveNavigationGroups(ssgOptions.navigation, base, ssgOptions.extension) ??
    (ssgOptions.theme?.sidebar.length
      ? buildThemeNavItems(ssgOptions.theme.sidebar, base, ssgOptions.extension)
      : buildNavItems(markdownFiles, srcDir, base, ssgOptions.extension));

  // Get site name from options or package.json
  let siteName = ssgOptions.siteName ?? "Documentation";
  if (!ssgOptions.siteName) {
    try {
      const pkgPath = path.join(root, "package.json");
      const pkg = JSON.parse(await fs.readFile(pkgPath, "utf-8"));
      if (pkg.name) {
        siteName = formatTitle(pkg.name);
      }
    } catch {
      // Use default
    }
  }

  // Collect OG image entries for batch rendering
  const ogImageEntries: OgImagePageEntry[] = [];
  // Parallel array tracking the inputPath for each ogImageEntry (same index)
  const ogImageInputPaths: string[] = [];
  // Map from inputPath to OG image URL (filled after batch render)
  const ogImageUrlMap = new Map<string, string>();

  // Determine if OG images should be generated
  const shouldGenerateOgImages =
    (options.ogImage || ssgOptions.generateOgImage) && !ssgOptions.bare;

  // Collect page metadata for OG image generation
  interface PageProcessResult {
    inputPath: string;
    routePaths: SsgRoutePaths;
    transformedHtml: string;
    title: string;
    description?: string;
    lastUpdated?: number;
    frontmatter: Record<string, unknown>;
    toc: TocEntry[];
  }
  const pageResults: PageProcessResult[] = [];
  const napi = ssgOptions.lastUpdated ? await importNapiModule() : undefined;

  // Process each file: transform markdown and collect metadata
  for (const inputPath of markdownFiles) {
    try {
      const content = await fs.readFile(inputPath, "utf-8");
      // Pass SSG options to transform for .md -> .html link conversion in Rust
      // The sourcePath is used to determine if the file is an index file for correct relative link resolution
      const result = await transformMarkdown(content, inputPath, options, {
        convertMdLinks: true,
        baseUrl: base,
        sourcePath: inputPath,
      });
      const frontmatter = normalizeVitePressFrontmatter(result.frontmatter);

      // Apply built-in plugin transformations (No-JS First)
      let transformedHtml = result.html;

      // Protect mermaid SVGs from rehype processing in plugins
      const { html: protectedHtml, svgs: mermaidSvgs } = protectMermaidSvgs(transformedHtml);
      transformedHtml = protectedHtml;

      // Transform Tabs, YouTube, GitHub, OGP, Mermaid plugins
      const pluginOptions: TransformAllOptions = {
        tabs: true,
        youtube: true,
        github: options.embeds.github,
        openGraph: options.embeds.openGraph,
        mermaid: true,
        githubToken: process.env.GITHUB_TOKEN,
      };
      transformedHtml = await transformAllPlugins(transformedHtml, pluginOptions);

      // Transform Island components
      if (hasIslands(transformedHtml)) {
        const islandResult = await transformIslands(transformedHtml);
        transformedHtml = islandResult.html;
      }

      // Restore protected mermaid SVGs
      transformedHtml = restoreMermaidSvgs(transformedHtml, mermaidSvgs);

      const title = extractTitle(transformedHtml, frontmatter);
      const description = frontmatter.description as string | undefined;
      const routePaths = getRoutePaths(
        inputPath,
        srcDir,
        outDir,
        base,
        ssgOptions.extension,
        ssgOptions.siteUrl,
      );

      pageResults.push({
        inputPath,
        routePaths,
        transformedHtml,
        title,
        description,
        lastUpdated: napi?.getGitLastUpdated(inputPath, root) ?? undefined,
        frontmatter,
        toc: result.toc,
      });

      // Collect OG image entry if generation is enabled
      if (shouldGenerateOgImages) {
        const { layout: _layout, ...frontmatterRest } = frontmatter;
        ogImageEntries.push({
          props: {
            ...frontmatterRest,
            title,
            description,
            siteName,
          },
          outputPath: routePaths.ogImagePath,
        });
        ogImageInputPaths.push(inputPath);
        // Pre-compute URL so HTML can reference it
        ogImageUrlMap.set(inputPath, routePaths.ogImageUrl);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      errors.push(`Failed to process ${inputPath}: ${errorMessage}`);
    }
  }

  // Batch generate OG images (Chromium-based)
  if (shouldGenerateOgImages && ogImageEntries.length > 0) {
    try {
      const ogResults = await generateOgImages(ogImageEntries, options.ogImageOptions, root);

      // When the whole batch failed because Chromium wasn't available
      // (common in CI without browser deps), avoid spamming the log with one
      // error per page — `openBrowser()` already warned once. Just clear the
      // og:image meta tags and move on.
      const allMissingBrowser =
        ogResults.length > 0
          && ogResults.every((result) => result.error === "Chromium not available");
      if (allMissingBrowser) {
        for (const inputPath of ogImageInputPaths) {
          ogImageUrlMap.delete(inputPath);
        }
      } else {
        let ogSuccessCount = 0;
        for (let i = 0; i < ogResults.length; i++) {
          const result = ogResults[i];
          if (result.error) {
            errors.push(`OG image failed for ${result.outputPath}: ${result.error}`);
            // Remove failed entries so og:image / twitter:image meta tags are not emitted
            ogImageUrlMap.delete(ogImageInputPaths[i]);
          } else {
            generatedFiles.push(result.outputPath);
            ogSuccessCount++;
          }
        }
        if (ogSuccessCount > 0) {
          const cachedCount = ogResults.filter((r) => r.cached && !r.error).length;
          console.log(
            `[ox-content:og-image] Generated ${ogSuccessCount} OG images` +
              (cachedCount > 0 ? ` (${cachedCount} from cache)` : ""),
          );
        }
      }
    } catch (err) {
      // Non-fatal: OG image failures never block the SSG build
      const errorMessage = err instanceof Error ? err.message : String(err);
      console.warn(`[ox-content:og-image] Batch generation failed: ${errorMessage}`);
      // Clear all entries so og:image / twitter:image meta tags are not emitted
      ogImageUrlMap.clear();
    }
  }

  // Generate HTML pages
  for (const pageResult of pageResults) {
    try {
      const {
        inputPath,
        routePaths,
        transformedHtml,
        title,
        description,
        lastUpdated,
        frontmatter,
        toc,
      } = pageResult;

      // Determine OG image URL for this page
      let pageOgImage = ssgOptions.ogImage; // fallback to static URL
      if (shouldGenerateOgImages && ogImageUrlMap.has(inputPath)) {
        pageOgImage = ogImageUrlMap.get(inputPath);
      }

      // Check if this is an entry page (layout: entry)
      let entryPage: SsgEntryPageConfig | undefined;
      if (frontmatter.layout === "entry") {
        entryPage = {
          hero: frontmatter.hero as HeroConfig | undefined,
          features: frontmatter.features as FeatureConfig[] | undefined,
        };
      }

      // Generate HTML based on bare option
      let html: string;
      if (ssgOptions.bare) {
        html = generateBareHtmlPage(transformedHtml, title);
      } else {
        const pageData: SsgPageData = {
          title,
          description,
          content: transformedHtml,
          toc,
          lastUpdated,
          frontmatter,
          path: routePaths.urlPath,
          href: routePaths.href,
          entryPage,
        };
        html = await generateHtmlPage(
          pageData,
          navItems,
          siteName,
          base,
          pageOgImage,
          ssgOptions.theme,
          getPageLocale(pageData.path, options.i18n),
          options.i18n ? options.i18n.locales : undefined,
        );
      }

      generatedPages.push({
        inputPath,
        outputPath: routePaths.outputPath,
        html,
      });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      errors.push(`Failed to generate HTML for ${pageResult.inputPath}: ${errorMessage}`);
    }
  }

  const optimizedOutput = await externalizeSharedPageAssets(generatedPages, outDir, base);
  generatedFiles.push(...optimizedOutput.assets);

  for (const page of optimizedOutput.pages) {
    await fs.mkdir(path.dirname(page.outputPath), { recursive: true });
    await fs.writeFile(page.outputPath, page.html, "utf-8");
    generatedFiles.push(page.outputPath);
  }

  return { files: generatedFiles, errors };
}
