import { spawn } from "node:child_process";
import { createHash } from "node:crypto";

export type GraphvizFailureMode = "error" | "warn";

export interface GraphvizOptions {
  /** Graphviz renderer command. @default 'dot' */
  command?: string;
  /** Extra arguments passed before `-Tsvg`. @default [] */
  args?: string[];
  /** Behavior when the Graphviz command is not available. @default 'error' */
  missingRenderer?: GraphvizFailureMode;
  /** Behavior when Graphviz rejects a DOT source. @default 'error' */
  renderErrors?: GraphvizFailureMode;
  /** Per-render timeout in milliseconds. @default 10000 */
  timeout?: number;
  /** Cache rendered SVGs in memory for this process. @default true */
  cache?: boolean;
  /** Cache TTL in milliseconds. @default 3600000 */
  cacheTTL?: number;
}

export interface ResolvedGraphvizOptions {
  command: string;
  args: string[];
  missingRenderer: GraphvizFailureMode;
  renderErrors: GraphvizFailureMode;
  timeout: number;
  cache: boolean;
  cacheTTL: number;
}

export interface GraphvizRenderBlock {
  source: string;
  occurrence: number;
}

interface CacheRecord {
  svg: string;
  timestamp: number;
}

interface ProcessError extends Error {
  code?: string;
  exitCode?: number | null;
  signal?: NodeJS.Signals | null;
  stderr?: string;
}

const DEFAULT_COMMAND = "dot";
const DEFAULT_TIMEOUT = 10_000;
const DEFAULT_CACHE_TTL = 3_600_000;
const rawSvgCache = new Map<string, CacheRecord>();
const rendererIdentityCache = new Map<string, Promise<string>>();
let missingRendererWarned = false;

export function clearGraphvizCache(): void {
  rawSvgCache.clear();
  rendererIdentityCache.clear();
  missingRendererWarned = false;
}

export function resolveGraphvizOptions(
  options: boolean | GraphvizOptions | undefined,
): ResolvedGraphvizOptions | false {
  if (!options) return false;
  const value = options === true ? {} : options;
  return {
    command: value.command ?? DEFAULT_COMMAND,
    args: value.args ? [...value.args] : [],
    missingRenderer: value.missingRenderer ?? "error",
    renderErrors: value.renderErrors ?? "error",
    timeout: value.timeout ?? DEFAULT_TIMEOUT,
    cache: value.cache ?? true,
    cacheTTL: value.cacheTTL ?? DEFAULT_CACHE_TTL,
  };
}

export function isResolvedGraphvizOptions(value: unknown): value is ResolvedGraphvizOptions {
  return Boolean(
    value &&
    typeof value === "object" &&
    typeof (value as ResolvedGraphvizOptions).command === "string" &&
    Array.isArray((value as ResolvedGraphvizOptions).args) &&
    typeof (value as ResolvedGraphvizOptions).timeout === "number" &&
    "missingRenderer" in value &&
    "renderErrors" in value,
  );
}

export async function rendererIdentityOrFallback(
  options: ResolvedGraphvizOptions,
): Promise<string | null> {
  try {
    return await resolveRendererIdentity(options);
  } catch (error) {
    const message = `[ox-content] Graphviz renderer not found: ${options.command}`;
    if (isMissingCommandError(error) && options.missingRenderer === "warn") {
      warnOnce(message);
      return null;
    }
    throw new Error(message);
  }
}

export async function renderGraphvizOrOriginal(
  block: GraphvizRenderBlock,
  options: ResolvedGraphvizOptions,
  identity: string,
): Promise<string | null> {
  try {
    const rawSvg = await renderGraphvizRawSvg(block.source, options, identity);
    return graphvizFigure(rawSvg, block.occurrence);
  } catch (error) {
    if (isMissingCommandError(error) && options.missingRenderer === "warn") {
      warnOnce(`[ox-content] Graphviz renderer not found: ${options.command}`);
      return null;
    }
    const message = graphvizErrorMessage(error);
    if (options.renderErrors === "warn") {
      console.warn(`[ox-content] Graphviz render error: ${message}`);
      return null;
    }
    throw new Error(`[ox-content] Graphviz render error: ${message}`);
  }
}

async function renderGraphvizRawSvg(
  source: string,
  options: ResolvedGraphvizOptions,
  identity: string,
): Promise<string> {
  const cacheKey = hashJson({
    source,
    command: options.command,
    args: options.args,
    identity,
    sanitizer: "graphviz-svg-v1",
  });
  const now = Date.now();
  if (options.cache) {
    const cached = rawSvgCache.get(cacheKey);
    if (cached && now - cached.timestamp < options.cacheTTL) return cached.svg;
  }

  const result = await runProcess(
    options.command,
    [...options.args, "-Tsvg"],
    source,
    options.timeout,
  );
  if (options.cache) rawSvgCache.set(cacheKey, { svg: result.stdout, timestamp: now });
  return result.stdout;
}

