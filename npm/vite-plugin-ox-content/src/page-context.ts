/**
 * Page Context for Static HTML Generation
 *
 * Provides a way to access page props (frontmatter, content, etc.)
 * from theme components during static rendering.
 *
 * @example
 * ```tsx
 * // theme/Layout.tsx
 * import { usePageProps, PageProps } from '@ox-content/vite-plugin';
 *
 * export function Layout({ children }: { children: JSX.Element }) {
 *   const page = usePageProps<MyPageProps>();
 *   return (
 *     <html>
 *       <head>
 *         <title>{page.title}</title>
 *       </head>
 *       <body>
 *         <header>{page.title}</header>
 *         <main>{children}</main>
 *       </body>
 *     </html>
 *   );
 * }
 * ```
 */

import type { TocEntry } from "./types";

/**
 * Base page props available for all pages.
 */
export interface BasePageProps {
  /** Page title from frontmatter or first heading */
  title: string;
  /** Page description from frontmatter */
  description?: string;
  /** Rendered HTML content */
  html: string;
  /** Table of contents entries */
  toc: TocEntry[];
  /** Last git commit timestamp in milliseconds */
  lastUpdated?: number;
  /** Source file path (relative to docs root) */
  path: string;
  /** Output URL path */
  url: string;
  /** Raw frontmatter object */
  frontmatter: Record<string, unknown>;
  /** Layout name from frontmatter */
  layout?: string;
}

/**
 * Extended page props with custom frontmatter.
 */
export type PageProps<T extends Record<string, unknown> = Record<string, unknown>> =
  BasePageProps & {
    /** Custom frontmatter fields */
    frontmatter: T & Record<string, unknown>;
  };

/**
 * Site-wide configuration available in context.
 */
export interface SiteConfig {
  /** Site name */
  name: string;
  /** Base URL path */
  base: string;
  /** All pages in the site */
  pages: BasePageProps[];
  /** Navigation groups */
  nav: NavGroup[];
}

/**
 * Navigation group.
 */
export interface NavGroup {
  title: string;
  items: NavItem[];
}

/**
 * Navigation item.
 */
export interface NavItem {
  title: string;
  path: string;
  href: string;
}

/**
 * Complete render context.
 */
export interface RenderContext<T extends Record<string, unknown> = Record<string, unknown>> {
  /** Current page props */
  page: PageProps<T>;
  /** Site configuration */
  site: SiteConfig;
}

// Internal context storage (set during render)
let currentContext: RenderContext | null = null;

/**
 * Sets the current render context.
 * Called internally during page rendering.
 * @internal
 */
export function setRenderContext(ctx: RenderContext): void {
  currentContext = ctx;
}

/**
 * Clears the current render context.
 * Called internally after page rendering.
 * @internal
 */
export function clearRenderContext(): void {
  currentContext = null;
}

/**
 * Gets the current page props.
 *
 * @returns The current page props
 * @throws Error if called outside of a render context
 *
 * @example
 * ```tsx
 * function PageTitle() {
 *   const page = usePageProps();
 *   return <h1>{page.title}</h1>;
 * }
 * ```
 */
export function usePageProps<
  T extends Record<string, unknown> = Record<string, unknown>,
>(): PageProps<T> {
  if (!currentContext) {
    throw new Error(
      "[ox-content] usePageProps() must be called during page rendering. " +
        "Make sure you are using it inside a theme component.",
    );
  }
  return currentContext.page as PageProps<T>;
}

/**
 * Gets the site configuration.
 *
 * @returns The site configuration
 * @throws Error if called outside of a render context
 *
 * @example
 * ```tsx
 * function SiteHeader() {
 *   const site = useSiteConfig();
 *   return <header>{site.name}</header>;
 * }
 * ```
 */
export function useSiteConfig(): SiteConfig {
  if (!currentContext) {
    throw new Error(
      "[ox-content] useSiteConfig() must be called during page rendering. " +
        "Make sure you are using it inside a theme component.",
    );
  }
  return currentContext.site;
}

