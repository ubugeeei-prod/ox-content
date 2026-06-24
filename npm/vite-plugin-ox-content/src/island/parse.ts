/**
 * Island Parser
 *
 * Detects <Island> components in HTML and transforms them
 * into hydration-ready elements with data attributes.
 */

import { unified } from "unified";
import rehypeParsePlugin from "rehype-parse";
import rehypeStringifyPlugin from "rehype-stringify";
import type { Root, Element } from "hast";
import { interopDefault } from "../interop";

// ESM-only plugins are double-wrapped by the CommonJS interop; unwrap. See #452.
const rehypeParse = interopDefault(rehypeParsePlugin);
const rehypeStringify = interopDefault(rehypeStringifyPlugin);

export type LoadStrategy = "eager" | "idle" | "visible" | "media";

export interface IslandInfo {
  component: string;
  load: LoadStrategy;
  mediaQuery?: string;
  props: Record<string, unknown>;
}

export interface ParseIslandsResult {
  html: string;
  islands: IslandInfo[];
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

/**
 * Parse JSX-style props from attributes.
 */
function parseProps(el: Element): Record<string, unknown> {
  const props: Record<string, unknown> = {};

  if (!el.properties) return props;

  for (const [key, value] of Object.entries(el.properties)) {
    // Skip special attributes
    if (["load", "media", "className", "class"].includes(key)) continue;

    // Handle JSX-style props like {0} or {true}
    if (typeof value === "string") {
      // Try to parse as JSON/JS value if it looks like one
      const trimmed = value.trim();
      if (trimmed.startsWith("{") && trimmed.endsWith("}")) {
        const inner = trimmed.slice(1, -1);
        try {
          // Try JSON parse first
          props[key] = JSON.parse(inner);
        } catch {
          // Try evaluating simple expressions
          if (inner === "true") props[key] = true;
          else if (inner === "false") props[key] = false;
          else if (inner === "null") props[key] = null;
          else if (!Number.isNaN(Number(inner))) props[key] = Number(inner);
          else props[key] = value;
        }
      } else {
        props[key] = value;
      }
    } else if (typeof value === "number" || typeof value === "boolean") {
      props[key] = value;
    } else if (Array.isArray(value)) {
      props[key] = value;
    }
  }

  return props;
}

/**
 * Find the component element inside <Island>.
 */
function findComponentElement(children: Element["children"]): Element | null {
  for (const child of children) {
    if (child.type === "element") {
      // Skip text/whitespace, look for actual component
      if (child.tagName !== "br" && child.tagName !== "span") {
        return child;
      }
    }
  }
  return null;
}

/**
 * Get component name from child element.
 */
function getComponentName(el: Element): string {
  // PascalCase tag names are components
  const tagName = el.tagName;
  if (tagName && /^[A-Z]/.test(tagName)) {
    return tagName;
  }
  // Check for data-component attribute
  return getAttribute(el, "data-component") || tagName;
}

let islandCounter = 0;

/**
 * Reset island counter (for testing).
 */
export function resetIslandCounter(): void {
  islandCounter = 0;
}

/**
 * Rehype plugin to transform Island components.
 */
function rehypeIslands(collectedIslands: IslandInfo[]) {
  return (tree: Root) => {
    const visit = (node: Root | Element) => {
      if ("children" in node) {
        for (let i = 0; i < node.children.length; i++) {
          const child = node.children[i];

          if (child.type === "element") {
            // Check for <Island> component
            if (child.tagName.toLowerCase() === "island") {
              const load = (getAttribute(child, "load") as LoadStrategy) || "eager";
              const mediaQuery = getAttribute(child, "media");

              // Find the component inside
              const componentEl = findComponentElement(child.children);

              if (componentEl) {
                const componentName = getComponentName(componentEl);
                const componentProps = parseProps(componentEl);

                // Collect island info
                const islandInfo: IslandInfo = {
                  component: componentName,
                  load,
                  mediaQuery,
                  props: componentProps,
                };
                collectedIslands.push(islandInfo);

                // Create island wrapper with data attributes
                const islandId = `ox-island-${islandCounter++}`;

                const islandElement: Element = {
                  type: "element",
                  tagName: "div",
                  properties: {
                    id: islandId,
                    "data-ox-island": componentName,
                    "data-ox-load": load,
                    ...(mediaQuery && { "data-ox-media": mediaQuery }),
                    "data-ox-props": JSON.stringify(componentProps),
                    className: ["ox-island"],
                  },
                  children: [
                    // Keep original content as fallback/placeholder
                    ...componentEl.children,
                  ],
                };

                node.children[i] = islandElement;
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
 * Transform Island components in HTML.
 *
 * Converts:
 * ```html
 * <Island load="visible">
 *   <Counter initial={0} />
 * </Island>
 * ```
 *
 * To:
 * ```html
 * <div id="ox-island-0"
 *      data-ox-island="Counter"
 *      data-ox-load="visible"
 *      data-ox-props='{"initial":0}'
 *      class="ox-island">
 *   <!-- fallback content -->
 * </div>
 * ```
 */
export async function transformIslands(html: string): Promise<ParseIslandsResult> {
  const islands: IslandInfo[] = [];

  const result = await unified()
    .use(rehypeParse, { fragment: true })
    .use(rehypeIslands, islands)
    .use(rehypeStringify)
    .process(html);

  return {
    html: String(result),
    islands,
  };
}

/**
 * Check if HTML contains any Island components.
 */
export function hasIslands(html: string): boolean {
  return /<island[\s>]/i.test(html);
}

/**
 * Extract island info without transforming HTML.
 * Useful for analysis/bundling purposes.
 */
export async function extractIslandInfo(html: string): Promise<IslandInfo[]> {
  const { islands } = await transformIslands(html);
  return islands;
}

/**
 * Generate client-side hydration script.
 * This is a minimal script that imports and initializes islands.
 */
export function generateHydrationScript(components: string[]): string {
  if (components.length === 0) return "";

  const imports = components.map((name) => `import ${name} from './${name}';`).join("\n");

  return `
import { initIslands } from '@ox-content/islands';
${imports}

const components = {
  ${components.join(",\n  ")}
};

// Initialize with your framework's hydration
// This example uses Vue, adapt for React/Svelte/etc.
import { createApp, h } from 'vue';

initIslands((el, props) => {
  const name = el.dataset.oxIsland;
  const Component = components[name];
  if (!Component) {
    console.warn(\`[ox-islands] Unknown component: \${name}\`);
    return;
  }

  const app = createApp({ render: () => h(Component, props) });
  app.mount(el);

  return () => app.unmount();
});
`;
}
