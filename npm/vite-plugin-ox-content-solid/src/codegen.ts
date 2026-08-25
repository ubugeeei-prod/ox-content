/**
 * Generates the Solid module a Markdown file compiles to.
 *
 * The output is JSX on purpose. Solid has no runtime element factory to target
 * — `vite-plugin-solid` compiles this into DOM or SSR instructions — so unlike
 * the React and Vue integrations there is no factory-call form to emit.
 */

import * as path from "path";
import type { ComponentIsland, ResolvedSolidOptions } from "./types";

export function generateSolidModule(
  content: string,
  usedComponents: string[],
  _islands: ComponentIsland[] | string[],
  frontmatter: Record<string, unknown>,
  options: ResolvedSolidOptions & { root?: string },
  id: string,
): string {
  const rawHtml = JSON.stringify(content);
  const frontmatterLiteral = JSON.stringify(frontmatter);

  // Markdown with no registered component skips the island runtime entirely and
  // compiles to a single `innerHTML` binding — no imports needed, because Solid
  // injects whatever its compiled output requires.
  if (usedComponents.length === 0) {
    return `
export const frontmatter = ${frontmatterLiteral};

const rawHtml = ${rawHtml};

export default function MarkdownContent() {
  return <div class="ox-content" innerHTML={rawHtml} />;
}
`;
  }

  const imports = renderComponentImports(usedComponents, options, id);
  const componentMap = usedComponents.map((name) => `  ${name},`).join("\n");

  return `
import { onCleanup, onMount } from 'solid-js';
import { render } from 'solid-js/web';
import { initIslands, readIslandSlotHtml } from '@ox-content/islands';
${imports}

export const frontmatter = ${frontmatterLiteral};

const rawHtml = ${rawHtml};
const components = {
${componentMap}
};

function createSolidHydrate() {
  return (element, props) => {
    const componentName = element.dataset.oxIsland;
    const Component = components[componentName];
    if (!Component) return;

    // Read the slot content before clearing: the island element still holds the
    // markup the Markdown transform left behind.
    const islandContent = readIslandSlotHtml(element);
    element.innerHTML = '';

    const dispose = render(
      () =>
        islandContent
          ? <Component {...props}><div innerHTML={islandContent} /></Component>
          : <Component {...props} />,
      element,
    );

    return () => dispose();
  };
}

export default function MarkdownContent() {
  let container;

  onMount(() => {
    if (!container) return;
    const controller = initIslands(createSolidHydrate(), {
      selector: '.ox-content [data-ox-island]',
    });
    onCleanup(() => controller.destroy());
  });

  return <div class="ox-content" ref={container} innerHTML={rawHtml} />;
}
`;
}

/** Rewrites registered component paths as imports relative to the Markdown file. */
function renderComponentImports(
  usedComponents: string[],
  options: ResolvedSolidOptions & { root?: string },
  id: string,
): string {
  const mdDir = path.dirname(id);
  const root = options.root || process.cwd();

  return usedComponents
    .map((name) => {
      const componentPath = options.components[name];
      if (!componentPath) return "";
      const absolutePath = path.resolve(root, componentPath.replace(/^\.\//, ""));
      const relativePath = path.relative(mdDir, absolutePath).replace(/\\/g, "/");
      const importPath = relativePath.startsWith(".") ? relativePath : "./" + relativePath;
      return `import ${name} from '${importPath}';`;
    })
    .filter(Boolean)
    .join("\n");
}
