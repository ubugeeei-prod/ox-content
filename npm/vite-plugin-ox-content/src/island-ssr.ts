/**
 * Optional adapter-side island SSR.
 *
 * Framework plugins may supply `renderIsland` to replace island inner HTML at
 * transform time. The hook receives the original slot HTML so adapters can pass
 * children into their SSR runtime. This helper stays framework-neutral and does
 * not import a framework SSR runtime.
 */

export type RenderIslandFn = (
  name: string,
  props: Record<string, unknown>,
  filePath: string,
  slotHtml?: string,
) => string | Promise<string>;

const RUST_PAYLOAD_KEYS = new Set(["props", "expressions", "spreads"]);
const PAYLOAD_SCRIPT = /^\s*<script type="application\/json">[\s\S]*?<\/script>/;

export async function applyIslandSsrHtml(
  html: string,
  renderIsland: RenderIslandFn,
  filePath: string,
  names?: Iterable<string>,
): Promise<string> {
  const allowed = names ? new Set(names) : null;
  const islands = findIslandRanges(html);
  let output = html;

  for (const island of islands.toReversed()) {
    if (allowed && !allowed.has(island.name)) {
      continue;
    }
    const inner = output.slice(island.innerStart, island.closeStart);
    const scriptMatch = inner.match(PAYLOAD_SCRIPT);
    const script = scriptMatch?.[0] ?? "";
    const slotHtml = inner.slice(script.length);
    const props = parseIslandProps(island.propsAttr, script);
    const ssrHtml = await renderIsland(island.name, props, filePath, slotHtml);
    const openTag = markIslandServerRendered(
      output.slice(island.openStart, island.openEnd),
      slotHtml,
    );
    output =
      output.slice(0, island.openStart) +
      openTag +
      script +
      ssrHtml +
      output.slice(island.closeStart);
  }

  return output;
}

interface IslandRange {
  name: string;
  openStart: number;
  openEnd: number;
  innerStart: number;
  closeStart: number;
  propsAttr?: string;
}

function findIslandRanges(html: string): IslandRange[] {
  const ranges: IslandRange[] = [];
  const openRe = /<(div|span)\b([^>]*\bdata-ox-island="([^"]+)"[^>]*)>/gi;
  let match: RegExpExecArray | null;
  while ((match = openRe.exec(html)) !== null) {
    const tag = match[1];
    const name = decodeHtmlAttr(match[3] ?? "");
    if (!tag || !name) continue;
    const innerStart = match.index + match[0].length;
    const closeStart = findMatchingClose(html, innerStart, tag);
    ranges.push({
      name,
      openStart: match.index,
      openEnd: innerStart,
      innerStart,
      closeStart,
      propsAttr: matchAttr(match[2] ?? "", "data-ox-props"),
    });
  }
  return ranges;
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

function markIslandServerRendered(openTag: string, slotHtml: string): string {
  const attrs = hasAttr(openTag, "data-ox-ssr") ? [] : ['data-ox-ssr="true"'];
  if (slotHtml.trim() && !hasAttr(openTag, "data-ox-content")) {
    attrs.push(`data-ox-content='${escapeSingleQuotedAttr(slotHtml)}'`);
  }
  if (attrs.length === 0) return openTag;
  return openTag.replace(/>$/, ` ${attrs.join(" ")}>`);
}

function hasAttr(openTag: string, name: string): boolean {
  return new RegExp(`\\s${name}(?:\\s*=|\\s|>|$)`, "i").test(openTag);
}

function escapeSingleQuotedAttr(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll("'", "&#39;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function parseIslandProps(propsAttr: string | undefined, script: string): Record<string, unknown> {
  const fromAttr = propsAttr ? tryParseJson(propsAttr) : undefined;
  if (fromAttr) return unwrapIslandProps(fromAttr);
  const scriptBody = script.match(/<script type="application\/json">([\s\S]*?)<\/script>/i)?.[1];
  return scriptBody ? unwrapIslandProps(tryParseJson(scriptBody) ?? {}) : {};
}

function tryParseJson(value: string): unknown {
  try {
    return JSON.parse(value);
  } catch {
    return undefined;
  }
}

function unwrapIslandProps(parsed: unknown): Record<string, unknown> {
  if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
    return {};
  }
  const record = parsed as Record<string, unknown>;
  const keys = Object.keys(record);
  if (
    keys.length > 0 &&
    keys.every((key) => RUST_PAYLOAD_KEYS.has(key)) &&
    record.props &&
    typeof record.props === "object" &&
    !Array.isArray(record.props)
  ) {
    return record.props as Record<string, unknown>;
  }
  return record;
}

function decodeHtmlAttr(value: string): string {
  return value
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&");
}
