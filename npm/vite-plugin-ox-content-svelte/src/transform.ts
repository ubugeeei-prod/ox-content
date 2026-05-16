import * as path from "path";
import { transformMarkdown as baseTransformMarkdown } from "@ox-content/vite-plugin";
import { compile } from "svelte/compiler";
import type {
  ResolvedSvelteOptions,
  SvelteTransformResult,
  ComponentIsland,
  ComponentsMap,
} from "./types";

const COMPONENT_REGEX = /<([A-Z][a-zA-Z0-9]*)\s*([^>]*?)\s*(?:\/>|>([\s\S]*?)<\/\1>)/g;
const PROP_REGEX = /([a-zA-Z0-9-]+)(?:=(?:"([^"]*)"|'([^']*)'|{([^}]*)}|\[([^\]]*)\]))?/g;

const ISLAND_MARKER_PREFIX = "OXCONTENT-ISLAND-";
const ISLAND_MARKER_SUFFIX = "-PLACEHOLDER";

interface Range {
  start: number;
  end: number;
}

export async function transformMarkdownWithSvelte(
  code: string,
  id: string,
  options: ResolvedSvelteOptions,
): Promise<SvelteTransformResult> {
  const components: ComponentsMap = options.components;
  const usedComponents: string[] = [];
  const islands: ComponentIsland[] = [];
  let islandIndex = 0;

  const { content: markdownContent, frontmatter } = extractFrontmatter(code);
  const fenceRanges = collectFenceRanges(markdownContent);
  let processedContent = "";
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  COMPONENT_REGEX.lastIndex = 0;
  while ((match = COMPONENT_REGEX.exec(markdownContent)) !== null) {
    const [fullMatch, componentName, propsString, rawIslandContent] = match;
    const matchStart = match.index;
    const matchEnd = matchStart + fullMatch.length;

    if (
      !Object.prototype.hasOwnProperty.call(components, componentName) ||
      isInRanges(matchStart, matchEnd, fenceRanges)
    ) {
      processedContent += markdownContent.slice(lastIndex, matchEnd);
      lastIndex = matchEnd;
      continue;
    }

    if (!usedComponents.includes(componentName)) {
      usedComponents.push(componentName);
    }

    const props = parseProps(propsString);
    const islandId = `ox-island-${islandIndex++}`;
    const islandContent =
      typeof rawIslandContent === "string" ? rawIslandContent.trim() : undefined;

    islands.push({
      name: componentName,
      props,
      position: matchStart,
      id: islandId,
      content: islandContent,
    });

    processedContent += markdownContent.slice(lastIndex, matchStart) + createIslandMarker(islandId);
    lastIndex = matchEnd;
  }
  processedContent += markdownContent.slice(lastIndex);

  const baseOptions = {
    srcDir: options.srcDir,
    outDir: options.outDir,
    base: options.base,
    extensions: options.extensions,
    ssg: {
      enabled: false,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
    },
    gfm: options.gfm,
    frontmatter: false,
    toc: options.toc,
    tocMaxDepth: options.tocMaxDepth,
    codeAnnotations: options.codeAnnotations,
    footnotes: true,
    tables: true,
    taskLists: true,
    strikethrough: true,
    highlight: false,
    highlightTheme: "github-dark",
    highlightLangs: [],
    mermaid: false,
    ogImage: false,
    ogImageOptions: {
      vuePlugin: "vitejs",
      width: 1200,
      height: 630,
      cache: true,
      concurrency: 1,
    },
    transformers: [],
    docs: false,
    ogViewer: false,
    search: {
      enabled: false,
      limit: 10,
      prefix: true,
      placeholder: "Search...",
      hotkey: "k",
    },
    i18n: false,
  } as unknown as Parameters<typeof baseTransformMarkdown>[2] & {
    codeAnnotations?: ResolvedSvelteOptions["codeAnnotations"];
  };

  const transformed = await baseTransformMarkdown(processedContent, id, baseOptions);

  const htmlWithIslands = injectIslandMarkers(transformed.html, islands);
  const svelteCode = generateSvelteModule(
    htmlWithIslands,
    usedComponents,
    islands,
    frontmatter,
    options,
    id,
  );

  const compiled = compile(svelteCode, {
    filename: id,
    generate: "client",
    runes: true,
  });

  const finalCode = `${compiled.js.code}\nexport const frontmatter = ${JSON.stringify(frontmatter)};`;

  return {
    code: finalCode,
    map: null,
    usedComponents,
    frontmatter,
  };
}

function createIslandMarker(islandId: string): string {
  return `${ISLAND_MARKER_PREFIX}${islandId}${ISLAND_MARKER_SUFFIX}`;
}

