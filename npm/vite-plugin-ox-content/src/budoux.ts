import type { OxContentOptions } from "./types";
import type { BudouxLanguage, BudouxParser, ResolvedBudouxOptions } from "./budoux-types";

export type {
  BudouxLanguage,
  BudouxOptions,
  BudouxParser,
  ResolvedBudouxOptions,
} from "./budoux-types";

const DEFAULT_LANGUAGE: BudouxLanguage = "ja";
const DEFAULT_SEPARATOR = "\u200b";
const ENTITY = /&(?:#[0-9]+|#x[0-9A-Fa-f]+|[A-Za-z][A-Za-z0-9]+);/g;
const PROTECTED_TAGS = new Set(["code", "math", "pre", "script", "style", "svg", "textarea"]);
const VOID_TAGS = new Set([
  "area",
  "base",
  "br",
  "col",
  "embed",
  "hr",
  "img",
  "input",
  "link",
  "meta",
  "param",
  "source",
  "track",
  "wbr",
]);

type BudouxModule = Partial<
  Record<
    | "loadDefaultJapaneseParser"
    | "loadDefaultSimplifiedChineseParser"
    | "loadDefaultTraditionalChineseParser"
    | "loadDefaultThaiParser",
    () => BudouxParser
  >
>;

const defaultParserCache = new Map<BudouxLanguage, Promise<BudouxParser>>();

export function resolveBudouxOptions(options: OxContentOptions["budoux"]): ResolvedBudouxOptions {
  if (!options) return { enabled: false, language: DEFAULT_LANGUAGE, separator: DEFAULT_SEPARATOR };
  if (options === true)
    return { enabled: true, language: DEFAULT_LANGUAGE, separator: DEFAULT_SEPARATOR };
  return {
    enabled: options.enabled ?? true,
    language: options.language ?? DEFAULT_LANGUAGE,
    separator: options.separator ?? DEFAULT_SEPARATOR,
    parser: options.parser,
  };
}

export async function transformBudouxHtml(
  html: string,
  options: ResolvedBudouxOptions | undefined,
): Promise<string> {
  if (!options?.enabled) return html;
  const parser = options.parser ?? (await loadDefaultParser(options.language));
  return transformHtmlText(html, (text) => segmentText(text, parser, options.separator));
}

function transformHtmlText(html: string, replacer: (text: string) => string): string {
  let output = "";
  let cursor = 0;
  const protectedStack: string[] = [];

  while (cursor < html.length) {
    const protectedTag = protectedStack.at(-1);
    if (protectedTag) {
      const closeStart = findClosingTag(html, cursor, protectedTag);
      if (closeStart === -1) {
        output += html.slice(cursor);
        break;
      }
      if (closeStart > cursor) {
        output += html.slice(cursor, closeStart);
        cursor = closeStart;
        continue;
      }
    }

    if (html.startsWith("<!--", cursor)) {
      const close = html.indexOf("-->", cursor + 4);
      if (close === -1) {
        output += html.slice(cursor);
        break;
      }
      output += html.slice(cursor, close + 3);
      cursor = close + 3;
      continue;
    }

    if (html[cursor] === "<") {
      const close = html.indexOf(">", cursor + 1);
      if (close === -1) {
        output += replacer(html.slice(cursor));
        break;
      }
      const tag = html.slice(cursor, close + 1);
      output += tag;
      updateProtectedStack(tag, protectedStack);
      cursor = close + 1;
      continue;
    }

    const nextTag = html.indexOf("<", cursor);
    const end = nextTag === -1 ? html.length : nextTag;
    output += replacer(html.slice(cursor, end));
    cursor = end;
  }

  return output;
}

function segmentText(text: string, parser: BudouxParser, separator: string): string {
  if (!text) return text;
  if (separator && text.includes(separator)) {
    return text
      .split(separator)
      .map((part) => segmentTextPart(part, parser, separator))
      .join(separator);
  }

  return segmentTextPart(text, parser, separator);
}

function segmentTextPart(text: string, parser: BudouxParser, separator: string): string {
  let output = "";
  let cursor = 0;
  for (const match of text.matchAll(ENTITY)) {
    const start = match.index ?? 0;
    output += segmentPlainText(text.slice(cursor, start), parser, separator);
    output += match[0];
    cursor = start + match[0].length;
  }
  return output + segmentPlainText(text.slice(cursor), parser, separator);
}

function segmentPlainText(text: string, parser: BudouxParser, separator: string): string {
  if (!text.trim()) return text;
  const phrases = parser.parse(text);
  return phrases.length <= 1 ? text : phrases.join(separator);
}

function updateProtectedStack(tag: string, stack: string[]): void {
  const name = tagName(tag);
  if (!name || !PROTECTED_TAGS.has(name)) return;

  if (tag.startsWith("</")) {
    const index = stack.lastIndexOf(name);
    if (index !== -1) stack.splice(index);
    return;
  }

  if (!tag.endsWith("/>") && !VOID_TAGS.has(name)) {
    stack.push(name);
  }
}

function tagName(tag: string): string | undefined {
  return /^<\/?\s*([A-Za-z][A-Za-z0-9:-]*)/.exec(tag)?.[1]?.toLowerCase();
}

function findClosingTag(html: string, from: number, tagName: string): number {
  return html.toLowerCase().indexOf(`</${tagName}`, from);
}

async function loadDefaultParser(language: BudouxLanguage): Promise<BudouxParser> {
  const existing = defaultParserCache.get(language);
  if (existing) return existing;

  const pending = import("budoux")
    .then((mod: BudouxModule) => {
      const loader = mod[loaderName(language)];
      if (!loader) {
        throw new Error(`[ox-content] BudouX language "${language}" is not supported by budoux.`);
      }
      return loader();
    })
    .catch((error: unknown) => {
      throw new Error(
        [
          "[ox-content] The `budoux` option requires the optional `budoux` package.",
          "Install it with your package manager or pass a custom `budoux.parser`.",
          `Original error: ${error instanceof Error ? error.message : String(error)}`,
        ].join(" "),
      );
    });
  defaultParserCache.set(language, pending);
  return pending;
}

function loaderName(language: BudouxLanguage): keyof BudouxModule {
  switch (language) {
    case "ja":
      return "loadDefaultJapaneseParser";
    case "zh-hans":
      return "loadDefaultSimplifiedChineseParser";
    case "zh-hant":
      return "loadDefaultTraditionalChineseParser";
    case "th":
      return "loadDefaultThaiParser";
  }
}
