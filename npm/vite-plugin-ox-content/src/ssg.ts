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

/** NAPI-facing nav group shape produced from a [`NavGroup`]. */
interface RustNavGroup {
  title: string;
  collapsed?: boolean;
  items: SsgNavItem[];
}

/**
 * Per-build cache for the Rust-facing nav conversion. `navGroups` is the same
 * `context.navItems` reference for every page in a build, so the deep recursive
 * copy below only needs to run once per build instead of once per page.
 */
const navGroupsForRustCache = new WeakMap<NavGroup[], RustNavGroup[]>();

function toRustNavItem(item: SsgNavItem): SsgNavItem {
  return {
    title: item.title,
    path: item.path,
    href: item.href,
    children: item.children?.map(toRustNavItem),
    collapsed: item.collapsed,
  };
}

function convertNavGroupsForRust(navGroups: NavGroup[]): RustNavGroup[] {
  const cached = navGroupsForRustCache.get(navGroups);
  if (cached) {
    return cached;
  }
  const converted = navGroups.map((group) => ({
    title: group.title,
    collapsed: group.collapsed,
    items: group.items.map(toRustNavItem),
  }));
  navGroupsForRustCache.set(navGroups, converted);
  return converted;
}

/**
 * Converts a `TocEntry` tree into the plain shape the Rust binding expects.
 * Hoisted to module scope so it isn't reallocated for every page; the
 * per-page `.map` over `pageData.toc` still runs since the TOC is page-specific.
 */
function toRustTocEntry(entry: TocEntry): TocEntry {
  return {
    depth: entry.depth,
    text: entry.text,
    slug: entry.slug,
    children: entry.children?.map(toRustTocEntry) ?? [],
  };
}

/** Rust-facing locale shape. */
interface RustLocale {
  code: string;
  name: string;
  dir: string;
}

/**
 * Per-build cache for the Rust-facing locale list. `i18n.locales` is the same
 * reference for every page in a build, so this mapping (and the `?? "ltr"`
 * default) only runs once per build instead of once per page.
 */
const rustLocalesCache = new WeakMap<LocaleConfig[], RustLocale[]>();

function toRustLocales(locales: LocaleConfig[]): RustLocale[] {
  const cached = rustLocalesCache.get(locales);
  if (cached) {
    return cached;
  }
  const converted = locales.map((locale) => ({
    code: locale.code,
    name: locale.name,
    dir: locale.dir ?? "ltr",
  }));
  rustLocalesCache.set(locales, converted);
  return converted;
}

/**
 * Per-build cache for the locale-code list passed to `getSsgPageLocale`. The
 * `i18n.locales` reference is stable across a build, so the `.map` to codes
 * runs once instead of once per page.
 */
const localeCodesCache = new WeakMap<LocaleConfig[], string[]>();