function collectFenceRanges(content: string): Range[] {
  const ranges: Range[] = [];
  let inFence = false;
  let fenceChar = "";
  let fenceLength = 0;
  let fenceStart = 0;
  let pos = 0;

  while (pos < content.length) {
    const lineEnd = content.indexOf("\n", pos);
    const next = lineEnd === -1 ? content.length : lineEnd + 1;
    const line = content.slice(pos, lineEnd === -1 ? content.length : lineEnd);
    const fenceMatch = line.match(/^\s{0,3}([`~]{3,})/);

    if (fenceMatch) {
      const marker = fenceMatch[1];
      if (!inFence) {
        inFence = true;
        fenceChar = marker[0];
        fenceLength = marker.length;
        fenceStart = pos;
      } else if (marker[0] === fenceChar && marker.length >= fenceLength) {
        inFence = false;
        ranges.push({ start: fenceStart, end: next });
        fenceChar = "";
        fenceLength = 0;
      }
    }

    pos = next;
  }

  if (inFence) {
    ranges.push({ start: fenceStart, end: content.length });
  }

  return ranges;
}

function isInRanges(start: number, end: number, ranges: Range[]): boolean {
  for (const range of ranges) {
    if (start < range.end && end > range.start) {
      return true;
    }
  }
  return false;
}

function injectIslandMarkers(html: string, islands: ComponentIsland[]): string {
  let output = html;

  for (const island of islands) {
    const marker = createIslandMarker(island.id);
    const propsAttr =
      Object.keys(island.props).length > 0
        ? ` data-ox-props='${JSON.stringify(island.props).replace(/'/g, "&#39;")}'`
        : "";
    const contentAttr = island.content
      ? ` data-ox-content='${island.content.replace(/'/g, "&#39;")}'`
      : "";
    const attrs = `data-ox-island="${island.name}"${propsAttr}${contentAttr}`;
    output = output.replaceAll(`<p>${marker}</p>`, `<div ${attrs}></div>`);
    output = output.replaceAll(marker, `<span ${attrs}></span>`);
  }

  return output;
}

function extractFrontmatter(content: string): {
  content: string;
  frontmatter: Record<string, unknown>;
} {
  const frontmatterRegex = /^---\n([\s\S]*?)\n---\n/;
  const match = frontmatterRegex.exec(content);

  if (!match) {
    return { content, frontmatter: {} };
  }

  const frontmatterStr = match[1];
  const frontmatter: Record<string, unknown> = {};

  for (const line of frontmatterStr.split("\n")) {
    const colonIndex = line.indexOf(":");
    if (colonIndex > 0) {
      const key = line.slice(0, colonIndex).trim();
      let value: unknown = line.slice(colonIndex + 1).trim();
      try {
        value = JSON.parse(value as string);
      } catch {
        if (
          typeof value === "string" &&
          ((value.startsWith('"') && value.endsWith('"')) ||
            (value.startsWith("'") && value.endsWith("'")))
        ) {
          value = value.slice(1, -1);
        }
      }
      frontmatter[key] = value;
    }
  }

  return { content: content.slice(match[0].length), frontmatter };
}

function parseProps(propsString: string): Record<string, unknown> {
  const props: Record<string, unknown> = {};
  if (!propsString) return props;

  PROP_REGEX.lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = PROP_REGEX.exec(propsString)) !== null) {
    const [, name, doubleQuoted, singleQuoted, braceValue, bracketValue] = match;
    if (name) {
      if (doubleQuoted !== undefined) props[name] = doubleQuoted;
      else if (singleQuoted !== undefined) props[name] = singleQuoted;
      else if (braceValue !== undefined) {
        try {
          props[name] = JSON.parse(braceValue);
        } catch {
          props[name] = braceValue;
        }
      } else if (bracketValue !== undefined) {
        try {
          props[name] = JSON.parse(`[${bracketValue}]`);
        } catch {
          props[name] = bracketValue;
        }
      } else props[name] = true;
    }
  }
  return props;
}

function generateSvelteModule(
  content: string,
  usedComponents: string[],
  islands: ComponentIsland[],
  frontmatter: Record<string, unknown>,
  options: ResolvedSvelteOptions & { root?: string },
  id: string,
): string {
  const mdDir = path.dirname(id);
  const root = options.root || process.cwd();

  const imports = usedComponents
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

  // If no islands, generate simpler code without island runtime
  if (islands.length === 0) {
    return `
<script>
  const frontmatter = ${JSON.stringify(frontmatter)};
  const rawHtml = ${JSON.stringify(content)};

  export { frontmatter };
</script>

<div class="ox-content">
  {@html rawHtml}
</div>

<style>
  .ox-content {
    line-height: 1.6;
  }
</style>
`;
  }

  const componentMap = usedComponents.map((name) => `  ${name},`).join("\n");

  return `
<script>
  import { createRawSnippet, onMount, mount, unmount } from 'svelte';
  import { initIslands } from '@ox-content/islands';
  ${imports}

  const frontmatter = ${JSON.stringify(frontmatter)};
  const rawHtml = ${JSON.stringify(content)};
  const components = {
${componentMap}
  };

  export { frontmatter };

  let container;

  function createSvelteHydrate() {
    const mounted = [];

    return (element, props) => {
      const componentName = element.dataset.oxIsland;
      const Component = components[componentName];
      if (!Component) return;

      const islandContent = element.dataset.oxContent || element.innerHTML;
      const componentProps = { ...props };
      if (islandContent) {
        componentProps.children = createRawSnippet(() => ({
          render: () => \`<div>\${islandContent}</div>\`,
        }));
      }

      const instance = mount(Component, { target: element, props: componentProps });
      mounted.push(instance);

      return () => unmount(instance);
    };
  }

  onMount(() => {
    if (!container) return;
    const controller = initIslands(createSvelteHydrate(), {
      selector: '.ox-content [data-ox-island]',
    });
    return () => controller.destroy();
  });
</script>

<div class="ox-content" bind:this={container}>
  {@html rawHtml}
</div>

<style>
  .ox-content {
    line-height: 1.6;
  }
</style>
`;
}
