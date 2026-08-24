import { describe, expect, it } from "vite-plus/test";
import { createCodePlay } from "./client";
import { javascriptExecuteRuntime } from "./javascript";
import { friendlyTransportMessage } from "./result";
import { stripTypeScript } from "./strip-typescript";
import { createMemoryTransport } from "./transport";
import {
  adapterResultFromTypecheckResponse,
  parseTsgoOutput,
  resolveTypecheckBackend,
  typecheckEndpointFailureMessage,
} from "./typescript";
import { PhaseTracker } from "./timing";
import { renderStderrHtml } from "./viewers";

describe("createCodePlay", () => {
  it("executes opted-in JavaScript and records stdio, provenance, and timing", async () => {
    const play = createCodePlay({ languages: { javascript: true } });
    const session = play.createSession({
      language: "js",
      code: `console.log("hello"); console.error("oops"); 41 + 1;`,
    });
    const result = await session.run();

    expect(result.status).toBe("ok");
    expect(result.value).toBe("42");
    expect(result.stdio.map((event) => [event.stream, event.text])).toEqual([
      ["stdout", "hello\n"],
      ["stderr", "oops\n"],
    ]);
    expect(result.stdout).toBe("hello\n");
    expect(result.stderr).toBe("oops\n");
    expect(session.stdout).toBe("hello\n");
    expect(session.stderr).toBe("oops\n");
    expect(result.provenance.execute?.runtime).toBe("node:vm");
    expect(result.timing.totalMs).toBeGreaterThanOrEqual(0);
    expect(result.timing.phases.some((phase) => phase.id === "execute")).toBe(true);
  });

  it("type-checks and executes TypeScript samples", async () => {
    const play = createCodePlay({ languages: { typescript: true } });
    const good = await play
      .createSession({ language: "ts", code: "const n: number = 1;\nconsole.log(n);" })
      .run();
    expect(good.status).toBe("ok");
    expect(good.stdio.some((event) => event.text.includes("1"))).toBe(true);
    expect(good.provenance.compile?.runtime).toBe("strip-types");

    const check = await play
      .createSession({ language: "tsx", code: "const n: number = 'nope';" })
      .typecheck();
    expect(check.status).toBe("error");
    expect(check.stdout).toBe("");
    expect(check.stderr).toBe("");
    expect(check.diagnostics.some((diagnostic) => diagnostic.message.length > 0)).toBe(true);
    expect(renderStderrHtml(check)).toContain("ox-code-play__diag--error");
    expect(renderStderrHtml(check)).not.toContain("No stderr.");
  });

  it("refuses languages that were not enabled", () => {
    const play = createCodePlay({ languages: { javascript: true } });
    expect(play.hasLanguage("python")).toBe(false);
    expect(() => play.createSession({ language: "python", code: "print(1)" })).toThrow(
      /Python is not enabled/,
    );
  });

  it("surfaces runtime errors as diagnostics and stderr", async () => {
    const play = createCodePlay({ languages: { javascript: true } });
    const result = await play
      .createSession({ language: "js", code: "throw new Error('boom');" })
      .run();
    expect(result.status).toBe("error");
    expect(result.diagnostics[0]?.message).toBe("boom");
    expect(result.stdout).toBe("");
    expect(result.stderr).toBe("boom\n");
    expect(
      result.stdio.some((event) => event.stream === "stderr" && event.text.includes("boom")),
    ).toBe(true);
  });

  it("routes console.warn and console.error to stderr, never stdout", async () => {
    const play = createCodePlay({ languages: { javascript: true } });
    const result = await play
      .createSession({
        language: "js",
        code: `console.log("out"); console.warn("warn"); console.error("err");`,
      })
      .run();
    expect(result.status).toBe("ok");
    expect(result.stdout).toBe("out\n");
    expect(result.stderr).toBe("warn\nerr\n");
    expect(result.stdio.filter((event) => event.stream === "stdout")).toEqual([
      expect.objectContaining({ text: "out\n" }),
    ]);
  });

  it("times out infinite JavaScript loops in node:vm", async () => {
    const play = createCodePlay({ languages: { javascript: true }, timeoutMs: 50 });
    const result = await play.createSession({ language: "js", code: "while (true) {}" }).run();
    expect(result.status).toBe("timeout");
    expect(result.diagnostics[0]?.message.length).toBeGreaterThan(0);
  });

  it("cancels an in-flight remote run without rejecting the session", async () => {
    const transport = createMemoryTransport(async (request) => {
      await new Promise<never>((_, reject) => {
        const fail = () =>
          reject(
            Object.assign(new Error("The Code Play run was cancelled."), { name: "AbortError" }),
          );
        if (request.signal?.aborted) {
          fail();
          return;
        }
        request.signal?.addEventListener("abort", fail, { once: true });
      });
      return { ok: true, status: 200, text: "{}" };
    });
    const session = createCodePlay({ languages: { rust: true }, transport }).createSession({
      language: "rust",
      code: "fn main() {}",
    });
    const pending = session.run();
    session.cancel();
    const result = await pending;
    expect(result.status).toBe("cancelled");
    expect(result.diagnostics[0]?.message).toMatch(/cancelled/i);
    expect(result.diagnostics[0]?.severity).toBe("info");
  });

  it("explains browser CORS failures instead of a raw fetch TypeError", () => {
    const error = new TypeError("Failed to fetch");
    expect(friendlyTransportMessage(error)).toMatch(/CORS/);
    expect(friendlyTransportMessage(new Error("offline"))).toBe("offline");
  });

  it("does not run JavaScript through page-origin Function", () => {
    expect(javascriptExecuteRuntime(true, false)).toBe("vm");
    expect(javascriptExecuteRuntime(false, true)).toBe("iframe");
    expect(() => javascriptExecuteRuntime(false, false)).toThrow(/sandbox iframe/);
  });

  it("picks a typecheck backend that cannot call a missing static-host proxy", () => {
    expect(resolveTypecheckBackend(true, undefined)).toBe("tsgo");
    expect(resolveTypecheckBackend(false, "/__ox-code-play/typecheck")).toBe("endpoint");
    expect(resolveTypecheckBackend(false, undefined)).toBe("unavailable");
  });

  it("turns transport throws into error results instead of rejecting", async () => {
    const play = createCodePlay({
      languages: { rust: true },
      transport: createMemoryTransport(() => {
        throw new Error("offline");
      }),
    });
    const result = await play.createSession({ language: "rust", code: "fn main() {}" }).run();
    expect(result.status).toBe("error");
    expect(result.diagnostics[0]?.message).toBe("offline");
  });

  it("explains a missing static-host typecheck endpoint", () => {
    expect(typecheckEndpointFailureMessage(404, "")).toMatch(/vite dev/);
    const tracker = new PhaseTracker();
    const result = adapterResultFromTypecheckResponse(
      { ok: false, status: 404, text: "" },
      "/__ox-code-play/typecheck",
      tracker,
    );
    expect(result.status).toBe("error");
    expect(result.diagnostics[0]?.message).toMatch(/endpoints\.typecheck/);
  });

  it("strips TypeScript annotations before execute", () => {
    expect(stripTypeScript("const n: number = 1;\nconsole.log(n);")).toMatch(/const n\s*= 1;/);
    expect(
      parseTsgoOutput(
        "snippet.ts(1,7): error TS2322: Type 'string' is not assignable to type 'number'.",
      ),
    ).toEqual([
      {
        message: "Type 'string' is not assignable to type 'number'.",
        severity: "error",
        line: 1,
        column: 7,
        source: "tsgo",
      },
    ]);
  });
});
