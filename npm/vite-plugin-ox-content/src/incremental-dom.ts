/// <reference lib="dom" />

export {
  DEFAULT_ATOM_CLASS,
  DEFAULT_CHAR_CLASS,
  DEFAULT_COMMITTED_CLASS,
  DEFAULT_PENDING_CLASS,
  DEFAULT_VISIBLE_CLASS,
  incrementalMarkdownDomStyles,
  injectIncrementalMarkdownDomStyles,
} from "./incremental-dom-styles";
export { createIncrementalMarkdownDomRenderer } from "./incremental-dom-renderer";
export type {
  IncrementalMarkdownDomAnimationOptions,
  IncrementalMarkdownDomApplyOptions,
  IncrementalMarkdownDomOptions,
  IncrementalMarkdownDomRenderer,
  IncrementalMarkdownRenderResultLike,
} from "./incremental-dom-types";
