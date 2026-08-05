/**
 * Markdown-side scanning shared by the Solid transform: locating component
 * tags, keeping fenced samples literal, and round-tripping islands through the
 * rendered HTML as placeholder markers.
 */

import type { ComponentIsland, ComponentsMap } from "./types";

const COMPONENT_REGEX = /<([A-Z][a-zA-Z0-9]*)\s*([^>]*?)\s*(?:\/>|>([\s\S]*?)<\/\1>)/g;
const PROP_REGEX = /([a-zA-Z0-9-]+)(?:=(?:"([^"]*)"|'([^']*)'|{([^}]*)}|\[([^\]]*)\]))?/g;

const ISLAND_MARKER_PREFIX = "OXCONTENT-ISLAND-";
const ISLAND_MARKER_SUFFIX = "-PLACEHOLDER";

interface Range {
  start: number;
  end: number;
}

export interface ScannedMarkdown {
  /** Markdown with component tags replaced by island placeholder markers. */
  content: string;
  islands: ComponentIsland[];
  usedComponents: string[];
}

/**
 * Replaces every registered component tag with a placeholder marker that
 * survives Markdown rendering, so the island can be re-attached to the produced
 * HTML afterwards. Tags inside fenced code blocks are left as literal text.
 */
export function scanComponents(markdown: string, components: ComponentsMap): ScannedMarkdown {
  const usedComponents: string[] = [];
  const islands: ComponentIsland[] = [];
  const fenceRanges = collectFenceRanges(markdown);

  let islandIndex = 0;
  let content = "";
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  COMPONENT_REGEX.lastIndex = 0;
  while ((match = COMPONENT_REGEX.exec(markdown)) !== null) {
    const [fullMatch, componentName, propsString, rawIslandContent] = match;
    const matchStart = match.index;
    const matchEnd = matchStart + fullMatch.length;

    if (
      !Object.prototype.hasOwnProperty.call(components, componentName) ||
      isInRanges(matchStart, matchEnd, fenceRanges)
    ) {
      content += markdown.slice(lastIndex, matchEnd);
      lastIndex = matchEnd;
      continue;
    }

    if (!usedComponents.includes(componentName)) {
      usedComponents.push(componentName);
    }

    const islandId = `ox-island-${islandIndex++}`;
    islands.push({
      name: componentName,
      props: parseProps(propsString),
      position: matchStart,
      id: islandId,
      content: typeof rawIslandContent === "string" ? rawIslandContent.trim() : undefined,
    });

    content += markdown.slice(lastIndex, matchStart) + createIslandMarker(islandId);
    lastIndex = matchEnd;
  }

  content += markdown.slice(lastIndex);

  return { content, islands, usedComponents };
}

/** Swaps the placeholder markers in rendered HTML for island mount points. */
export function injectIslandMarkers(html: string, islands: ComponentIsland[]): string {
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

export function extractFrontmatter(content: string): {
  content: string;
  frontmatter: Record<string, unknown>;
} {
  const frontmatterRegex = /^---\n([\s\S]*?)\n---\n/;
  const match = frontmatterRegex.exec(content);

  if (!match) {
    return { content, frontmatter: {} };
  }

  const frontmatter: Record<string, unknown> = {};

  for (const line of match[1].split("\n")) {
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

function parseProps(propsString: string): Record<string, unknown> {
  const props: Record<string, unknown> = {};
  if (!propsString) return props;

  PROP_REGEX.lastIndex = 0;
  let match: RegExpExecArray | null;
  while ((match = PROP_REGEX.exec(propsString)) !== null) {
    const [, name, doubleQuoted, singleQuoted, braceValue, bracketValue] = match;
    if (!name) continue;

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

  return props;
}
