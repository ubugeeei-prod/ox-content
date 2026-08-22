/**
 * Automatic-runtime entry point for the static HTML JSX transform.
 *
 * `jsxImportSource` makes the compiler emit `import { jsx } from
 * "<source>/jsx-runtime"`, so this subpath has to exist as its own module —
 * the same symbols on the package's main entry cannot satisfy it.
 *
 * @example
 * ```json
 * { "compilerOptions": { "jsx": "react-jsx", "jsxImportSource": "@ox-content/vite-plugin" } }
 * ```
 *
 * The implementation lives in ./jsx-html; this file only re-exports what the
 * transform reaches for. Named exports only, matching React's runtime shape.
 */

export { Fragment, jsx, jsxs } from "./jsx-html";
export type { JSX } from "./jsx-html";
export type { JSXChild, JSXElementType, JSXNode, JSXProps } from "./jsx-html";
