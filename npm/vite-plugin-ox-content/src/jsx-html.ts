/**
 * Custom JSX Runtime for Static HTML Generation
 *
 * This module provides a JSX runtime that outputs static HTML strings.
 * No React, no hydration, no client-side JavaScript - just pure HTML.
 *
 * This is the implementation. The JSX transform imports it through the
 * ./jsx-runtime and ./jsx-dev-runtime entry points, which are the subpaths
 * `jsxImportSource` resolves. Keeping the implementation under its own name
 * keeps those two entries from colliding with the shared bundle chunk.
 *
 * @example
 * ```tsx
 * // tsconfig.json or vite.config.ts
 * {
 *   "compilerOptions": {
 *     "jsx": "react-jsx",
 *     "jsxImportSource": "@ox-content/vite-plugin"
 *   }
 * }
 *
 * // MyComponent.tsx
 * export function Hero({ title }: { title: string }) {
 *   return (
 *     <section class="hero">
 *       <h1>{title}</h1>
 *     </section>
 *   );
 * }
 * ```
 */

// `jsxImportSource` makes TypeScript look for the element types in
// `<source>/jsx-runtime` itself, so the namespace has to travel with the
// runtime exports and not only be declared globally in ./jsx.d.ts.
export type { JSX } from "./jsx";

// Self-closing tags that don't need closing tags
const VOID_ELEMENTS = new Set([
  "area",
  "base",
  "br",
  "col",
  "embed",
  "hr",
  "img",
  "input",
  "link",
  "meta",
  "param",
  "source",
  "track",
  "wbr",
]);

// Attributes that should be rendered as boolean
const BOOLEAN_ATTRS = new Set([
  "allowfullscreen",
  "async",
  "autofocus",
  "autoplay",
  "checked",
  "controls",
  "default",
  "defer",
  "disabled",
  "formnovalidate",
  "hidden",
  "inert",
  "ismap",
  "itemscope",
  "loop",
  "multiple",
  "muted",
  "nomodule",
  "novalidate",
  "open",
  "playsinline",
  "readonly",
  "required",
  "reversed",
  "selected",
]);

/**
 * Escapes HTML special characters to prevent XSS.
 */
function escapeHtml(str: string): string {
  return str
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#x27;");
}

/**
 * React prop names whose HTML spelling is not the lowercased identifier.
 */
const ATTR_ALIASES: Record<string, string> = {
  className: "class",
  htmlFor: "for",
  httpEquiv: "http-equiv",
  acceptCharset: "accept-charset",
};

/** SVG attributes that stay camelCase in HTML (they are case-sensitive). */
const SVG_CAMEL_CASE = new Set(["viewBox"]);

/**
 * Converts a JSX/React prop name to the HTML attribute the runtime emits.
 *
 * HTML attributes are case-insensitive and conventionally lowercase
 * (`charSet` → `charset`, `tabIndex` → `tabindex`). A few React names are
 * not a lowercase of themselves (`className`, `httpEquiv`), and SVG names
 * like `viewBox` must keep their case.
 */
function toHtmlAttr(name: string): string {
  const aliased = ATTR_ALIASES[name];
  if (aliased !== undefined) return aliased;
  if (SVG_CAMEL_CASE.has(name)) return name;
  if (name.startsWith("data") || name.startsWith("aria")) {
    return name.replace(/([A-Z])/g, "-$1").toLowerCase();
  }
  if (/[A-Z]/.test(name)) {
    // SVG presentation attributes are kebab-case; remaining HTML names are
    // just the lowercased identifier (`charSet` → `charset`).
    if (name.startsWith("stroke") || name === "clipPath") {
      return name.replace(/([A-Z])/g, "-$1").toLowerCase();
    }
    return name.toLowerCase();
  }
  return name;
}

/**
 * Renders an attribute value to a string.
 */
