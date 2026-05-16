/**
 * Theme Renderer for Static HTML Generation
 *
 * Renders JSX theme components to static HTML strings.
 * No client-side JavaScript is included by default.
 */

import { renderToString, raw, type JSXNode } from "./jsx-runtime";
import {
  setRenderContext,
  clearRenderContext,
  generateFrontmatterTypes,
  type RenderContext,
  type PageProps,
  type SiteConfig,
  type NavGroup,
} from "./page-context";
import type { TocEntry } from "./types";
import { writeFile, mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";

/**
 * Theme component type.
 */
export type ThemeComponent = (props: ThemeProps) => JSXNode;

/**
 * Props passed to the theme component.
 */
export interface ThemeProps {
  /** Rendered page content as JSX */
  children: JSXNode;
}

/**
 * Page data for rendering.
 */
export interface PageData {
  /** Page title */
  title: string;
  /** Page description */
  description?: string;
  /** Rendered HTML content */
  html: string;
  /** Table of contents */
  toc: TocEntry[];
  /** Last git commit timestamp in milliseconds */
  lastUpdated?: number;
  /** Source file path */
  path: string;
  /** Output URL path */
  url: string;
  /** Frontmatter */
  frontmatter: Record<string, unknown>;
  /** Layout name */
  layout?: string;
}

/**
 * Theme render options.
 */
export interface ThemeRenderOptions {
  /** Theme component to use */
  theme: ThemeComponent;
  /** Site name */
  siteName: string;
  /** Base URL path */
  base: string;
  /** Navigation groups */
  nav: NavGroup[];
  /** All pages (for site context) */
  pages: PageData[];
  /** Output directory for type definitions */
  typesOutDir?: string;
}

/**
 * Renders a page using the theme component.
 *
 * @param page - Page data to render
 * @param options - Theme render options
 * @returns Rendered HTML string
 */
export function renderPage(page: PageData, options: ThemeRenderOptions): string {
  const { theme, siteName, base, nav, pages } = options;

  // Build page props
  const pageProps: PageProps = {
    title: page.title,
    description: page.description,
    html: page.html,
    toc: page.toc,
    lastUpdated: page.lastUpdated,
    path: page.path,
    url: page.url,
    frontmatter: page.frontmatter,
    layout: page.layout,
  };

  // Build site config
  const siteConfig: SiteConfig = {
    name: siteName,
    base,
    nav,
    pages: pages.map((p) => ({
      title: p.title,
      description: p.description,
      html: p.html,
      toc: p.toc,
      lastUpdated: p.lastUpdated,
      path: p.path,
      url: p.url,
      frontmatter: p.frontmatter,
      layout: p.layout,
    })),
  };

  // Set render context
  const context: RenderContext = {
    page: pageProps,
    site: siteConfig,
  };

  setRenderContext(context);

  try {
    // Render theme with page content
    const contentNode = raw(page.html);
    const result = theme({ children: contentNode });

    // Get HTML string
    const html = renderToString(result);

    // Add doctype if not present
    if (!html.trimStart().toLowerCase().startsWith("<!doctype")) {
      return `<!DOCTYPE html>\n${html}`;
    }

    return html;
  } finally {
    clearRenderContext();
  }
}

/**
 * Renders all pages and generates type definitions.
 *
 * @param pages - All pages to render
 * @param options - Theme render options
 * @returns Map of output paths to rendered HTML
 */
export async function renderAllPages(
  pages: PageData[],
  options: ThemeRenderOptions,
): Promise<Map<string, string>> {
  const results = new Map<string, string>();

  // Render each page
  for (const page of pages) {
    const html = renderPage(page, { ...options, pages });
    results.set(page.url, html);
  }

  // Generate type definitions if output directory is specified
  if (options.typesOutDir) {
    await generateTypes(pages, options.typesOutDir);
  }

  return results;
}

/**
 * Generates TypeScript type definitions from page frontmatter.
 *
 * @param pages - All pages
 * @param outDir - Output directory for types
 */
export async function generateTypes(pages: PageData[], outDir: string): Promise<void> {
  // Collect all frontmatter samples
  const samples = pages.map((p) => p.frontmatter);

  // Generate types
  const types = generateFrontmatterTypes(samples);

  // Write to file
  const typesPath = join(outDir, "page-props.d.ts");
  await mkdir(dirname(typesPath), { recursive: true });
  await writeFile(typesPath, types, "utf-8");
}

/**
 * Default theme component.
 * A minimal theme that renders page content with basic styling.
 */
export function DefaultTheme({ children }: ThemeProps): JSXNode {
  // Use hooks inside the component
  const { usePageProps, useSiteConfig } = require("./page-context");
  const page = usePageProps();
  const site = useSiteConfig();

  return {
    __html: `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${escapeHtml(page.title)} - ${escapeHtml(site.name)}</title>
  ${page.description ? `<meta name="description" content="${escapeHtml(page.description)}">` : ""}
  <style>
    :root {
      --octc-color-primary: #4f6fae;
      --octc-color-text: #131a30;
      --octc-color-bg: #ffffff;
      --octc-color-bg-alt: #f5f7fb;
      --octc-color-text-muted: #4f607b;
      --octc-color-border: #d2dbea;
    }
    body {
      font-family: "IBM Plex Sans", "Avenir Next", "Segoe UI Variable", "Segoe UI", sans-serif;
      line-height: 1.7;
      color: var(--octc-color-text);
      background: var(--octc-color-bg);
      max-width: 800px;
      margin: 0 auto;
      padding: 2rem;
    }
    a { color: var(--octc-color-primary); }
  </style>
</head>
<body>
  <header>
    <h1>${escapeHtml(site.name)}</h1>
  </header>
  <main>
    ${children.__html}
  </main>
</body>
</html>`,
  };
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

/**
 * Creates a theme with layout switching support.
 *
 * @example
 * ```tsx
 * import { createTheme } from '@ox-content/vite-plugin';
 * import { DefaultLayout } from './layouts/Default';
 * import { EntryLayout } from './layouts/Entry';
 *
 * export default createTheme({
 *   layouts: {
 *     default: DefaultLayout,
 *     entry: EntryLayout,
 *   },
 * });
 * ```
 */
export function createTheme(config: {
  layouts: Record<string, ThemeComponent>;
  defaultLayout?: string;
}): ThemeComponent {
  const { layouts, defaultLayout = "default" } = config;

  return function ThemeWithLayouts({ children }: ThemeProps): JSXNode {
    // eslint-disable-next-line @typescript-eslint/no-require-imports
    const { usePageProps } = require("./page-context");
    const page = usePageProps();

    // Get layout from frontmatter or use default
    const layoutName = page.layout ?? defaultLayout;
    const Layout = layouts[layoutName] ?? layouts[defaultLayout];

    if (!Layout) {
      throw new Error(
        `[ox-content] Layout "${layoutName}" not found. ` +
          `Available layouts: ${Object.keys(layouts).join(", ")}`,
      );
    }

    return Layout({ children });
  };
}