/**
 * Gets the full render context.
 *
 * @returns The complete render context
 * @throws Error if called outside of a render context
 *
 * @example
 * ```tsx
 * function Layout({ children }) {
 *   const ctx = useRenderContext();
 *   return (
 *     <html>
 *       <head><title>{ctx.page.title} - {ctx.site.name}</title></head>
 *       <body>{children}</body>
 *     </html>
 *   );
 * }
 * ```
 */
export function useRenderContext<
  T extends Record<string, unknown> = Record<string, unknown>,
>(): RenderContext<T> {
  if (!currentContext) {
    throw new Error(
      "[ox-content] useRenderContext() must be called during page rendering. " +
        "Make sure you are using it inside a theme component.",
    );
  }
  return currentContext as RenderContext<T>;
}

/**
 * Gets the navigation groups.
 *
 * @example
 * ```tsx
 * function Sidebar() {
 *   const nav = useNav();
 *   return (
 *     <nav>
 *       {each(nav, (group) => (
 *         <div>
 *           <h3>{group.title}</h3>
 *           <ul>
 *             {each(group.items, (item) => (
 *               <li><a href={item.href}>{item.title}</a></li>
 *             ))}
 *           </ul>
 *         </div>
 *       ))}
 *     </nav>
 *   );
 * }
 * ```
 */
export function useNav(): NavGroup[] {
  return useSiteConfig().nav;
}

/**
 * Checks if the given path is the current page.
 *
 * @example
 * ```tsx
 * function NavLink({ href, children }) {
 *   const isActive = useIsActive(href);
 *   return <a href={href} class={isActive ? 'active' : ''}>{children}</a>;
 * }
 * ```
 */
export function useIsActive(path: string): boolean {
  const page = usePageProps();
  return page.path === path || page.url === path;
}

// Type generation helpers

/**
 * Schema for frontmatter type generation.
 */
export interface FrontmatterSchema {
  /** Field name */
  name: string;
  /** TypeScript type */
  type: string;
  /** Whether the field is optional */
  optional: boolean;
  /** JSDoc description */
  description?: string;
}

/**
 * Infers TypeScript types from frontmatter values.
 */
export function inferType(value: unknown): string {
  if (value === null) return "null";
  if (value === undefined) return "undefined";
  if (typeof value === "string") return "string";
  if (typeof value === "number") return "number";
  if (typeof value === "boolean") return "boolean";
  if (Array.isArray(value)) {
    if (value.length === 0) return "unknown[]";
    const itemTypes = [...new Set(value.map(inferType))];
    if (itemTypes.length === 1) return `${itemTypes[0]}[]`;
    return `(${itemTypes.join(" | ")})[]`;
  }
  if (typeof value === "object") {
    const entries = Object.entries(value as Record<string, unknown>);
    if (entries.length === 0) return "Record<string, unknown>";
    const props = entries.map(([k, v]) => `${k}: ${inferType(v)}`).join("; ");
    return `{ ${props} }`;
  }
  return "unknown";
}

/**
 * Generates TypeScript interface from frontmatter samples.
 */
export function generateFrontmatterTypes(
  samples: Record<string, unknown>[],
  interfaceName = "PageFrontmatter",
): string {
  // Collect all fields and their types across all samples
  const fields = new Map<string, { types: Set<string>; count: number }>();

  for (const sample of samples) {
    for (const [key, value] of Object.entries(sample)) {
      const existing = fields.get(key) ?? { types: new Set(), count: 0 };
      existing.types.add(inferType(value));
      existing.count++;
      fields.set(key, existing);
    }
  }

  // Generate interface
  const lines: string[] = [
    "/**",
    " * Auto-generated frontmatter type based on your pages.",
    " * DO NOT EDIT - this file is generated by ox-content.",
    " */",
    "",
    `export interface ${interfaceName} {`,
  ];

  for (const [name, { types, count }] of fields) {
    const isOptional = count < samples.length;
    const typeStr = [...types].join(" | ");
    const optionalMark = isOptional ? "?" : "";
    lines.push(`  ${name}${optionalMark}: ${typeStr};`);
  }

  lines.push("}");
  lines.push("");
  lines.push(
    `export type PageProps = import('@ox-content/vite-plugin').PageProps<${interfaceName}>;`,
  );
  lines.push("");

  return lines.join("\n");
}
