/**
 * Tabs Plugin - Pure CSS implementation
 *
 * Transforms <Tabs>/<Tab> components into accessible HTML
 * with CSS :has() based tab switching (no JavaScript required).
 */

import { unified } from "unified";
import rehypeParse from "rehype-parse";
import rehypeStringify from "rehype-stringify";
import type { Root, Element } from "hast";

let tabGroupCounter = 0;

/**
 * Reset tab group counter (for testing).
 */
export function resetTabGroupCounter(): void {
  tabGroupCounter = 0;
}

/**
 * Get element attribute value.
 */
function getAttribute(el: Element, name: string): string | undefined {
  const value = el.properties?.[name];
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value.join(" ");
  return undefined;
}

interface TabData {
  label: string;
  content: Element[];
}

/**
 * Parse Tab elements from Tabs children.
 */
function parseTabChildren(children: Element["children"]): TabData[] {
  const tabs: TabData[] = [];

  for (const child of children) {
    if (child.type !== "element") continue;

    // Handle <Tab label="...">
    if (child.tagName.toLowerCase() === "tab") {
      const label = getAttribute(child, "label") || `Tab ${tabs.length + 1}`;
      tabs.push({
        label,
        content: child.children.filter(
          (c): c is Element => c.type === "element" || c.type === "text",
        ) as Element[],
      });
    }
  }

  return tabs;
}

/**
 * Create the HTML structure for tabs.
 */
function createTabsElement(tabs: TabData[], groupId: string): Element {
  const children: Element["children"] = [];

  // Create header with radio inputs and labels
  const headerChildren: Element["children"] = [];

  tabs.forEach((tab, index) => {
    const inputId = `ox-tab-${groupId}-${index}`;

    // Radio input
    headerChildren.push({
      type: "element",
      tagName: "input",
      properties: {
        type: "radio",
        name: `ox-tabs-${groupId}`,
        id: inputId,
        checked: index === 0 ? true : undefined,
      },
      children: [],
    });

    // Label
    headerChildren.push({
      type: "element",
      tagName: "label",
      properties: {
        htmlFor: inputId,
      },
      children: [{ type: "text", value: tab.label }],
    });
  });

  // Tabs header
  children.push({
    type: "element",
    tagName: "div",
    properties: { className: ["ox-tabs-header"] },
    children: headerChildren,
  });

  // Tab panels
  tabs.forEach((tab, index) => {
    children.push({
      type: "element",
      tagName: "div",
      properties: {
        className: ["ox-tab-panel"],
        "data-tab": String(index),
      },
      children: tab.content,
    });
  });

  return {
    type: "element",
    tagName: "div",
    properties: {
      className: ["ox-tabs"],
      "data-group": groupId,
    },
    children,
  };
}

/**
 * Create fallback HTML using <details> elements.
 */
function createFallbackElement(tabs: TabData[]): Element {
  const children: Element["children"] = [];

  tabs.forEach((tab, index) => {
    children.push({
      type: "element",
      tagName: "details",
      properties: {
        open: index === 0 ? true : undefined,
      },
      children: [
        {
          type: "element",
          tagName: "summary",
          properties: {},
          children: [{ type: "text", value: tab.label }],
        },
        {
          type: "element",
          tagName: "div",
          properties: { className: ["ox-tabs-fallback-content"] },
          children: tab.content,
        },
      ],
    });
  });

  return {
    type: "element",
    tagName: "noscript",
    properties: {},
    children: [
      {
        type: "element",
        tagName: "div",
        properties: { className: ["ox-tabs-fallback"] },
        children,
      },
    ],
  };
}

/**
 * Rehype plugin to transform Tabs components.
 */
function rehypeTabs() {
  return (tree: Root) => {
    const visit = (node: Root | Element) => {
      if ("children" in node) {
        for (let i = 0; i < node.children.length; i++) {
          const child = node.children[i];

          if (child.type === "element") {
            // Check for <Tabs> component
            if (child.tagName.toLowerCase() === "tabs") {
              const tabs = parseTabChildren(child.children);

              if (tabs.length > 0) {
                const groupId = String(tabGroupCounter++);
                const tabsElement = createTabsElement(tabs, groupId);
                const fallbackElement = createFallbackElement(tabs);

                // Replace <Tabs> with new structure
                // Keep main tabs and add noscript fallback
                const wrapper: Element = {
                  type: "element",
                  tagName: "div",
                  properties: { className: ["ox-tabs-container"] },
                  children: [tabsElement, fallbackElement],
                };

                node.children[i] = wrapper;
              }
            } else {
              visit(child);
            }
          }
        }
      }
    };

    visit(tree);
  };
}

/**
 * Transform Tabs components in HTML.
 */
export async function transformTabs(html: string): Promise<string> {
  // `rehypeTabs` only acts on `<Tabs>` elements. When the document has none,
  // the rehype parse/stringify round-trip is wasted work, so skip it. The
  // marker is a deliberately loose superset of the element name to avoid ever
  // skipping a page that actually contains a `<Tabs>`.
  if (!/<tabs/i.test(html)) {
    return html;
  }

  const result = await unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeTabs)
    .use(rehypeStringify)
    .process(html);

  return String(result);
}

/**
 * Generate dynamic CSS for :has() based tab switching.
 * This is needed because :has() selectors need unique IDs.
 */
export function generateTabsCSS(groupCount: number): string {
  if (groupCount === 0) return "";

  let css = "/* Dynamic Tabs CSS */\n";

  for (let g = 0; g < groupCount; g++) {
    for (let t = 0; t < 8; t++) {
      css += `.ox-tabs[data-group="${g}"]:has(#ox-tab-${g}-${t}:checked) .ox-tab-panel[data-tab="${t}"] { display: block; }\n`;
    }
  }

  return css;
}