function renderAttr(name: string, value: unknown): string {
  const htmlName = toHtmlAttr(name);

  // Skip undefined, null, and false values
  if (value === undefined || value === null || value === false) {
    return "";
  }

  // Boolean attributes
  if (BOOLEAN_ATTRS.has(htmlName)) {
    return value ? ` ${htmlName}` : "";
  }

  // Style object to string
  if (name === "style" && typeof value === "object") {
    const styleStr = Object.entries(value as Record<string, string | number>)
      .map(([k, v]) => {
        const prop = k.replace(/([A-Z])/g, "-$1").toLowerCase();
        return `${prop}:${v}`;
      })
      .join(";");
    return ` style="${escapeHtml(styleStr)}"`;
  }

  // Regular attribute
  return ` ${htmlName}="${escapeHtml(String(value as string | number | boolean))}"`;
}

/**
 * JSX element type - either a string (intrinsic) or a function component.
 */
export type JSXElementType = string | ((props: Record<string, unknown>) => JSXNode);

/**
 * Valid JSX child types.
 */
export type JSXChild = string | number | boolean | null | undefined | JSXNode | JSXChild[];

/**
 * JSX node - the result of JSX expressions.
 */
export interface JSXNode {
  __html: string;
}

/**
 * Props with children.
 */
export interface JSXProps {
  children?: JSXChild;
  [key: string]: unknown;
}

/**
 * Renders children to HTML string.
 */
function renderChildren(children: JSXChild): string {
  if (children === null || children === undefined || children === false) {
    return "";
  }

  if (children === true) {
    return "";
  }

  if (typeof children === "string") {
    return escapeHtml(children);
  }

  if (typeof children === "number") {
    return String(children);
  }

  if (Array.isArray(children)) {
    return children.map(renderChildren).join("");
  }

  if (typeof children === "object" && "__html" in children) {
    return children.__html;
  }

  return "";
}

/**
 * Creates a JSX element.
 * This is the core function called by the JSX transform.
 */
export function jsx(type: JSXElementType, props: JSXProps, _key?: string): JSXNode {
  const { children, ...restProps } = props;

  // Function component
  if (typeof type === "function") {
    return type({ ...restProps, children });
  }

  // Intrinsic element (HTML tag)
  const tag = type;
  let html = `<${tag}`;

  // Normalize React aliases first so `class` + `className` become one attribute
  // (last write wins) instead of emitting `class` twice.
  const attrs = new Map<string, unknown>();
  for (const [name, value] of Object.entries(restProps)) {
    if (name === "key" || name === "ref") continue;
    attrs.set(toHtmlAttr(name), value);
  }
  for (const [name, value] of attrs) {
    html += renderAttr(name, value);
  }

  // Self-closing tags
  if (VOID_ELEMENTS.has(tag)) {
    html += " />";
    return { __html: html };
  }

  html += ">";

  // Render children
  if (children !== undefined) {
    html += renderChildren(children);
  }

  html += `</${tag}>`;

  return { __html: html };
}

/**
 * Creates a JSX element with static children.
 * Called by the JSX transform for elements with multiple children.
 */
export function jsxs(type: JSXElementType, props: JSXProps, key?: string): JSXNode {
  return jsx(type, props, key);
}

/**
 * Fragment component - renders children without a wrapper element.
 */
export function Fragment({ children }: { children?: JSXChild }): JSXNode {
  return { __html: renderChildren(children) };
}

/**
 * Renders a JSX node to an HTML string.
 */
export function renderToString(node: JSXNode): string {
  return node.__html;
}

/**
 * Creates raw HTML without escaping.
 * Use with caution - only for trusted content.
 *
 * @example
 * ```tsx
 * <div>{raw('<strong>Bold</strong>')}</div>
 * ```
 */
export function raw(html: string): JSXNode {
  return { __html: html };
}

/**
 * Conditionally renders content.
 *
 * @example
 * ```tsx
 * {when(isLoggedIn, <UserMenu />)}
 * ```
 */
export function when(condition: boolean, content: JSXNode): JSXNode {
  return condition ? content : { __html: "" };
}

/**
 * Maps over an array and renders each item.
 *
 * @example
 * ```tsx
 * {each(items, (item) => <li>{item.name}</li>)}
 * ```
 */
export function each<T>(items: T[], render: (item: T, index: number) => JSXNode): JSXNode {
  const html = items.map((item, i) => render(item, i).__html).join("");
  return { __html: html };
}

// Default export for convenience
export default {
  jsx,
  jsxs,
  Fragment,
  renderToString,
  raw,
  when,
  each,
};