async function resolveRendererIdentity(options: ResolvedGraphvizOptions): Promise<string> {
  const key = hashJson({ command: options.command, args: options.args });
  const cached = rendererIdentityCache.get(key);
  if (cached) return cached;
  const pending = readRendererIdentity(options).catch((error) => {
    rendererIdentityCache.delete(key);
    throw error;
  });
  rendererIdentityCache.set(key, pending);
  return pending;
}

async function readRendererIdentity(options: ResolvedGraphvizOptions): Promise<string> {
  try {
    const result = await runProcess(options.command, [...options.args, "-V"], "", options.timeout);
    const version = `${result.stdout}\n${result.stderr}`.trim();
    return version || `${options.command} ${options.args.join(" ")}`.trim();
  } catch (error) {
    if (isMissingCommandError(error)) throw error;
    return `${options.command} ${options.args.join(" ")}`.trim();
  }
}

function runProcess(
  command: string,
  args: string[],
  input: string,
  timeout: number,
): Promise<{ stdout: string; stderr: string }> {
  return new Promise((resolve, reject) => {
    const child = spawn(command, args, { stdio: ["pipe", "pipe", "pipe"] });
    let stdout = "";
    let stderr = "";
    const timer = setTimeout(() => {
      child.kill();
      reject(processError(`timed out after ${timeout}ms`));
    }, timeout);

    child.stdout.setEncoding("utf8");
    child.stderr.setEncoding("utf8");
    child.stdout.on("data", (chunk: string) => {
      stdout += chunk;
    });
    child.stderr.on("data", (chunk: string) => {
      stderr += chunk;
    });
    child.on("error", (error) => {
      clearTimeout(timer);
      reject(error);
    });
    child.on("close", (exitCode, signal) => {
      clearTimeout(timer);
      if (exitCode === 0) {
        resolve({ stdout, stderr });
      } else {
        reject(
          processError(stderr || `exited with ${exitCode ?? signal}`, exitCode, signal, stderr),
        );
      }
    });
    child.stdin.end(input);
  });
}

function graphvizFigure(rawSvg: string, occurrence: number): string {
  const prefix = `ox-graphviz-${occurrence}-${hashText(rawSvg).slice(0, 10)}`;
  const svg = sanitizeGraphvizSvg(rawSvg, prefix);
  return `<figure class="ox-graphviz" role="img" aria-label="Graphviz diagram">${svg}</figure>`;
}

function sanitizeGraphvizSvg(rawSvg: string, prefix: string): string {
  const match = rawSvg.match(/<svg\b[\s\S]*<\/svg>/i);
  if (!match) throw new Error("renderer did not produce an SVG document");
  let svg = match[0]
    .replace(/<script\b[\s\S]*?<\/script>/gi, "")
    .replace(/<style\b[\s\S]*?<\/style>/gi, "")
    .replace(/<foreignObject\b[\s\S]*?<\/foreignObject>/gi, "")
    .replace(/\s+on[a-z]+\s*=\s*(?:"[^"]*"|'[^']*'|[^\s>]+)/gi, "")
    .replace(/\s+(?:href|xlink:href)\s*=\s*(["'])\s*(?!#)[\s\S]*?\1/gi, "");
  const ids = Array.from(svg.matchAll(/\sid=(["'])([^"']+)\1/gi), (match) => match[2]!);
  for (const id of ids) {
    const next = `${prefix}-${id.replace(/[^A-Za-z0-9_-]/g, "-")}`;
    svg = svg
      .replace(new RegExp(`\\sid=(["'])${escapeRegExp(id)}\\1`, "g"), ` id="${next}"`)
      .replace(new RegExp(`url\\(#${escapeRegExp(id)}\\)`, "g"), `url(#${next})`)
      .replace(new RegExp(`(["'])#${escapeRegExp(id)}\\1`, "g"), (_all, quote) => {
        return `${quote}#${next}${quote}`;
      });
  }
  return svg;
}

function graphvizErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    const process = error as ProcessError;
    return process.stderr?.trim() || process.message;
  }
  return String(error);
}

function processError(
  message: string,
  exitCode: number | null = null,
  signal: NodeJS.Signals | null = null,
  stderr = "",
): ProcessError {
  const error = new Error(message) as ProcessError;
  error.exitCode = exitCode;
  error.signal = signal;
  error.stderr = stderr;
  return error;
}

function isMissingCommandError(error: unknown): boolean {
  return error instanceof Error && (error as ProcessError).code === "ENOENT";
}

function warnOnce(message: string): void {
  if (missingRendererWarned) return;
  missingRendererWarned = true;
  console.warn(message);
}

function hashText(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function hashJson(value: unknown): string {
  return hashText(JSON.stringify(value));
}

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
