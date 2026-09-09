import type { Options as HtmlMinifierOptions } from "html-minifier-terser";

const HYDRATION_COMMENT = /^\s*[!/$#]/u;

const HTML_MINIFY_OPTIONS: HtmlMinifierOptions = {
  caseSensitive: true,
  collapseWhitespace: true,
  conservativeCollapse: true,
  ignoreCustomComments: [HYDRATION_COMMENT],
  minifyCSS: minifyCss,
  minifyJS: minifyJs,
  removeComments: true,
  removeRedundantAttributes: true,
  useShortDoctype: true,
};

export interface HtmlMinifyContext {
  scripts: Map<string, Promise<string>>;
  styles: Map<string, Promise<string>>;
}

let cleanCssModule: Promise<typeof import("clean-css")> | undefined;
let htmlMinifierModule: Promise<typeof import("html-minifier-terser")> | undefined;
let terserModule: Promise<typeof import("terser")> | undefined;

export function createHtmlMinifyContext(): HtmlMinifyContext {
  return {
    scripts: new Map(),
    styles: new Map(),
  };
}

export async function minifyHtmlOutput(
  html: string,
  context: HtmlMinifyContext = createHtmlMinifyContext(),
): Promise<string> {
  const { minify } = await loadHtmlMinifier();
  return minify(html, optionsForContext(context));
}

function optionsForContext(context: HtmlMinifyContext): HtmlMinifierOptions {
  return {
    ...HTML_MINIFY_OPTIONS,
    minifyCSS: (text, type) =>
      cachedMinify(context.styles, cacheKey(type ?? "style", text), () => minifyCss(text, type)),
    minifyJS: (text, inline) =>
      cachedMinify(context.scripts, cacheKey(inline === true ? "inline" : "script", text), () =>
        minifyJs(text, inline),
      ),
  };
}

function cachedMinify(
  cache: Map<string, Promise<string>>,
  key: string,
  minify: () => Promise<string>,
): Promise<string> {
  const cached = cache.get(key);
  if (cached) return cached;

  const next = minify().catch((error: unknown) => {
    if (cache.get(key) === next) {
      cache.delete(key);
    }
    throw error;
  });
  cache.set(key, next);
  return next;
}

function cacheKey(kind: string, text: string): string {
  return `${kind}\0${text}`;
}

async function minifyJs(text: string, inline?: boolean): Promise<string> {
  const { minify: minifyJavaScript } = await loadTerser();
  const result = await minifyJavaScript(text, {
    parse: { bare_returns: inline === true },
  });
  return (result.code ?? "").replace(/;$/u, "");
}

async function minifyCss(text: string, type?: string): Promise<string> {
  const { default: CleanCss } = await loadCleanCss();
  const output = new CleanCss().minify(wrapCss(text, type));
  if (output.errors.length > 0) {
    throw new Error(`[ox-content] HTML CSS minification failed: ${output.errors.join("; ")}`);
  }
  return unwrapCss(output.styles, type);
}

function wrapCss(text: string, type?: string): string {
  if (type === "inline") {
    return `*{${text}}`;
  }
  if (type === "media") {
    return `@media ${text}{a{top:0}}`;
  }
  return text;
}

function unwrapCss(text: string, type?: string): string {
  const pattern =
    type === "inline"
      ? /^\*\{(?<css>[\s\S]*)\}$/u
      : type === "media"
        ? /^@media (?<css>[\s\S]*?)\s*\{[\s\S]*\}$/u
        : undefined;
  return pattern ? (text.match(pattern)?.groups?.css ?? text) : text;
}

function loadCleanCss(): Promise<typeof import("clean-css")> {
  return (cleanCssModule ??= import("clean-css"));
}

function loadHtmlMinifier(): Promise<typeof import("html-minifier-terser")> {
  return (htmlMinifierModule ??= import("html-minifier-terser"));
}

function loadTerser(): Promise<typeof import("terser")> {
  return (terserModule ??= import("terser"));
}