function localeCodesFor(locales: LocaleConfig[]): string[] {
  const cached = localeCodesCache.get(locales);
  if (cached) {
    return cached;
  }
  const codes = locales.map((locale) => locale.code);
  localeCodesCache.set(locales, codes);
  return codes;
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

  // Convert TocEntry to the format expected by Rust (converter is module-scoped).
  const tocForRust = pageData.toc.map(toRustTocEntry);

  // Convert NavGroup to the format expected by Rust (cached per build).
  const navGroupsForRust = convertNavGroupsForRust(navGroups);

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
      availableLocales: availableLocales ? toRustLocales(availableLocales) : undefined,
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
  // Asset extraction is batched after all pages are rendered so the Rust side
  // can de-duplicate identical CSS/JS chunks across the whole build. Doing it
  // page-by-page would miss shared chunks and write duplicate assets.
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
      localeCodesFor(i18n.locales),
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

interface BuildSsgContext {
  options: ResolvedOptions;
  ssgOptions: ResolvedSsgOptions;
  root: string;
  srcDir: string;
  outDir: string;
  base: string;
  siteName: string;
  navItems: NavGroup[];
  shouldGenerateOgImages: boolean;
  napi?: Awaited<ReturnType<typeof importNapiModule>>;
}

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

interface CollectedPageResults {
  pageResults: PageProcessResult[];
  ogImageEntries: OgImagePageEntry[];
  ogImageInputPaths: string[];
  ogImageUrlMap: Map<string, string>;
  errors: string[];
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
  const generatedFiles: string[] = [];
  const errors: string[] = [];

  await cleanOutputDirectory(ssgOptions, outDir);

  const markdownFiles = await collectMarkdownFiles(srcDir, options.extensions);
  const context = await createBuildSsgContext(options, root, srcDir, outDir, markdownFiles);
  const collected = await collectPageResults(context, markdownFiles);
  errors.push(...collected.errors);

  await generateOgImageAssets(context, collected, generatedFiles, errors);

  const generatedPages = await generateHtmlPages(context, collected.pageResults, collected, errors);
  await writeGeneratedPages(generatedPages, context, generatedFiles);

  return { files: generatedFiles, errors };
}

async function cleanOutputDirectory(ssgOptions: ResolvedSsgOptions, outDir: string): Promise<void> {
  if (!ssgOptions.clean) {
    return;
  }

  try {
    await fs.rm(outDir, { recursive: true, force: true });
  } catch {
    // Ignore if directory doesn't exist.
  }
}

async function createBuildSsgContext(
  options: ResolvedOptions,
  root: string,
  srcDir: string,
  outDir: string,
  markdownFiles: string[],
): Promise<BuildSsgContext> {
  const ssgOptions = options.ssg;
  const base = options.base.endsWith("/") ? options.base : options.base + "/";
  const navItems =
    resolveNavigationGroups(ssgOptions.navigation, base, ssgOptions.extension) ??
    (ssgOptions.theme?.sidebar.length
      ? buildThemeNavItems(ssgOptions.theme.sidebar, base, ssgOptions.extension)
      : buildNavItems(markdownFiles, srcDir, base, ssgOptions.extension));

  return {
    options,
    ssgOptions,
    root,
    srcDir,
    outDir,
    base,
    navItems,
    siteName: await resolveSiteName(root, ssgOptions),
    shouldGenerateOgImages: (options.ogImage || ssgOptions.generateOgImage) && !ssgOptions.bare,
    napi: ssgOptions.lastUpdated ? await importNapiModule() : undefined,
  };
}

async function resolveSiteName(root: string, ssgOptions: ResolvedSsgOptions): Promise<string> {
  if (ssgOptions.siteName) {
    return ssgOptions.siteName;
  }

  try {
    const pkgPath = path.join(root, "package.json");
    const pkg = JSON.parse(await fs.readFile(pkgPath, "utf-8"));
    return pkg.name ? formatTitle(pkg.name) : "Documentation";
  } catch {
    return "Documentation";
  }
}

async function collectPageResults(
  context: BuildSsgContext,
  markdownFiles: string[],
): Promise<CollectedPageResults> {
  const collected: CollectedPageResults = {
    pageResults: [],
    ogImageEntries: [],
    ogImageInputPaths: [],
    ogImageUrlMap: new Map(),
    errors: [],
  };

  for (const inputPath of markdownFiles) {
    try {
      const pageResult = await transformSsgPage(context, inputPath);
      collected.pageResults.push(pageResult);
      collectOgImageEntry(context, pageResult, collected);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      collected.errors.push(`Failed to process ${inputPath}: ${errorMessage}`);
    }
  }

  return collected;
}

async function transformSsgPage(
  context: BuildSsgContext,
  inputPath: string,
): Promise<PageProcessResult> {
  const content = await fs.readFile(inputPath, "utf-8");
  const result = await transformMarkdown(content, inputPath, context.options, {
    convertMdLinks: true,
    baseUrl: context.base,
    sourcePath: inputPath,
  });
  const frontmatter = normalizeVitePressFrontmatter(result.frontmatter);
  const transformedHtml = await transformSsgHtml(result.html, context.options);
  const title = extractTitle(transformedHtml, frontmatter);

  return {
    inputPath,
    routePaths: getRoutePaths(
      inputPath,
      context.srcDir,
      context.outDir,
      context.base,
      context.ssgOptions.extension,
      context.ssgOptions.siteUrl,
    ),
    transformedHtml,
    title,
    description: frontmatter.description as string | undefined,
    lastUpdated: context.napi?.getGitLastUpdated(inputPath, context.root) ?? undefined,
    frontmatter,
    toc: result.toc,
  };
}

async function transformSsgHtml(html: string, options: ResolvedOptions): Promise<string> {
  // Mermaid SVGs are protected before plugin transforms because some transforms
  // still use HTML parser/stringifier steps that can corrupt SVG foreignObject
  // markup. The protect/restore pair keeps the rest of the pipeline free to
  // operate on normal HTML strings.
  const { html: protectedHtml, svgs: mermaidSvgs } = protectMermaidSvgs(html);
  const pluginOptions: TransformAllOptions = {
    tabs: true,
    youtube: true,
    github: options.embeds.github,
    openGraph: options.embeds.openGraph,
    pm: options.embeds.pm,
    spotify: options.embeds.spotify,
    stackBlitz: options.embeds.stackBlitz,
    twitter: options.embeds.twitter,
    bluesky: options.embeds.bluesky,
    webContainer: options.embeds.webContainer,
    mermaid: true,
    githubToken: process.env.GITHUB_TOKEN,
  };

  let transformedHtml = await transformAllPlugins(protectedHtml, pluginOptions);
  if (hasIslands(transformedHtml)) {
    const islandResult = await transformIslands(transformedHtml);
    transformedHtml = islandResult.html;
  }

  return restoreMermaidSvgs(transformedHtml, mermaidSvgs);
}

function collectOgImageEntry(
  context: BuildSsgContext,
  pageResult: PageProcessResult,
  collected: CollectedPageResults,
): void {
  if (!context.shouldGenerateOgImages) {
    return;
  }

  const { layout: _layout, ...frontmatterRest } = pageResult.frontmatter;
  collected.ogImageEntries.push({
    props: {
      ...frontmatterRest,
      title: pageResult.title,
      description: pageResult.description,
      siteName: context.siteName,
    },
    outputPath: pageResult.routePaths.ogImagePath,
  });
  collected.ogImageInputPaths.push(pageResult.inputPath);
  collected.ogImageUrlMap.set(pageResult.inputPath, pageResult.routePaths.ogImageUrl);
}

async function generateOgImageAssets(
  context: BuildSsgContext,
  collected: CollectedPageResults,
  generatedFiles: string[],
  errors: string[],
): Promise<void> {
  if (!context.shouldGenerateOgImages || collected.ogImageEntries.length === 0) {
    return;
  }

  try {
    const ogResults = await generateOgImages(
      collected.ogImageEntries,
      context.options.ogImageOptions,
      context.root,
    );
    if (clearMissingBrowserOgImages(ogResults, collected)) {
      return;
    }

    reportOgImageResults(ogResults, collected, generatedFiles, errors);
  } catch (err) {
    const errorMessage = err instanceof Error ? err.message : String(err);
    console.warn(`[ox-content:og-image] Batch generation failed: ${errorMessage}`);
    collected.ogImageUrlMap.clear();
  }
}

function clearMissingBrowserOgImages(
  ogResults: Awaited<ReturnType<typeof generateOgImages>>,
  collected: CollectedPageResults,
): boolean {
  const allMissingBrowser =
    ogResults.length > 0 && ogResults.every((result) => result.error === "Chromium not available");
  if (!allMissingBrowser) {
    return false;
  }

  for (const inputPath of collected.ogImageInputPaths) {
    collected.ogImageUrlMap.delete(inputPath);
  }
  return true;
}

function reportOgImageResults(
  ogResults: Awaited<ReturnType<typeof generateOgImages>>,
  collected: CollectedPageResults,
  generatedFiles: string[],
  errors: string[],
): void {
  let ogSuccessCount = 0;

  for (let i = 0; i < ogResults.length; i++) {
    const result = ogResults[i];
    if (result.error) {
      errors.push(`OG image failed for ${result.outputPath}: ${result.error}`);
      collected.ogImageUrlMap.delete(collected.ogImageInputPaths[i]);
    } else {
      generatedFiles.push(result.outputPath);
      ogSuccessCount++;
    }
  }

  if (ogSuccessCount > 0) {
    const cachedCount = ogResults.filter((result) => result.cached && !result.error).length;
    console.log(
      `[ox-content:og-image] Generated ${ogSuccessCount} OG images` +
        (cachedCount > 0 ? ` (${cachedCount} from cache)` : ""),
    );
  }
}

async function generateHtmlPages(
  context: BuildSsgContext,
  pageResults: PageProcessResult[],
  collected: CollectedPageResults,
  errors: string[],
): Promise<GeneratedHtmlPage[]> {
  const generatedPages: GeneratedHtmlPage[] = [];

  for (const pageResult of pageResults) {
    try {
      generatedPages.push({
        inputPath: pageResult.inputPath,
        outputPath: pageResult.routePaths.outputPath,
        html: await renderSsgPage(context, pageResult, collected.ogImageUrlMap),
      });
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      errors.push(`Failed to generate HTML for ${pageResult.inputPath}: ${errorMessage}`);
    }
  }

  return generatedPages;
}

async function renderSsgPage(
  context: BuildSsgContext,
  pageResult: PageProcessResult,
  ogImageUrlMap: Map<string, string>,
): Promise<string> {
  if (context.ssgOptions.bare) {
    return generateBareHtmlPage(pageResult.transformedHtml, pageResult.title);
  }

  const pageData = createSsgPageData(pageResult);
  const pageOgImage =
    context.shouldGenerateOgImages && ogImageUrlMap.has(pageResult.inputPath)
      ? ogImageUrlMap.get(pageResult.inputPath)
      : context.ssgOptions.ogImage;

  return generateHtmlPage(
    pageData,
    context.navItems,
    context.siteName,
    context.base,
    pageOgImage,
    context.ssgOptions.theme,
    getPageLocale(pageData.path, context.options.i18n),
    context.options.i18n ? context.options.i18n.locales : undefined,
  );
}

function createSsgPageData(pageResult: PageProcessResult): SsgPageData {
  const { frontmatter } = pageResult;
  const entryPage =
    frontmatter.layout === "entry"
      ? {
          hero: frontmatter.hero as HeroConfig | undefined,
          features: frontmatter.features as FeatureConfig[] | undefined,
        }
      : undefined;

  return {
    title: pageResult.title,
    description: pageResult.description,
    content: pageResult.transformedHtml,
    toc: pageResult.toc,
    lastUpdated: pageResult.lastUpdated,
    frontmatter,
    path: pageResult.routePaths.urlPath,
    href: pageResult.routePaths.href,
    entryPage,
  };
}

async function writeGeneratedPages(
  generatedPages: GeneratedHtmlPage[],
  context: BuildSsgContext,
  generatedFiles: string[],
): Promise<void> {
  // Shared asset extraction needs the complete page set to maximize
  // de-duplication. Only after replacement do we write pages and record both
  // the generated assets and the rewritten HTML files.
  const optimizedOutput = await externalizeSharedPageAssets(
    generatedPages,
    context.outDir,
    context.base,
  );
  generatedFiles.push(...optimizedOutput.assets);

  for (const page of optimizedOutput.pages) {
    await fs.mkdir(path.dirname(page.outputPath), { recursive: true });
    await fs.writeFile(page.outputPath, page.html, "utf-8");
    generatedFiles.push(page.outputPath);
  }
}
