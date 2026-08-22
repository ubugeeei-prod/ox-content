/**
 * Development-runtime entry point for the static HTML JSX transform.
 *
 * TypeScript's `react-jsxdev` transform (and Vite's dev-mode JSX transform)
 * import from `<jsxImportSource>/jsx-dev-runtime` and call `jsxDEV` with the
 * extra debug arguments below. This runtime renders static HTML, so there is
 * nothing useful to do with them — they are accepted and ignored so the same
 * templates compile in both dev and build.
 *
 * @example
 * ```json
 * { "compilerOptions": { "jsx": "react-jsxdev", "jsxImportSource": "@ox-content/vite-plugin" } }
 * ```
 */

import { jsx } from "./jsx-html";
import type { JSXElementType, JSXNode, JSXProps } from "./jsx-html";

export { Fragment, jsx, jsxs } from "./jsx-html";
export type { JSX } from "./jsx-html";
export type { JSXChild, JSXElementType, JSXNode, JSXProps } from "./jsx-html";

/** Source position the dev transform attaches to each element. */
export interface JSXSource {
  fileName?: string;
  lineNumber?: number;
  columnNumber?: number;
}

/**
 * Creates a JSX element in development mode.
 *
 * The transform passes `isStaticChildren`, `source` and `self` for React's
 * dev warnings; static HTML generation has no use for them.
 */
export function jsxDEV(
  type: JSXElementType,
  props: JSXProps,
  key?: string,
  _isStaticChildren?: boolean,
  _source?: JSXSource,
  _self?: unknown,
): JSXNode {
  return jsx(type, props, key);
}
