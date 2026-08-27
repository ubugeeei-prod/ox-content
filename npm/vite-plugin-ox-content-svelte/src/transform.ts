import {
  applyIslandSsrHtml,
  discoverDocumentMdxIslands,
  renderIslandComponentImports,
  resolveContentRootPath,
  resolveMdxForFilePath,
  transformMarkdown as baseTransformMarkdown,
  type ResolvedDocumentComponentImport,
} from "@ox-content/vite-plugin";
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
const DOCUMENT_PROP_MARKER_PREFIX = "OXCONTENT-DOCUMENT-PROP-";
const DOCUMENT_PROP_MARKER_SUFFIX = "-PLACEHOLDER";
const PAYLOAD_SCRIPT = /^\s*<script type="application\/json">[\s\S]*?<\/script>/i;
const RUST_PAYLOAD_KEYS = new Set(["props", "expressions", "spreads"]);

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
  const { content: markdownContent, frontmatter } = extractFrontmatter(code);
  const mdx = resolveMdxForFilePath(id, options.mdx);

  const baseOptions = {
    srcDir: options.srcDir,
    outDir: options.outDir,
    base: options.base,
    extensions: options.extensions,
    mdx,
    ssg: {
      enabled: false,
      extension: ".html",
      clean: false,
      bare: false,
      generateOgImage: false,
      lastUpdated: false,
      pagination: false,
      breadcrumbs: false,
      jsonLd: false,
      readerChrome: false,
      localeSwitcher: false,
      a11y: false,
      pageChrome: false,
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
    autolinks: options.autolinks,
    highlight: false,
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
    embeds: options.embeds,
    i18n: false,
  } as unknown as Parameters<typeof baseTransformMarkdown>[2] & {
    codeAnnotations?: ResolvedSvelteOptions["codeAnnotations"];
  };

  if (mdx) {
    const documentExpressions = options.mdxDocumentProps
      ? prepareMdxDocumentExpressions(markdownContent, id)
      : { content: markdownContent, expressions: [] };
    const transformed = await baseTransformMarkdown(documentExpressions.content, id, baseOptions);
    const discovered = await discoverDocumentMdxIslands({
      source: markdownContent,
      html: transformed.html,
      components,
      imports: transformed.imports,
      documentPath: id,
      contentRoot: resolveContentRootPath({ srcDir: options.srcDir, root: options.root }),
      srcDir: options.srcDir,
    });
    if (options.mdxDocumentProps) {
      return compileSvelteResult(
        generateMdxDocumentPropsSvelteModule(
          transformed.html,
          discovered.usedComponents,
          frontmatter,
          options,
          id,
          discovered.localBindings,
          documentExpressions.expressions,
        ),
        id,
        discovered.usedComponents,
        frontmatter,
        options.ssr,
      );
    }
    const html = options.renderIsland
      ? await applyIslandSsrHtml(
          transformed.html,
          options.renderIsland,
          id,
          discovered.usedComponents,
        )
      : transformed.html;
    return compileSvelteResult(
      generateSvelteModule(
        html,
        discovered.usedComponents,
        discovered.usedComponents,
        frontmatter,
        options,
        id,
        discovered.localBindings,
      ),
      id,
      discovered.usedComponents,
      frontmatter,
      options.ssr,
    );
  }

  const usedComponents: string[] = [];
  const islands: ComponentIsland[] = [];
  let islandIndex = 0;

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

  const transformed = await baseTransformMarkdown(processedContent, id, baseOptions);
  const htmlWithIslands = injectIslandMarkers(transformed.html, islands);
  return compileSvelteResult(
    generateSvelteModule(htmlWithIslands, usedComponents, islands, frontmatter, options, id),
    id,
    usedComponents,
    frontmatter,
    options.ssr,
  );
}

