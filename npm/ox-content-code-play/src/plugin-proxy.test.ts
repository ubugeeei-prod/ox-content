import { Readable } from "node:stream";
import type { IncomingMessage, ServerResponse } from "node:http";
import { afterEach, describe, expect, it } from "vite-plus/test";
import {
  assertHttpUrl,
  PROXY_MAX_BODY_BYTES,
  ProxyRequestError,
  proxy,
  readLimitedBody,
  typecheckProxy,
  writeProxyError,
} from "./plugin-proxy";

const originalFetch = globalThis.fetch;

afterEach(() => {
  globalThis.fetch = originalFetch;
});

describe("assertHttpUrl", () => {
  it("accepts http and https destinations", () => {
    expect(assertHttpUrl("https://play.rust-lang.org/execute").host).toBe("play.rust-lang.org");
    expect(assertHttpUrl("http://127.0.0.1:8080/execute").protocol).toBe("http:");
  });

  it("rejects non-http schemes, invalid URLs, and embedded credentials", () => {
    expect(() => assertHttpUrl("file:///etc/passwd")).toThrow(ProxyRequestError);
    expect(() => assertHttpUrl("not a url")).toThrow(/Invalid Code Play proxy destination/);
    expect(() => assertHttpUrl("https://user:secret@play.rust-lang.org/execute")).toThrow(
      /must not include credentials/,
    );
    try {
      assertHttpUrl("ftp://play.rust-lang.org/execute");
    } catch (error) {
      expect(error).toBeInstanceOf(ProxyRequestError);
      expect((error as ProxyRequestError).statusCode).toBe(400);
    }
  });
});

describe("readLimitedBody", () => {
  it("reads a POST body and rejects other methods", async () => {
    await expect(readLimitedBody(request("hello", "GET"))).rejects.toMatchObject({
      statusCode: 405,
      message: "Code Play proxy only accepts POST.",
    });
    await expect(readLimitedBody(request("hello", "POST"))).resolves.toEqual(Buffer.from("hello"));
  });

  it("rejects bodies that exceed the limit before concatenating the rest", async () => {
    await expect(readLimitedBody(request("abcdef"), 4)).rejects.toMatchObject({
      statusCode: 413,
      message: "Code Play proxy request body is too large.",
    });
    expect(PROXY_MAX_BODY_BYTES).toBe(256 * 1024);
  });
});

describe("proxy", () => {
  it("forwards POST bodies to the configured destination", async () => {
    const calls: Array<{ url: string; body: string; type: string | undefined }> = [];
    globalThis.fetch = (async (url, init) => {
      calls.push({
        url: requestUrl(url),
        body: decodeBody(init?.body),
        type: new Headers(init?.headers).get("content-type") ?? undefined,
      });
      return new Response(JSON.stringify({ success: true }), {
        status: 200,
        headers: { "Content-Type": "application/json" },
      });
    }) as typeof fetch;

    const res = response();
    await proxy(
      request(JSON.stringify({ code: "fn main() {}" })),
      res,
      "https://play.rust-lang.org/execute",
      "application/json",
    );
    expect(res.statusCode).toBe(200);
    expect(res.state.body).toBe(JSON.stringify({ success: true }));
    expect(res.state.headers["content-type"]).toBe("application/json");
    expect(calls).toEqual([
      {
        url: "https://play.rust-lang.org/execute",
        body: JSON.stringify({ code: "fn main() {}" }),
        type: "application/json",
      },
    ]);
  });

  it("returns 405 for GET without calling fetch", async () => {
    let called = false;
    globalThis.fetch = (async () => {
      called = true;
      return new Response("nope");
    }) as typeof fetch;
    const res = response();
    await proxy(
      request("{}", "GET"),
      res,
      "https://play.golang.org/compile",
      "application/x-www-form-urlencoded",
    );
    expect(called).toBe(false);
    expect(res.statusCode).toBe(405);
    expect(JSON.parse(res.state.body)).toEqual({ error: "Code Play proxy only accepts POST." });
  });

  it("returns 400 for a non-http destination without calling fetch", async () => {
    let called = false;
    globalThis.fetch = (async () => {
      called = true;
      return new Response("nope");
    }) as typeof fetch;
    const res = response();
    await proxy(request("{}"), res, "file:///tmp/rust", "application/json");
    expect(called).toBe(false);
    expect(res.statusCode).toBe(400);
    expect(JSON.parse(res.state.body).error).toMatch(/http\(s\)/);
  });

  it("returns a generic 502 and never leaks the fetch error", async () => {
    globalThis.fetch = (async () => {
      throw new Error("ECONNREFUSED 10.0.0.5:443 secret-token");
    }) as typeof fetch;
    const res = response();
    await proxy(request("{}"), res, "https://play.rust-lang.org/execute", "application/json");
    expect(res.statusCode).toBe(502);
    expect(res.state.body).toBe(JSON.stringify({ error: "Code Play proxy failed." }));
    expect(res.state.body).not.toContain("secret-token");
    expect(res.state.body).not.toContain("10.0.0.5");
  });
});

