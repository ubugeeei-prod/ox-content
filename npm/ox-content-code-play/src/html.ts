import { resolveLanguage } from "./catalog";
import { DEFAULT_ENDPOINTS, DEFAULT_VIEWERS } from "./config";
import { decodeHtml, escapeAttribute } from "./escape";
import { payloadTypecheckEnabled } from "./payload-factory";
import type { PlaygroundEndpoints, PlayPayload } from "./types";

const COMMENT_PATTERN = /<!--ox-code-play:([A-Za-z0-9+/=]+)-->\s*(<pre\b[\s\S]*?<\/pre>)/gi;
const CODEPLAY_TAG_PATTERN = /<CodePlay\b([^>]*)>([\s\S]*?)<\/CodePlay>/gi;
const PRE_PATTERN = /<pre\b[^>]*>\s*<code\b([^>]*)>([\s\S]*?)<\/code>\s*<\/pre>/gi;

export interface HtmlEnhanceOptions {
  scriptSrc?: string;
  decodePayload: (value: string) => PlayPayload;
  encodePayload: (payload: PlayPayload) => string;
  matchFences?: Array<{ language: string; code: string; payload: string }>;
  endpoints?: PlaygroundEndpoints;
}

export function enhancePlayHtml(html: string, options: HtmlEnhanceOptions): string {
  let next = wrapCommentedBlocks(html, options);
  next = upgradeCodePlayTags(next, options);
  if (options.matchFences?.length) {
    next = wrapMatchingFences(next, options.matchFences);
  }
  if (
    options.scriptSrc &&
    next.includes("data-ox-code-play") &&
    !next.includes(options.scriptSrc)
  ) {
    next += `\n<script type="module" src="${escapeAttribute(options.scriptSrc)}"></script>\n`;
  }
  return next;
}

export function enhanceGeneratedModule(source: string, options: HtmlEnhanceOptions): string {
  const marker = "export const html = ";
  const start = source.indexOf(marker);
  if (start === -1) {
    return source;
  }
  const jsonStart = start + marker.length;
  if (source[jsonStart] !== '"') {
    return source;
  }
  const parsed = readJsonString(source, jsonStart);
  if (!parsed) {
    return source;
  }
  const enhanced = enhancePlayHtml(parsed.value, options);
  return source.slice(0, jsonStart) + JSON.stringify(enhanced) + source.slice(parsed.end);
}

function wrapCommentedBlocks(html: string, _options: HtmlEnhanceOptions): string {
  return html.replace(COMMENT_PATTERN, (_all, payload: string, pre: string) => {
    return wrapWidget(payload, pre);
  });
}

function upgradeCodePlayTags(html: string, options: HtmlEnhanceOptions): string {
  return html.replace(CODEPLAY_TAG_PATTERN, (all, attrs: string, body: string) => {
    if (/\bdata-ox-code-play=/.test(all)) {
      return all;
    }
    const language = readAttr(attrs, "lang") ?? readAttr(attrs, "language") ?? "text";
    const code = decodeHtml(body).replace(/^\n/, "").replace(/\n$/, "");
    const definition = resolveLanguage(language);
    const endpoints = options.endpoints ?? DEFAULT_ENDPOINTS;
    const payload = options.encodePayload({
      language: definition?.id ?? language,
      code,
      capabilities: {
        execute: true,
        typecheck: payloadTypecheckEnabled(
          /\btypecheck\b/i.test(attrs),
          undefined,
          definition,
          endpoints,
        ),
      },
      config: {},
      viewers: { ...DEFAULT_VIEWERS },
      ui: "default",
      timeoutMs: 10_000,
      endpoints,
    });
    return wrapWidget(
      payload,
      `<pre><code class="language-${escapeAttribute(language)}">${body}</code></pre>`,
    );
  });
}

function wrapMatchingFences(
  html: string,
  matches: Array<{ language: string; code: string; payload: string }>,
): string {
  const unused = matches.map((item) => ({ ...item, used: false }));
  return html.replace(PRE_PATTERN, (all, attrs: string, body: string) => {
    const className = readAttr(attrs, "class") ?? "";
    const language = className
      .split(/\s+/)
      .find((token) => token.startsWith("language-"))
      ?.slice("language-".length);
    const code = normalizeCode(decodeHtml(stripTags(body)));
    const match = unused.find(
      (item) =>
        !item.used && aliasesEqual(item.language, language) && normalizeCode(item.code) === code,
    );
    if (!match) {
      return all;
    }
    match.used = true;
    return wrapWidget(match.payload, all);
  });
}

function wrapWidget(payload: string, inner: string): string {
  return `<ox-code-play data-ox-code-play="${escapeAttribute(payload)}">${inner}</ox-code-play>`;
}

function readJsonString(source: string, start: number): { value: string; end: number } | undefined {
  try {
    const raw = sliceJsonString(source, start);
    return { value: JSON.parse(raw) as string, end: start + raw.length };
  } catch {
    return undefined;
  }
}

function sliceJsonString(source: string, start: number): string {
  let end = start + 1;
  let escaped = false;
  while (end < source.length) {
    const char = source[end];
    if (escaped) {
      escaped = false;
    } else if (char === "\\") {
      escaped = true;
    } else if (char === '"') {
      return source.slice(start, end + 1);
    }
    end += 1;
  }
  throw new Error("Unterminated HTML JSON string.");
}

function readAttr(attrs: string, name: string): string | undefined {
  const match = new RegExp(`(?:^|\\s)${name}\\s*=\\s*"([^"]+)"`, "i").exec(attrs);
  return match?.[1];
}

function stripTags(value: string): string {
  return value.replace(/<[^>]+>/g, "");
}

function normalizeCode(value: string): string {
  return value.replace(/\r\n/g, "\n").replace(/\n$/, "");
}

function aliasesEqual(left: string | undefined, right: string | undefined): boolean {
  if (!left || !right) {
    return false;
  }
  return left.toLowerCase() === right.toLowerCase();
}
