import type { CodePlayPreset, ViewerFlags } from "./types";

export interface ParsedPlayOptions {
  typecheck: boolean;
  title?: string;
  config: Record<string, unknown>;
  ui?: CodePlayPreset;
  viewers?: Partial<ViewerFlags>;
  timeoutMs?: number;
}

export function parsePlayMeta(meta: string): ParsedPlayOptions {
  const options = emptyPlayOptions();
  for (const token of splitPlayInfo(meta)) {
    if (token === "typecheck") {
      options.typecheck = true;
      continue;
    }
    if (token === "play-compact") {
      options.ui = "compact";
      continue;
    }
    if (token === "play-headless") {
      options.ui = "headless";
      continue;
    }
    const pair = readTokenPair(token);
    if (!pair) {
      continue;
    }
    applyPlayOption(options, pair.name, pair.value);
  }
  return options;
}

export function parseCodePlayAttributes(attrs: string): ParsedPlayOptions {
  const options = emptyPlayOptions();
  for (const [name, value] of readAttributes(attrs)) {
    if (name === "typecheck") {
      options.typecheck = value !== "false";
      continue;
    }
    if (name === "title") {
      options.title = value;
      continue;
    }
    if (name === "ui") {
      options.ui = parseUi(value);
      continue;
    }
    if (name === "timeout" || name === "timeout-ms" || name === "timeoutms") {
      options.timeoutMs = parsePositiveInt(value);
      continue;
    }
    if (name === "viewers") {
      options.viewers = parseViewers(value);
      continue;
    }
    if (name.startsWith("config-")) {
      options.config[configKeyFromAttribute(name.slice("config-".length))] =
        coerceOptionValue(value);
    }
  }
  return options;
}

export function readCodePlayAttribute(attrs: string, name: string): string | undefined {
  return readAttributes(attrs).get(name.toLowerCase());
}

export function splitPlayInfo(info: string): string[] {
  const tokens: string[] = [];
  let current = "";
  let quote: '"' | "'" | undefined;
  let escaped = false;
  for (const char of info.trim()) {
    if (escaped) {
      current += `\\${char}`;
      escaped = false;
      continue;
    }
    if (char === "\\") {
      escaped = true;
      continue;
    }
    if (quote) {
      if (char === quote) {
        current += char;
        quote = undefined;
      } else {
        current += char;
      }
      continue;
    }
    if (char === '"' || char === "'") {
      current += char;
      quote = char;
      continue;
    }
    if (/\s/.test(char)) {
      if (current) {
        tokens.push(current);
        current = "";
      }
      continue;
    }
    current += char;
  }
  if (current) {
    tokens.push(current);
  }
  return tokens;
}

function emptyPlayOptions(): ParsedPlayOptions {
  return { typecheck: false, config: {} };
}

function applyPlayOption(options: ParsedPlayOptions, rawName: string, value: string): void {
  const name = rawName.toLowerCase();
  if (name === "play-title") {
    options.title = value;
    return;
  }
  if (name === "play-ui") {
    options.ui = parseUi(value);
    return;
  }
  if (name === "play-timeout" || name === "play-timeout-ms" || name === "play-timeoutms") {
    options.timeoutMs = parsePositiveInt(value);
    return;
  }
  if (name === "play-viewers") {
    options.viewers = parseViewers(value);
    return;
  }
  const configKey =
    name.startsWith("play-config:") || name.startsWith("play-config.")
      ? rawName.slice("play-config:".length)
      : name.startsWith("play-")
        ? rawName.slice("play-".length)
        : "";
  if (configKey) {
    options.config[configKey] = coerceOptionValue(value);
  }
}

function readTokenPair(token: string): { name: string; value: string } | undefined {
  const index = token.indexOf("=");
  if (index === -1) {
    return undefined;
  }
  return { name: token.slice(0, index), value: unquote(token.slice(index + 1)) };
}

function readAttributes(attrs: string): Map<string, string> {
  const values = new Map<string, string>();
  const pattern = /([:\w-]+)(?:\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>]+)))?/g;
  for (const match of attrs.matchAll(pattern)) {
    const name = match[1]?.toLowerCase();
    if (!name) {
      continue;
    }
    values.set(name, match[2] ?? match[3] ?? match[4] ?? "true");
  }
  return values;
}

function parseUi(value: string): CodePlayPreset | undefined {
  return value === "default" || value === "compact" || value === "headless" ? value : undefined;
}

function parsePositiveInt(value: string): number | undefined {
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
}

function parseViewers(value: string): Partial<ViewerFlags> | undefined {
  const viewers: Partial<ViewerFlags> = {};
  for (const token of value.split(",")) {
    const trimmed = token.trim();
    const enabled = !trimmed.startsWith("-");
    const key = (enabled ? trimmed : trimmed.slice(1)) as keyof ViewerFlags;
    if (
      key === "config" ||
      key === "stdio" ||
      key === "stderr" ||
      key === "provenance" ||
      key === "timing"
    ) {
      viewers[key] = enabled;
    }
  }
  return Object.keys(viewers).length > 0 ? viewers : undefined;
}

function coerceOptionValue(value: string): string | number | boolean {
  if (value === "true") {
    return true;
  }
  if (value === "false") {
    return false;
  }
  const numeric = Number(value);
  return value.trim() !== "" && Number.isFinite(numeric) ? numeric : value;
}

function configKeyFromAttribute(value: string): string {
  return value.replace(/-([a-z])/g, (_, char: string) => char.toUpperCase());
}

function unquote(value: string): string {
  if (
    (value.startsWith('"') && value.endsWith('"')) ||
    (value.startsWith("'") && value.endsWith("'"))
  ) {
    return value.slice(1, -1);
  }
  return value;
}
