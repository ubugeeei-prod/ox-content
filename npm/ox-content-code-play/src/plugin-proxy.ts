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

export async function typecheckProxy(req: IncomingMessage, res: ServerResponse): Promise<void> {
  try {
    const body = JSON.parse((await readLimitedBody(req)).toString("utf8")) as {
      language?: string;
      code?: string;
      config?: Record<string, unknown>;
    };
    const { createCodePlay } = await import("./client");
    const language = body.language ?? "typescript";
    const play = createCodePlay({ languages: { [language]: true } });
    const result = await play
      .createSession({ language, code: body.code ?? "", config: body.config })
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

function writeJson(res: ServerResponse, status: number, body: unknown): void {
  res.statusCode = status;
  res.setHeader("Content-Type", "application/json; charset=utf-8");
  res.end(JSON.stringify(body));
}