function compileSvelteResult(
  svelteCode: string,
  id: string,
  usedComponents: string[],
  frontmatter: Record<string, unknown>,
  ssr = false,
): SvelteTransformResult {
  const compiled = compile(svelteCode, {
    filename: id,
    generate: ssr ? "server" : "client",
    runes: true,
  });

  return {
    code: `${compiled.js.code}\nexport const frontmatter = ${JSON.stringify(frontmatter)};`,
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

interface MdxDocumentExpression {
  marker: string;
  expression: string;
  path: string[];
}

interface PreparedMdxDocumentExpressions {
  content: string;
  expressions: MdxDocumentExpression[];
}

interface MdxIslandRange {
  name: string;
  tag: string;
  openStart: number;
  openEnd: number;
  innerStart: number;
  contentStart: number;
  closeStart: number;
  closeEnd: number;
  propsAttr?: string;
  script?: string;
}

interface MdxIslandPayload {
  props: Record<string, unknown>;
  expressions: Record<string, string>;
  spreads: string[];
}

interface MdxTemplateContext {
  html: string;
  filePath: string;
  usedComponents: Set<string>;
  expressionsByMarker: Map<string, MdxDocumentExpression>;
  islandRanges: MdxIslandRange[];
}

function prepareMdxDocumentExpressions(
  content: string,
  filePath: string,
): PreparedMdxDocumentExpressions {
  const skipRanges = mergeRanges([
    ...collectFenceRanges(content),
    ...collectInlineCodeRanges(content),
    ...collectMdxEsmLineRanges(content),
  ]);
  const expressions: MdxDocumentExpression[] = [];
  let output = "";
  let cursor = 0;
  let rangeIndex = 0;
  let inTag = false;
  let quote: string | null = null;

  while (cursor < content.length) {
    const range = skipRanges[rangeIndex];
    if (range && cursor >= range.end) {
      rangeIndex += 1;
      continue;
    }
    if (range && cursor === range.start) {
      output += content.slice(range.start, range.end);
      cursor = range.end;
      continue;
    }

    const char = content[cursor]!;
    if (inTag) {
      output += char;
      if (quote) {
        if (char === quote && content[cursor - 1] !== "\\") {
          quote = null;
        }
      } else if (char === '"' || char === "'") {
        quote = char;
      } else if (char === ">") {
        inTag = false;
      }
      cursor += 1;
      continue;
    }

    if (char === "<" && startsHtmlLikeTag(content, cursor)) {
      inTag = true;
      output += char;
      cursor += 1;
      continue;
    }

    if (char === "{" && content[cursor - 1] !== "\\") {
      const end = findMdxExpressionEnd(content, cursor + 1);
      if (end !== -1) {
        const expression = content.slice(cursor + 1, end).trim();
        const path = parseDocumentPropPath(expression);
        if (!path) {
          throw new Error(
            `[ox-content-svelte] Unsupported MDX document prop expression "{${expression}}" in ${filePath}. Only identifiers and dotted property paths are supported.`,
          );
        }
        const marker = `${DOCUMENT_PROP_MARKER_PREFIX}${expressions.length}${DOCUMENT_PROP_MARKER_SUFFIX}`;
        expressions.push({ marker, expression, path });
        output += marker;
        cursor = end + 1;
        continue;
      }
    }

    output += char;
    cursor += 1;
  }

  return { content: output, expressions };
}

function collectInlineCodeRanges(content: string): Range[] {
  const ranges: Range[] = [];
  const fenceRanges = collectFenceRanges(content);
  let lineStart = 0;

  while (lineStart < content.length) {
    const lineEnd = content.indexOf("\n", lineStart);
    const end = lineEnd === -1 ? content.length : lineEnd;
    if (!isInRanges(lineStart, end, fenceRanges)) {
      let cursor = lineStart;
      while (cursor < end) {
        const marker = matchBacktickRun(content, cursor);
        if (!marker) {
          cursor += 1;
          continue;
        }
        const close = content.indexOf(marker, cursor + marker.length);
        if (close === -1 || close >= end) {
          cursor += marker.length;
          continue;
        }
        ranges.push({ start: cursor, end: close + marker.length });
        cursor = close + marker.length;
      }
    }
    lineStart = lineEnd === -1 ? content.length : lineEnd + 1;
  }

  return ranges;
}

function collectMdxEsmLineRanges(content: string): Range[] {
  const ranges: Range[] = [];
  const fenceRanges = collectFenceRanges(content);
  let lineStart = 0;

  while (lineStart < content.length) {
    const lineEnd = content.indexOf("\n", lineStart);
    const end = lineEnd === -1 ? content.length : lineEnd + 1;
    const contentEnd = lineEnd === -1 ? content.length : lineEnd;
    if (!isInRanges(lineStart, contentEnd, fenceRanges)) {
      const line = content.slice(lineStart, contentEnd).trimStart();
      if (line.startsWith("import ") || line.startsWith("export ")) {
        ranges.push({ start: lineStart, end });
      }
    }
    lineStart = lineEnd === -1 ? content.length : lineEnd + 1;
  }

  return ranges;
}

function mergeRanges(ranges: Range[]): Range[] {
  const sorted = ranges
    .filter((range) => range.end > range.start)
    .sort((left, right) => left.start - right.start || left.end - right.end);
  const merged: Range[] = [];

  for (const range of sorted) {
    const previous = merged.at(-1);
    if (previous && range.start <= previous.end) {
      previous.end = Math.max(previous.end, range.end);
    } else {
      merged.push({ ...range });
    }
  }

  return merged;
}

function matchBacktickRun(content: string, index: number): string | null {
  if (content[index] !== "`") return null;
  let end = index + 1;
  while (content[end] === "`") {
    end += 1;
  }
  return content.slice(index, end);
}

function startsHtmlLikeTag(content: string, index: number): boolean {
  const next = content[index + 1];
  return next === "/" || next === "!" || next === "?" || /[A-Za-z]/.test(next ?? "");
}

function findMdxExpressionEnd(content: string, start: number): number {
  let depth = 1;
  let quote: string | null = null;
  let escaped = false;

  for (let index = start; index < content.length; index += 1) {
    const char = content[index]!;
    if (quote) {
      if (escaped) {
        escaped = false;
      } else if (char === "\\") {
        escaped = true;
      } else if (char === quote) {
        quote = null;
      }
      continue;
    }

    if (char === '"' || char === "'" || char === "`") {
      quote = char;
      continue;
    }
    if (char === "{") {
      depth += 1;
      continue;
    }
    if (char === "}") {
      depth -= 1;
      if (depth === 0) return index;
    }
  }

  return -1;
}

function parseDocumentPropPath(expression: string): string[] | null {
  if (
    !/^[A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*$/.test(expression) ||
    RESERVED_DOCUMENT_PROP_WORDS.has(expression)
  ) {
    return null;
  }
  return expression.split(".");
}

const RESERVED_DOCUMENT_PROP_WORDS = new Set([
  "false",
  "Infinity",
  "NaN",
  "null",
  "this",
  "true",
  "undefined",
]);

function generateMdxDocumentPropsSvelteModule(
  html: string,
  usedComponents: string[],
  frontmatter: Record<string, unknown>,
  options: ResolvedSvelteOptions & { root?: string },
  id: string,
  localBindings: ReadonlyMap<string, ResolvedDocumentComponentImport>,
  documentExpressions: readonly MdxDocumentExpression[],
): string {
  const filePathLiteral = JSON.stringify(id);
  const imports = renderIslandComponentImports(usedComponents, {
    globalComponents: options.components,
    localBindings,
    documentPath: id,
    root: options.root,
  });
  const template = renderMdxDocumentTemplate(html, usedComponents, id, documentExpressions);

  return `
<script>
  ${imports}

  const frontmatter = ${JSON.stringify(frontmatter)};
  export { frontmatter };

  let __ox_mdx_props = $props();

  function __ox_mdx_document_prop(props, path, expression) {
    const propName = path.join(".");
    let value = props;
    for (const segment of path) {
      if (
        value == null ||
        (typeof value !== "object" && typeof value !== "function") ||
        !(segment in Object(value))
      ) {
        throw new Error('[ox-content-svelte] Missing MDX document prop "' + propName + '" in ' + ${filePathLiteral} + ' for expression {' + expression + '}.');
      }
      value = value[segment];
    }
    if (value === undefined) {
      throw new Error('[ox-content-svelte] Missing MDX document prop "' + propName + '" in ' + ${filePathLiteral} + ' for expression {' + expression + '}.');
    }
    return value;
  }
</script>

<div class="ox-content">${template}</div>

<style>
  .ox-content {
    line-height: 1.6;
  }
</style>
`;
}

function renderMdxDocumentTemplate(
  html: string,
  usedComponents: string[],
  filePath: string,
  documentExpressions: readonly MdxDocumentExpression[],
): string {
  const context: MdxTemplateContext = {
    html,
    filePath,
    usedComponents: new Set(usedComponents),
    expressionsByMarker: new Map(
      documentExpressions.map((expression) => [expression.marker, expression] as const),
    ),
    islandRanges: findMdxIslandRanges(html),
  };
  return renderHtmlRange(context, 0, html.length);
}

function renderHtmlRange(context: MdxTemplateContext, start: number, end: number): string {
  let output = "";
  let cursor = start;

  while (cursor < end) {
    const island = findNextIslandRange(context, cursor, end);
    if (!island) {
      output += renderRawHtmlTemplate(context.html.slice(cursor, end), context.expressionsByMarker);
      break;
    }

    output += renderRawHtmlTemplate(
      context.html.slice(cursor, island.openStart),
      context.expressionsByMarker,
    );
    output += renderMdxIslandTemplate(context, island);
    cursor = island.closeEnd;
  }

  return output;
}

function findNextIslandRange(
  context: MdxTemplateContext,
  cursor: number,
  end: number,
): MdxIslandRange | null {
  for (const island of context.islandRanges) {
    if (island.openStart < cursor || island.closeEnd > end) {
      continue;
    }
    if (context.usedComponents.has(island.name)) {
      return island;
    }
  }
  return null;
}

function renderRawHtmlTemplate(
  html: string,
  expressionsByMarker: ReadonlyMap<string, MdxDocumentExpression>,
): string {
  if (!html) return "";
  let output = "";
  let cursor = 0;

  while (cursor < html.length) {
    const next = findNextDocumentExpressionMarker(html, cursor, expressionsByMarker);
    if (!next) {
      output += renderRawHtmlBlock(html.slice(cursor));
      break;
    }
    output += renderRawHtmlBlock(html.slice(cursor, next.index));
    output += renderDocumentExpression(next.expression);
    cursor = next.index + next.expression.marker.length;
  }

  return output;
}

function findNextDocumentExpressionMarker(
  html: string,
  start: number,
  expressionsByMarker: ReadonlyMap<string, MdxDocumentExpression>,
): { index: number; expression: MdxDocumentExpression } | null {
  let nextIndex = -1;
  let nextExpression: MdxDocumentExpression | undefined;

  for (const expression of expressionsByMarker.values()) {
    const index = html.indexOf(expression.marker, start);
    if (index !== -1 && (nextIndex === -1 || index < nextIndex)) {
      nextIndex = index;
      nextExpression = expression;
    }
  }

  return nextExpression ? { index: nextIndex, expression: nextExpression } : null;
}

function renderRawHtmlBlock(html: string): string {
  return html ? `{@html ${JSON.stringify(html).replaceAll("</script", "<\\/script")}}` : "";
}

function renderDocumentExpression(expression: MdxDocumentExpression): string {
  return `{${documentPropResolverExpression(expression.path, expression.expression)}}`;
}

function renderMdxIslandTemplate(context: MdxTemplateContext, island: MdxIslandRange): string {
  assertSvelteComponentName(island.name, context.filePath);
  const attrs = renderMdxIslandAttributes(readMdxIslandPayload(island), context.filePath);
  const children = renderHtmlRange(context, island.contentStart, island.closeStart);
  return children
    ? `<${island.name}${attrs}>${children}</${island.name}>`
    : `<${island.name}${attrs} />`;
}

function renderMdxIslandAttributes(payload: MdxIslandPayload, filePath: string): string {
  const attrs: string[] = [];

  for (const spread of payload.spreads) {
    const expression = spread.trim().startsWith("...")
      ? spread.trim().slice(3).trim()
      : spread.trim();
    const path = parseDocumentPropPath(expression);
    if (!path) {
      throw new Error(
        `[ox-content-svelte] Unsupported MDX document prop spread "{${spread}}" in ${filePath}. Only identifiers and dotted property paths are supported.`,
      );
    }
    attrs.push(`{...${documentPropResolverExpression(path, expression)}}`);
  }

  for (const [name, value] of Object.entries(payload.props)) {
    assertSvelteAttributeName(name, filePath);
    attrs.push(`${name}={${renderSvelteLiteral(value)}}`);
  }

  for (const [name, expression] of Object.entries(payload.expressions)) {
    assertSvelteAttributeName(name, filePath);
    const path = parseDocumentPropPath(expression.trim());
    if (!path) {
      throw new Error(
        `[ox-content-svelte] Unsupported MDX document prop expression "{${expression}}" for prop "${name}" in ${filePath}. Only identifiers and dotted property paths are supported.`,
      );
    }
    attrs.push(`${name}={${documentPropResolverExpression(path, expression.trim())}}`);
  }

  return attrs.length > 0 ? ` ${attrs.join(" ")}` : "";
}

function documentPropResolverExpression(path: readonly string[], expression: string): string {
  return `__ox_mdx_document_prop(__ox_mdx_props, ${JSON.stringify(path)}, ${JSON.stringify(expression)})`;
}

function renderSvelteLiteral(value: unknown): string {
  const literal = JSON.stringify(value);
  return literal === undefined ? "undefined" : literal.replaceAll("</script", "<\\/script");
}

function findMdxIslandRanges(html: string): MdxIslandRange[] {
  const ranges: MdxIslandRange[] = [];
  const openRe = /<(div|span)\b([^>]*\bdata-ox-island="([^"]+)"[^>]*)>/gi;
  let match: RegExpExecArray | null;

  while ((match = openRe.exec(html)) !== null) {
    const tag = match[1]!;
    const name = decodeHtmlAttr(match[3] ?? "");
    if (!name) continue;
    const openStart = match.index;
    const openEnd = match.index + match[0].length;
    const closeStart = findMatchingClose(html, openEnd, tag);
    const closeEnd = closeStart < html.length ? closeStart + tag.length + 3 : html.length;
    const inner = html.slice(openEnd, closeStart);
    const scriptMatch = inner.match(PAYLOAD_SCRIPT);
    const script = scriptMatch?.[0];
    ranges.push({
      name,
      tag,
      openStart,
      openEnd,
      innerStart: openEnd,
      contentStart: openEnd + (script?.length ?? 0),
      closeStart,
      closeEnd,
      propsAttr: matchAttr(match[2] ?? "", "data-ox-props"),
      script,
    });
  }

  return ranges.sort((left, right) => left.openStart - right.openStart);
}

function findMatchingClose(html: string, from: number, tag: string): number {
  const openNeedle = `<${tag}`;
  const closeNeedle = `</${tag}>`;
  let depth = 1;
  let cursor = from;

  while (cursor < html.length) {
    const nextOpen = indexOfTagOpen(html, openNeedle, cursor);
    const nextClose = html.indexOf(closeNeedle, cursor);
    if (nextClose === -1) return html.length;
    if (nextOpen !== -1 && nextOpen < nextClose) {
      depth += 1;
      cursor = nextOpen + openNeedle.length;
    } else {
      depth -= 1;
      if (depth === 0) return nextClose;
      cursor = nextClose + closeNeedle.length;
    }
  }

  return html.length;
}

function indexOfTagOpen(html: string, openNeedle: string, from: number): number {
  let cursor = from;
  while (cursor < html.length) {
    const index = html.indexOf(openNeedle, cursor);
    if (index === -1) return -1;
    const next = html[index + openNeedle.length];
    if (next === " " || next === ">" || next === "\t" || next === "\n" || next === "/") {
      return index;
    }
    cursor = index + openNeedle.length;
  }
  return -1;
}

function matchAttr(attrs: string, name: string): string | undefined {
  const match = new RegExp(`\\b${name}="([^"]*)"`, "i").exec(attrs);
  return match?.[1] === undefined ? undefined : decodeHtmlAttr(match[1]);
}

function readMdxIslandPayload(island: MdxIslandRange): MdxIslandPayload {
  const fromAttr = island.propsAttr ? tryParseJson(island.propsAttr) : undefined;
  const fromScript = island.script
    ? tryParseJson(
        island.script.match(/<script type="application\/json">([\s\S]*?)<\/script>/i)?.[1] ?? "",
      )
    : undefined;
  return normalizeMdxIslandPayload(fromAttr ?? fromScript ?? {});
}

function normalizeMdxIslandPayload(parsed: unknown): MdxIslandPayload {
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    return { props: {}, expressions: {}, spreads: [] };
  }

  const record = parsed as Record<string, unknown>;
  const keys = Object.keys(record);
  if (keys.length > 0 && keys.every((key) => RUST_PAYLOAD_KEYS.has(key))) {
    return {
      props: toRecord(record.props),
      expressions: toStringRecord(record.expressions),
      spreads: toStringArray(record.spreads),
    };
  }

  return { props: record, expressions: {}, spreads: [] };
}

function toRecord(value: unknown): Record<string, unknown> {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Record<string, unknown>)
    : {};
}

