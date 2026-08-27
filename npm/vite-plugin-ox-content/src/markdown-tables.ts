export interface MarkdownTableEnhancementOptions {
  /**
   * Tables to enhance. Defaults to Markdown content tables styled by
   * `@ox-content/vite-plugin/styles/core.css`.
   */
  selector?: string;
  /**
   * Accessible name applied only when an overflowing table has no existing
   * `aria-label` or `aria-labelledby`.
   */
  label?: string;
}

const TABINDEX_FLAG = "oxTableScrollTabindex";
const LABEL_FLAG = "oxTableScrollLabel";
const SCROLLABLE_ATTR = "data-ox-table-scrollable";

export function markdownTableScrollLabel(locale?: string): string {
  const language = locale?.split("-")[0]?.toLowerCase();
  if (language === "ja") {
    return "横スクロールできる表";
  }
  return "Scrollable table";
}

/**
 * Makes overflowing Markdown tables keyboard-scrollable without wrapping or
 * replacing the native `<table>` element.
 *
 * Run this after rendering Markdown and again after layout-changing updates.
 * Narrow tables stay out of the tab order when overflow can be measured.
 */
export function enhanceMarkdownTables(
  root?: ParentNode,
  options: MarkdownTableEnhancementOptions = {},
): number {
  const target = root ?? globalThis.document;
  if (!target?.querySelectorAll) {
    return 0;
  }

  const selector = options.selector ?? ".content table";
  const locale = ownerDocument(target)?.documentElement.lang;
  const label = options.label ?? markdownTableScrollLabel(locale);
  let scrollableCount = 0;

  for (const table of target.querySelectorAll(selector)) {
    if (typeof HTMLElement === "undefined" || !(table instanceof HTMLElement)) {
      continue;
    }
    if (table.tagName !== "TABLE") {
      continue;
    }

    const scrollable = table.scrollWidth > table.clientWidth + 1;
    table.toggleAttribute(SCROLLABLE_ATTR, scrollable);
    if (scrollable) {
      scrollableCount++;
      if (!table.hasAttribute("tabindex")) {
        table.tabIndex = 0;
        table.dataset[TABINDEX_FLAG] = "true";
      }
      if (!table.hasAttribute("aria-label") && !table.hasAttribute("aria-labelledby")) {
        table.setAttribute("aria-label", label);
        table.dataset[LABEL_FLAG] = "true";
      }
    } else {
      if (table.dataset[TABINDEX_FLAG] === "true") {
        table.removeAttribute("tabindex");
        delete table.dataset[TABINDEX_FLAG];
      }
      if (table.dataset[LABEL_FLAG] === "true") {
        table.removeAttribute("aria-label");
        delete table.dataset[LABEL_FLAG];
      }
    }
  }

  return scrollableCount;
}

function ownerDocument(root: ParentNode): Document | undefined {
  if (typeof Document !== "undefined" && root instanceof Document) {
    return root;
  }
  return root.ownerDocument ?? globalThis.document;
}
