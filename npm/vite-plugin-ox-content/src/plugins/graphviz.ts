import {
  isResolvedGraphvizOptions,
  rendererIdentityOrFallback,
  renderGraphvizOrOriginal,
  resolveGraphvizOptions,
  type GraphvizOptions,
  type GraphvizRenderBlock,
  type ResolvedGraphvizOptions,
} from "./graphviz-renderer";

export {
  clearGraphvizCache,
  resolveGraphvizOptions,
  type GraphvizFailureMode,
  type GraphvizOptions,
  type ResolvedGraphvizOptions,
} from "./graphviz-renderer";

interface GraphvizFence extends GraphvizRenderBlock {
  start: number;
  end: number;
}

const GRAPHVIZ_HTML_BLOCK =
  /<pre><code class="[^"]*\blanguage-(?:dot|graphviz)\b[^"]*">([\s\S]*?)<\/code><\/pre>/gi;

export async function prepareGraphvizFences(
  markdown: string,
  options: boolean | GraphvizOptions | ResolvedGraphvizOptions,
): Promise<{ markdown: string; replacements: Map<string, string> }> {
  const resolved = isResolvedGraphvizOptions(options) ? options : resolveGraphvizOptions(options);
  const replacements = new Map<string, string>();
  if (!resolved || (!markdown.includes("```") && !markdown.includes("~~~"))) {
    return { markdown, replacements };
  }

  const fences = collectGraphvizFences(markdown);
  if (fences.length === 0) return { markdown, replacements };

  const identity = await rendererIdentityOrFallback(resolved);
  if (!identity) return { markdown, replacements };

  let output = "";
  let cursor = 0;
  for (const fence of fences) {
    output += markdown.slice(cursor, fence.start);
    const rendered = await renderGraphvizOrOriginal(fence, resolved, identity);
    if (rendered) {
      const placeholder = `<!--ox-graphviz-${replacements.size}-->`;
      replacements.set(placeholder, rendered);
      output += placeholder;
    } else {
      output += markdown.slice(fence.start, fence.end);
    }
    cursor = fence.end;
  }
  return { markdown: output + markdown.slice(cursor), replacements };
}

export function restoreGraphvizPlaceholders(
  html: string,
  replacements: Map<string, string>,
): string {
  if (replacements.size === 0) return html;
  return html.replace(/<!--ox-graphviz-\d+-->/g, (placeholder) => {
    return replacements.get(placeholder) ?? placeholder;
  });
}

export async function transformGraphvizStatic(
  html: string,
  options: boolean | GraphvizOptions | ResolvedGraphvizOptions = true,
): Promise<string> {
  const resolved = isResolvedGraphvizOptions(options) ? options : resolveGraphvizOptions(options);
  if (!resolved || !/\blanguage-(?:dot|graphviz)\b/i.test(html)) return html;

  const blocks = Array.from(html.matchAll(GRAPHVIZ_HTML_BLOCK), (match, occurrence) => ({
    start: match.index ?? 0,
    end: (match.index ?? 0) + match[0].length,
    source: decodeCodeBlock(match[1] ?? ""),
    occurrence,
  }));
  if (blocks.length === 0) return html;

  const identity = await rendererIdentityOrFallback(resolved);
  if (!identity) return html;

  let output = "";
  let cursor = 0;
  for (const block of blocks) {
    output += html.slice(cursor, block.start);
    output +=
      (await renderGraphvizOrOriginal(block, resolved, identity)) ??
      html.slice(block.start, block.end);
    cursor = block.end;
  }
  return output + html.slice(cursor);
}

function collectGraphvizFences(markdown: string): GraphvizFence[] {
  const fences: GraphvizFence[] = [];
  let lineStart = 0;
  let open: { start: number; contentStart: number; char: "`" | "~"; length: number } | null = null;
  while (lineStart < markdown.length) {
    const lineEnd = nextLineEnd(markdown, lineStart);
    const line = trimLineBreak(markdown.slice(lineStart, lineEnd));
    if (!open) {
      const marker = line.match(/^ {0,3}(`{3,}|~{3,})([^\r\n]*)$/);
      if (marker && isGraphvizInfo(marker[2] ?? "")) {
        const fence = marker[1]!;
        open = {
          start: lineStart,
          contentStart: lineEnd,
          char: fence[0] as "`" | "~",
          length: fence.length,
        };
      }
    } else if (isClosingFence(line, open.char, open.length)) {
      fences.push({
        start: open.start,
        end: lineEnd,
        source: markdown.slice(open.contentStart, lineStart),
        occurrence: fences.length,
      });
      open = null;
    }
    lineStart = lineEnd;
  }
  return fences;
}

function isGraphvizInfo(info: string): boolean {
  const lang = info
    .trim()
    .split(/\s+/)[0]
    ?.replace(/^\{\.?/, "")
    .replace(/\}$/, "");
  return lang?.toLowerCase() === "dot" || lang?.toLowerCase() === "graphviz";
}

function isClosingFence(line: string, char: "`" | "~", length: number): boolean {
  return new RegExp(`^ {0,3}${escapeRegExp(char)}{${length},}\\s*$`).test(line);
}

function decodeCodeBlock(value: string): string {
  return value
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"')
    .replace(/&#39;/g, "'");
}

function nextLineEnd(value: string, start: number): number {
  const newline = value.indexOf("\n", start);
  return newline === -1 ? value.length : newline + 1;
}

function trimLineBreak(value: string): string {
  return value.replace(/\r?\n$/, "");
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
