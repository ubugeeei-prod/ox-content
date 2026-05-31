import type {
  LanguageRegistration,
  ResolvedOgImageOptions,
  ResolvedOptions,
  ResolvedSsgOptions,
  ThemeRegistration,
} from "@ox-content/vite-plugin";
import type { ResolvedSlideEditorOptions } from "./editor-options";
import type { ResolvedSlidePdfOptions } from "./pdf-options";
import type { OxContentSlidesOptions, SlideSourceRenderer, SlideThemeConfig } from "./public-types";

/**
 * Normalized plugin options used across the slide build pipeline.
 */
export interface ResolvedSlidesPluginOptions extends Omit<
  OxContentSlidesOptions,
  "ssg" | "ogImageOptions" | "pdf" | "renderers" | "theme"
> {
  srcDir: string;
  outDir: string;
  base: string;
  baseHref: string;
  routeBase: string;
  routePrefix: string;
  animations: boolean;
  editor: ResolvedSlideEditorOptions;
  presenter: boolean;
  separator: string;
  extensions: string[];
  renderers: Record<string, SlideSourceRenderer>;
  ssg: ResolvedSsgOptions;
  ogImageOptions: ResolvedOgImageOptions;
  pdf: ResolvedSlidePdfOptions;
  theme: SlideThemeConfig;
  napiTheme: NapiSlideTheme;
  markdown: ResolvedOptions;
  gfm: boolean;
  footnotes: boolean;
  tables: boolean;
  taskLists: boolean;
  strikethrough: boolean;
  highlight: boolean;
  highlightTheme: string | ThemeRegistration;
  highlightLangs: LanguageRegistration[];
  mermaid: boolean;
}

/**
 * Speaker notes extracted from a source slide via the NAPI bridge.
 */
export interface NapiSlideComments {
  content: string;
  notes: string[];
}

/**
 * Parsed deck payload returned from the Rust Markdown parser.
 */
export interface NapiSlideDeck {
  frontmatter: string;
  slides: Array<{ content: string; notes: string[] }>;
}

/**
 * Theme values forwarded to the Rust HTML shell renderer.
 */
export interface NapiSlideTheme {
  aspectRatio?: string;
  maxWidth?: string;
  maxHeight?: string;
  padding?: string;
  surfaceRadius?: string;
  codeBackground?: string;
  builtinAnimations?: boolean;
  canvasBackground?: string;
  surfaceBackground?: string;
  surfaceBorder?: string;
  surfaceShadow?: string;
  presenterSidebarBackground?: string;
  fontSans?: string;
  fontMono?: string;
  colorText?: string;
  colorTextMuted?: string;
  colorPrimary?: string;
  colorBorder?: string;
  darkCanvasBackground?: string;
  darkSurfaceBackground?: string;
  darkSurfaceBorder?: string;
  darkPresenterSidebarBackground?: string;
  darkCodeBackground?: string;
  darkColorText?: string;
  darkColorTextMuted?: string;
  darkColorPrimary?: string;
  darkColorBorder?: string;
}

/**
 * Render payload for a single slide or presenter page.
 */
export interface NapiSlideRenderData {
  deckTitle: string;
  slideTitle: string;
  slideDescription?: string;
  slideContentHtml: string;
  slideNotesHtml?: string;
  slideNumber: number;
  slideCount: number;
  homeHref: string;
  slideHref: string;
  presenterHref?: string;
  previousHref?: string;
  nextHref?: string;
  nextSlideHref?: string;
}

/**
 * Render payload for a deck-wide print shell used during PDF export.
 */
export interface NapiDeckPrintRenderData {
  deckTitle: string;
  deckDescription?: string;
  pageWidth: string;
  pageHeight: string;
  slides: Array<{
    slideTitle: string;
    slideContentHtml: string;
    slideNumber: number;
    slideCount: number;
  }>;
}

/**
 * Runtime API provided by `@ox-content/napi`.
 */
export interface NapiModule {
  extractSlideComments(source: string): NapiSlideComments;
  parseMarkdownSlideDeck(source: string, separator?: string): NapiSlideDeck;
  generateDeckPrintHtml(data: NapiDeckPrintRenderData, theme?: NapiSlideTheme): string;
  generateSlideHtml(data: NapiSlideRenderData, theme?: NapiSlideTheme): string;
  generatePresenterHtml(data: NapiSlideRenderData, theme?: NapiSlideTheme): string;
}

/**
 * Discovered slide source file on disk.
 */
export interface SlideFileEntry {
  filePath: string;
}

/**
 * Fully rendered data for one generated slide page.
 */
export interface SlidePageData {
  title: string;
  description?: string;
  contentHtml: string;
  notesHtml?: string;
  notes: string[];
  frontmatter: Record<string, unknown>;
  sourcePath: string;
  slideNumber: number;
  slideCount: number;
  href: string;
  outputPath: string;
  presenterHref?: string;
  presenterOutputPath?: string;
}

/**
 * Deck-level output data including every rendered slide.
 */
export interface SlideDeckData {
  slug: string;
  title: string;
  description?: string;
  href: string;
  outputPath: string;
  printOutputPath?: string;
  pdfOutputPath?: string;
  presenterHref?: string;
  presenterOutputPath?: string;
  slides: SlidePageData[];
}

/**
 * Route lookup entry used by the dev server MPA middleware.
 */
export interface SlideRouteData {
  deck: SlideDeckData;
  slide: SlidePageData;
  presenter: boolean;
}

/**
 * Indexed route map built from the rendered deck collection.
 */
export interface SlideBuildArtifacts {
  routes: Map<string, SlideRouteData>;
}
