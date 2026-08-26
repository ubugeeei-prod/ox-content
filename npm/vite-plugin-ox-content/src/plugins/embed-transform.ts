/**
 * Reserve first-party embed components so MDX island lowering cannot swallow them.
 *
 * Rust still emits `data-ox-island` for every PascalCase tag. This rewrite turns
 * reserved built-in names back into HTML tags so `transformBuiltinEmbeds` can
 * run. A document-local import of the same name keeps the island.
 */

const PAYLOAD_SCRIPT = /^\s*<script type="application\/json">[\s\S]*?<\/script>/;
const RUST_PAYLOAD_KEYS = new Set(["props", "expressions", "spreads"]);

/**
 * Canonical PascalCase names for first-party embed / media components.
 * Keep in sync with `SELF_CLOSING_EMBED_TAG` in `./index.ts`.
 */
export const RESERVED_BUILTIN_COMPONENTS = [
  "GitHub",
  "OgCard",
  "Tweet",
  "XPost",
  "Bluesky",
  "Spotify",
  "AppleMusic",
  "SpeakerDeck",
  "Audio",
  "Video",
  "StackBlitz",
  "WebContainer",
  "YouTube",
  "NotByAI",
] as const;

const RESERVED_BUILTIN_COMPONENT_SET = new Set<string>(RESERVED_BUILTIN_COMPONENTS);

export type ReservedBuiltinComponent = (typeof RESERVED_BUILTIN_COMPONENTS)[number];

export function isReservedBuiltinComponent(name: string): boolean {
  return RESERVED_BUILTIN_COMPONENT_SET.has(name);
}

export function documentLocalComponentNames(
  imports: readonly { specifiers: readonly { local: string; kind: string }[] }[],
): Set<string> {
  const names = new Set<string>();
  for (const statement of imports) {
    for (const spec of statement.specifiers) {
      if (spec.kind !== "namespace") {
        names.add(spec.local);
      }
    }
  }
  return names;
}

/**
 * Replace reserved built-in islands with the original HTML tags so embed
 * transforms can see them. Document-local bindings of the same name stay islands.
 */
export function restoreReservedBuiltinIslands(html: string, localNames?: Iterable<string>): string {
  if (!html.includes("data-ox-island=")) {
    return html;
  }

  const local = localNames ? new Set(localNames) : new Set<string>();
  const islands = findIslandRanges(html);
  let output = html;

  for (const island of islands.toReversed()) {
    if (!isReservedBuiltinComponent(island.name) || local.has(island.name)) {
      continue;
    }

    const inner = output.slice(island.innerStart, island.closeStart);
    const scriptMatch = inner.match(PAYLOAD_SCRIPT);
    const children = unwrapSingleParagraph(
      (scriptMatch ? inner.slice(scriptMatch[0].length) : inner).trim(),
    );
    const props = parseLiteralProps(island.propsAttr, scriptMatch?.[0] ?? "");
    const rebuilt = `<${island.name}${stringifyLiteralAttrs(props)}>${children}</${island.name}>`;
    output = output.slice(0, island.openStart) + rebuilt + output.slice(island.closeEnd);
  }

  return output;
}

export function filterReservedBuiltinComponentNames(
  names: readonly string[],
  localNames?: Iterable<string>,
): string[] {
  const local = localNames ? new Set(localNames) : new Set<string>();
  return names.filter((name) => !isReservedBuiltinComponent(name) || local.has(name));
}

interface IslandRange {
  name: string;
  openStart: number;
  innerStart: number;
  closeStart: number;
  closeEnd: number;
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
    const closeEnd = closeStart < html.length ? closeStart + `</${tag}>`.length : html.length;
    ranges.push({
      name,
      openStart: match.index,
      innerStart,
      closeStart,
      closeEnd,
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

function parseLiteralProps(propsAttr: string | undefined, script: string): Record<string, unknown> {
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

function stringifyLiteralAttrs(props: Record<string, unknown>): string {
  let attrs = "";
  for (const [name, value] of Object.entries(props)) {
    if (!isSafeAttrName(name) || value == null || value === false) {
      continue;
    }
    if (value === true) {
      attrs += ` ${name}`;
      continue;
    }
    if (typeof value === "string" || typeof value === "number" || typeof value === "bigint") {
      attrs += ` ${name}="${escapeAttribute(String(value))}"`;
    }
  }
  return attrs;
}

function isSafeAttrName(name: string): boolean {
  return /^[A-Za-z_:][\w:.-]*$/.test(name);
}

function unwrapSingleParagraph(html: string): string {
  const wrapped = html.match(/^<p>([\s\S]*)<\/p>$/i);
  return wrapped?.[1] ?? html;
}

function escapeAttribute(value: string): string {
  return value
    .replaceAll("&", "&amp;")
    .replaceAll('"', "&quot;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;");
}

function decodeHtmlAttr(value: string): string {
  return value
    .replaceAll("&quot;", '"')
    .replaceAll("&#39;", "'")
    .replaceAll("&lt;", "<")
    .replaceAll("&gt;", ">")
    .replaceAll("&amp;", "&");
}
