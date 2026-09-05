import type { OxContentAssetManifest } from "./assets";
import type {
  DocumentAssetManifest,
  RenderDocumentAssetsInput,
  RenderDocumentAssetsResult,
} from "./document-assets";
import type { RenderThemeTokenCssOptions, ThemeTokenSource } from "./theme-tokens";
import type { OxContentOptions, ResolvedOptions } from "./types";

export type MaybePromise<T> = T | Promise<T>;

export interface OxContentCustomHostOptions {
  /**
   * Host module loaded through Vite SSR. The module may default-export the host
   * object, export `host`, or be the host object itself.
   */
  host: string | OxContentCustomHostModule;
  /** Ox Content options shared with output writers and asset helpers. */
  oxContent?: OxContentOptions;
  /** Build lifecycle controls. */
  build?: OxContentCustomHostBuildOptions;
  /** Development middleware and cache controls. */
  dev?: OxContentCustomHostDevOptions;
  /** Optional framework-owned theme token stylesheet. */
  themeTokens?: false | OxContentCustomHostThemeTokensOptions;
}

export interface OxContentCustomHostBuildOptions {
  /** Disable only the custom-host build writer. */
  enabled?: boolean;
  /** Run during `vite build --mode test`. Disabled by default. */
  runInTest?: boolean;
  /** Apply Vite `transformIndexHtml` to HTML outputs. */
  transformHtml?: boolean;
}

export interface OxContentCustomHostDevOptions {
  /** Disable only the custom-host development middleware. */
  enabled?: boolean;
  /** Apply Vite `transformIndexHtml` to HTML responses. */
  transformHtml?: boolean;
  /** Debounce for full reloads after dependency invalidation. */
  reloadDebounceMs?: number;
}

export interface OxContentCustomHostThemeTokensOptions extends RenderThemeTokenCssOptions {
  theme: ThemeTokenSource | ThemeTokenSource[];
  /**
   * Public stylesheet path. Build writes this file and dev serves the same href.
   *
   * @default "__ox_theme_tokens__/theme-tokens.css"
   */
  href?: string;
}

export interface OxContentCustomHostModule {
  routes:
    | readonly OxContentCustomHostRoute[]
    | ((
        context: OxContentCustomHostRoutesContext,
      ) => MaybePromise<readonly OxContentCustomHostRoute[]>);
  notFound?: (
    context: OxContentCustomHostNotFoundContext,
  ) => MaybePromise<OxContentCustomHostRenderResult | Response | undefined | void>;
}

export interface OxContentCustomHostRoute {
  path: string;
  render: (
    context: OxContentCustomHostRenderContext,
  ) => MaybePromise<OxContentCustomHostRenderResult | Response | undefined | void>;
  /** Source file used for resource resolution, Markdown companions, and git lastmod. */
  inputPath?: string;
  /** Source bytes used for Markdown companion output. */
  source?: string;
  title?: string;
  description?: string;
  aliases?: readonly string[];
  redirect?: string;
  draft?: boolean;
  unlisted?: boolean;
  frontmatter?: Record<string, unknown>;
  dependencies?: readonly string[];
}

export interface OxContentCustomHostRenderResult {
  body?: string | Uint8Array;
  html?: string;
  text?: string;
  status?: number;
  statusText?: string;
  contentType?: string;
  headers?: HeadersInit;
  outputPath?: string;
  inputPath?: string;
  source?: string;
  title?: string;
  description?: string;
  loc?: string;
  lastUpdated?: number;
  aliases?: readonly string[];
  redirect?: string;
  draft?: boolean;
  unlisted?: boolean;
  frontmatter?: Record<string, unknown>;
  dependencies?: readonly string[];
}

export interface OxContentCustomHostAssetsContext {
  selfHosted: OxContentAssetManifest;
  clientManifest?: DocumentAssetManifest;
  themeTokens?: ResolvedThemeTokens;
  document(input?: RenderDocumentAssetsInput): RenderDocumentAssetsResult;
}

export interface OxContentCustomHostBaseContext {
  mode: "build" | "serve";
  root: string;
  outDir: string;
  base: string;
  options: ResolvedOptions;
  loadModule(moduleId: string): Promise<unknown>;
  assets: OxContentCustomHostAssetsContext;
}

export interface OxContentCustomHostRoutesContext extends OxContentCustomHostBaseContext {}

export interface OxContentCustomHostRenderContext extends OxContentCustomHostBaseContext {
  route: OxContentCustomHostRoute;
  request: Request;
  url: URL;
}

export interface OxContentCustomHostNotFoundContext extends OxContentCustomHostBaseContext {
  request: Request;
  url: URL;
}

export interface ResolvedThemeTokens {
  href: string;
  outputPath: string;
  css: string;
}

export interface SerializedResponse {
  status: number;
  statusText?: string;
  headers: [string, string][];
  body: Uint8Array;
  dependencies: string[];
}

export interface DevCacheEntry {
  promise: Promise<SerializedResponse | undefined>;
}

export interface RenderedBuildRoute {
  route: OxContentCustomHostRoute;
  routePath: string;
  outputPath: string;
  contentType: string;
  body: string | Uint8Array;
  result: OxContentCustomHostRenderResult;
}
