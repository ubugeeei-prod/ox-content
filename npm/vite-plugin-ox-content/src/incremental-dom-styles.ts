/// <reference lib="dom" />

export const DEFAULT_COMMITTED_CLASS = "ox-incremental-committed";
export const DEFAULT_PENDING_CLASS = "ox-incremental-pending";
export const DEFAULT_CHAR_CLASS = "ox-incremental-char";
export const DEFAULT_VISIBLE_CLASS = "ox-incremental-char--visible";
export const DEFAULT_ATOM_CLASS = "ox-incremental-atom";

export const incrementalMarkdownDomStyles = `
.${DEFAULT_COMMITTED_CLASS},
.${DEFAULT_PENDING_CLASS} {
  display: contents;
}

.${DEFAULT_CHAR_CLASS},
.${DEFAULT_ATOM_CLASS} {
  animation: ox-incremental-char-in var(--ox-incremental-duration, 170ms) ease-out both;
  animation-delay: var(--ox-incremental-delay, 0ms);
}

.${DEFAULT_VISIBLE_CLASS} {
  animation: none;
  opacity: 1;
}

@keyframes ox-incremental-char-in {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}
`.trim();

export function injectIncrementalMarkdownDomStyles(
  document: Document = globalThis.document,
): HTMLStyleElement {
  const existing = document.querySelector<HTMLStyleElement>(
    "style[data-ox-incremental-dom]",
  );
  if (existing) {
    return existing;
  }

  const style = document.createElement("style");
  style.dataset.oxIncrementalDom = "";
  style.textContent = incrementalMarkdownDomStyles;
  document.head.append(style);
  return style;
}
