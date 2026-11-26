import type { IncomingMessage, ServerResponse } from "node:http";

export const PROXY_MAX_BODY_BYTES = 256 * 1024;

export class ProxyRequestError extends Error {
  readonly statusCode: number;

  constructor(statusCode: number, message: string) {
    super(message);
    this.name = "ProxyRequestError";
    this.statusCode = statusCode;
  }
}

export function assertHttpUrl(url: string): URL {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    throw new ProxyRequestError(400, "Invalid Code Play proxy destination.");
  }
  if (parsed.protocol !== "https:" && parsed.protocol !== "http:") {
    throw new ProxyRequestError(400, "Code Play proxy destination must be http(s).");
  }
  if (parsed.username || parsed.password) {
    throw new ProxyRequestError(400, "Code Play proxy destination must not include credentials.");
  }
  return parsed;
}

export async function readLimitedBody(
  req: IncomingMessage,
  limit = PROXY_MAX_BODY_BYTES,
): Promise<Buffer> {
  if (req.method !== "POST") {
    throw new ProxyRequestError(405, "Code Play proxy only accepts POST.");
  }
  const chunks: Buffer[] = [];
  let size = 0;
  for await (const chunk of req) {
    const buffer = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk);
    size += buffer.byteLength;
    if (size > limit) {
      throw new ProxyRequestError(413, "Code Play proxy request body is too large.");
    }
    chunks.push(buffer);
  }
  return Buffer.concat(chunks);
}

export interface TypecheckProxyOptions {
  /** Server-side `languages.typescript.config.tsgoCommand`. Requests cannot change it. */
  tsgoCommand?: unknown;
}

export async function typecheckProxy(
  req: IncomingMessage,
  res: ServerResponse,
  options: TypecheckProxyOptions = {},
): Promise<void> {
  try {
    // A JSON body cannot be sent cross-origin without a CORS preflight.
    if (!isJsonContentType(req.headers["content-type"])) {
      throw new ProxyRequestError(415, "Code Play typecheck expects application/json.");
    }
    const body = JSON.parse((await readLimitedBody(req)).toString("utf8")) as {
      language?: string;
      code?: string;
      config?: Record<string, unknown>;
    };
    const language = body.language ?? "typescript";
    if (language !== "typescript") {
      throw new ProxyRequestError(400, "Code Play typecheck only supports TypeScript.");
    }
    const config = { ...body.config };
    delete config.tsgoCommand;
    const { createCodePlay } = await import("./client");
    const play = createCodePlay({
      languages: {
        typescript:
          typeof options.tsgoCommand === "string"
            ? { config: { tsgoCommand: options.tsgoCommand } }
            : true,
      },
    });
    const result = await play
      .createSession({ language, code: typeof body.code === "string" ? body.code : "", config })
      .typecheck();
    writeJson(res, 200, result);
  } catch (error) {
    writeProxyError(res, error, 400, "Typecheck failed.");
  }
}

export async function proxy(
  req: IncomingMessage,
  res: ServerResponse,
  url: string,
  contentType: string,
): Promise<void> {
  try {
    assertHttpUrl(url);
    const body = await readLimitedBody(req);
    const response = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": contentType },
      body: Uint8Array.from(body),
    });
    res.statusCode = response.status;
    res.setHeader("Content-Type", response.headers.get("content-type") ?? "application/json");
    res.end(await response.text());
  } catch (error) {
    writeProxyError(res, error, 502, "Code Play proxy failed.");
  }
}

export function writeProxyError(
  res: ServerResponse,
  error: unknown,
  fallbackStatus: number,
  fallbackMessage: string,
): void {
  if (error instanceof ProxyRequestError) {
    writeJson(res, error.statusCode, { error: error.message });
    return;
  }
  writeJson(res, fallbackStatus, { error: fallbackMessage });
}

function isJsonContentType(value: string | undefined): boolean {
  return value?.split(";")[0]?.trim().toLowerCase() === "application/json";
}

function writeJson(res: ServerResponse, status: number, body: unknown): void {
  res.statusCode = status;
  res.setHeader("Content-Type", "application/json; charset=utf-8");
  res.end(JSON.stringify(body));
}