describe("typecheckProxy", () => {
  it("type-checks TypeScript and returns JSON", async () => {
    const res = response();
    await typecheckProxy(
      request(JSON.stringify({ language: "typescript", code: "const n: number = 1;" })),
      res,
    );
    expect(res.statusCode).toBe(200);
    const body = JSON.parse(res.state.body) as { status: string; diagnostics: unknown[] };
    expect(body.status).toBe("ok");
    expect(body.diagnostics).toEqual([]);
  });

  it("returns 400 for invalid JSON and 405 for GET", async () => {
    const bad = response();
    await typecheckProxy(request("{not-json"), bad);
    expect(bad.statusCode).toBe(400);
    expect(JSON.parse(bad.state.body)).toEqual({ error: "Typecheck failed." });
    expect(bad.state.body).not.toContain("SyntaxError");

    const get = response();
    await typecheckProxy(request("{}", "GET"), get);
    expect(get.statusCode).toBe(405);
    expect(JSON.parse(get.state.body).error).toMatch(/POST/);
  });
});

describe("writeProxyError", () => {
  it("preserves ProxyRequestError status and hides unknown errors", () => {
    const typed = response();
    writeProxyError(typed, new ProxyRequestError(413, "too big"), 502, "fallback");
    expect(typed.statusCode).toBe(413);
    expect(JSON.parse(typed.state.body)).toEqual({ error: "too big" });

    const hidden = response();
    writeProxyError(hidden, new Error("internal stack"), 502, "Code Play proxy failed.");
    expect(hidden.statusCode).toBe(502);
    expect(hidden.state.body).toBe(JSON.stringify({ error: "Code Play proxy failed." }));
  });
});

function requestUrl(url: unknown): string {
  if (typeof url === "string") {
    return url;
  }
  if (url instanceof URL) {
    return url.href;
  }
  if (url instanceof Request) {
    return url.url;
  }
  return "";
}

function decodeBody(body: unknown): string {
  if (typeof body === "string") {
    return body;
  }
  if (body instanceof Uint8Array) {
    return Buffer.from(body).toString("utf8");
  }
  return "";
}

function request(body: string, method = "POST"): IncomingMessage {
  const req = Readable.from([Buffer.from(body)]) as IncomingMessage;
  req.method = method;
  return req;
}

function response(): ServerResponse & { state: { body: string; headers: Record<string, string> } } {
  const state = { body: "", headers: {} as Record<string, string> };
  const res = {
    statusCode: 200,
    state,
    setHeader(name: string, value: unknown) {
      state.headers[String(name).toLowerCase()] = String(value);
    },
    end(chunk?: unknown) {
      state.body = typeof chunk === "string" ? chunk : "";
    },
  };
  return res as unknown as ServerResponse & { state: typeof state };
}