function toStringRecord(value: unknown): Record<string, string> {
  const record = toRecord(value);
  const output: Record<string, string> = {};
  for (const [key, entry] of Object.entries(record)) {
    if (typeof entry === "string") {
      output[key] = entry;
    }
  }
  return output;
}

function toStringArray(value: unknown): string[] {
  return Array.isArray(value)
    ? value.filter((entry): entry is string => typeof entry === "string")
    : [];
}

function tryParseJson(value: string): unknown {
  try {
    return JSON.parse(value);
  } catch {
    return undefined;
  }
}

function decodeHtmlAttr(value: string): string {
  return value
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&");
}

function assertSvelteComponentName(name: string, filePath: string): void {
  if (!/^[A-Z][A-Za-z0-9_$]*$/.test(name)) {
    throw new Error(
      `[ox-content-svelte] Unsupported MDX component name "${name}" in ${filePath} for mdxDocumentProps. Only simple Svelte component identifiers are supported.`,
    );
  }
}

function assertSvelteAttributeName(name: string, filePath: string): void {
  if (!/^[A-Za-z_$][\w$-]*$/.test(name)) {
    throw new Error(
      `[ox-content-svelte] Unsupported MDX component prop name "${name}" in ${filePath}.`,
    );
  }
}

function generateSvelteModule(
  content: string,
  usedComponents: string[],
  _islands: ComponentIsland[] | string[],
  frontmatter: Record<string, unknown>,
  options: ResolvedSvelteOptions & { root?: string },
  id: string,
  localBindings?: ReadonlyMap<string, ResolvedDocumentComponentImport>,
): string {
  // Rust island payloads include `</script>`; that must not close this SFC block.
  const rawHtmlLiteral = JSON.stringify(content).replaceAll("</script", "<\\/script");

  const imports = renderIslandComponentImports(usedComponents, {
    globalComponents: options.components,
    localBindings,
    documentPath: id,
    root: options.root,
  });

  // If no registered islands, generate simpler code without island runtime
  if (usedComponents.length === 0) {
    return `
<script>
  const frontmatter = ${JSON.stringify(frontmatter)};
  const rawHtml = ${rawHtmlLiteral};

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
  import { createRawSnippet, hydrate, mount, onMount, unmount } from 'svelte';
  import { initIslands, readIslandSlotHtml } from '@ox-content/islands';
  ${imports}

  const frontmatter = ${JSON.stringify(frontmatter)};
  const rawHtml = ${rawHtmlLiteral};
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

      const islandContent = readIslandSlotHtml(element);
      const componentProps = { ...props };
      if (islandContent) {
        componentProps.children = createRawSnippet(() => ({
          render: () => \`<div>\${islandContent}</div>\`,
        }));
      }

      const attach = element.dataset.oxSsr === 'true' ? hydrate : mount;
      const instance = attach(Component, { target: element, props: componentProps });
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
