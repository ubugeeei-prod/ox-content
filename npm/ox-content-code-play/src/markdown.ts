import {
  parseCodePlayAttributes,
  parsePlayMeta,
  readCodePlayAttribute,
  splitPlayInfo,
} from "./authoring";
import type { CodePlayPreset, ViewerFlags } from "./types";
import type { ParsedProjectOptions } from "./authoring";

export { parseCodePlayAttributes, parsePlayMeta, type ParsedPlayOptions } from "./authoring";

export interface PlayFence {
  language: string;
  meta: string;
  code: string;
  raw: string;
  start: number;
  end: number;
  typecheck: boolean;
  title?: string;
  config: Record<string, unknown>;
  ui?: CodePlayPreset;
  viewers?: Partial<ViewerFlags>;
  timeoutMs?: number;
  project?: ParsedProjectOptions;
}

export interface ParsedFence {
  language: string;
  meta: string;
  code: string;
  raw: string;
  start: number;
  end: number;
  indent: string;
  marker: string;
}

const FENCE_OPEN = /^( {0,3})(`{3,}|~{3,})([^\n]*)$/;

export function parseTopLevelFences(source: string): ParsedFence[] {
  const lines = source.split("\n");
  const fences: ParsedFence[] = [];
  let index = 0;
  let offset = 0;

  while (index < lines.length) {
    const line = lines[index] ?? "";
    const open = FENCE_OPEN.exec(line);
    if (!open) {
      offset += line.length + 1;
      index += 1;
      continue;
    }

    const indent = open[1] ?? "";
    const marker = open[2] ?? "```";
    const info = (open[3] ?? "").trim();
    const { language, meta } = readFenceInfo(info);
    const start = offset;
    index += 1;
    offset += line.length + 1;
    const body: string[] = [];

    while (index < lines.length) {
      const candidate = lines[index] ?? "";
      const close = new RegExp(`^ {0,3}${escapeRegExp(marker)}[ \t]*$`).exec(candidate);
      if (close) {
        const raw = `${line}\n${body.join("\n")}${body.length > 0 ? "\n" : ""}${candidate}`;
        fences.push({
          language,
          meta,
          code: body.join("\n"),
          raw,
          start,
          end: start + raw.length,
          indent,
          marker,
        });
        offset += candidate.length + 1;
        index += 1;
        break;
      }
      body.push(candidate);
      offset += candidate.length + 1;
      index += 1;
    }
  }

  return fences;
}

export function parsePlayFences(source: string): PlayFence[] {
  return parseTopLevelFences(source)
    .filter((fence) => hasToken(fence.meta, "play"))
    .map((fence) => {
      const options = parsePlayMeta(fence.meta);
      return {
        language: fence.language,
        meta: fence.meta,
        code: fence.code,
        raw: fence.raw,
        start: fence.start,
        end: fence.end,
        typecheck: options.typecheck,
        title: options.title,
        config: options.config,
        ui: options.ui,
        viewers: options.viewers,
        timeoutMs: options.timeoutMs,
        project: options.project,
      };
    });
}

export function stripPlayMeta(meta: string): string {
  return splitPlayInfo(meta)
    .filter(
      (token) =>
        token &&
        token !== "play" &&
        token !== "typecheck" &&
        !token.startsWith("play-") &&
        !token.startsWith("play:"),
    )
    .join(" ");
}

export function rewritePlayFences(
  source: string,
  encode: (fence: PlayFence) => string | null,
): string {
  const fences = parsePlayFences(source);
  if (fences.length === 0) {
    return source;
  }

  let cursor = 0;
  let output = "";
  for (const fence of fences) {
    output += source.slice(cursor, fence.start);
    const encoded = encode(fence);
    if (encoded === null) {
      output += source.slice(fence.start, fence.end);
    } else {
      const cleanedMeta = stripPlayMeta(fence.meta);
      const info = [fence.language, cleanedMeta].filter(Boolean).join(" ");
      output += `<!--ox-code-play:${encoded}-->\n\`\`\`${info}\n${fence.code}\n\`\`\``;
    }
    cursor = fence.end;
  }
  output += source.slice(cursor);
  return output;
}

export function parseCodePlayTags(source: string): PlayFence[] {
  const tags: PlayFence[] = [];
  const pattern = /<CodePlay\b([^>]*)>([\s\S]*?)<\/CodePlay>/gi;
  for (const match of source.matchAll(pattern)) {
    const attrs = match[1] ?? "";
    const language =
      readCodePlayAttribute(attrs, "lang") ?? readCodePlayAttribute(attrs, "language") ?? "text";
    const options = parseCodePlayAttributes(attrs);
    tags.push({
      language,
      meta: "play",
      code: stripIndent((match[2] ?? "").replace(/^\n/, "").replace(/\n$/, "")),
      raw: match[0] ?? "",
      start: match.index ?? 0,
      end: (match.index ?? 0) + (match[0]?.length ?? 0),
      typecheck: options.typecheck,
      title: options.title,
      config: options.config,
      ui: options.ui,
      viewers: options.viewers,
      timeoutMs: options.timeoutMs,
      project: options.project,
    });
  }
  return tags;
}

function readFenceInfo(info: string): { language: string; meta: string } {
  const match = /^(\S+)(?:\s+([\s\S]*))?$/.exec(info);
  return { language: match?.[1] ?? "", meta: match?.[2]?.trim() ?? "" };
}

function hasToken(meta: string, token: string): boolean {
  return splitPlayInfo(meta).includes(token);
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function stripIndent(value: string): string {
  const lines = value.split("\n");
  const indents = lines
    .filter((line) => line.trim())
    .map((line) => line.match(/^ */)?.[0].length ?? 0);
  const indent = indents.length > 0 ? Math.min(...indents) : 0;
  return lines.map((line) => line.slice(indent)).join("\n");
}
