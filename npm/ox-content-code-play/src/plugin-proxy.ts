import type { IncomingMessage, ServerResponse } from "node:http";

export async function typecheckProxy(req: IncomingMessage, res: ServerResponse): Promise<void> {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }
  try {
    const body = JSON.parse(Buffer.concat(chunks).toString("utf8")) as {
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
    res.setHeader("Content-Type", "application/json; charset=utf-8");
    res.end(JSON.stringify(result));
  } catch (error) {
    res.statusCode = 400;
    res.end(error instanceof Error ? error.message : "Typecheck failed.");
  }
}

export async function proxy(
  req: IncomingMessage,
  res: ServerResponse,
  url: string,
  contentType: string,
): Promise<void> {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk));
  }
  try {
    const response = await fetch(url, {
      method: "POST",
      headers: { "Content-Type": contentType },
      body: Buffer.concat(chunks),
    });
    res.statusCode = response.status;
    res.setHeader("Content-Type", response.headers.get("content-type") ?? "application/json");
    res.end(await response.text());
  } catch (error) {
    res.statusCode = 502;
    res.end(error instanceof Error ? error.message : "Code Play proxy failed.");
  }
}
