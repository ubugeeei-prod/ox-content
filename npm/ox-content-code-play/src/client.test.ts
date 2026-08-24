import { describe, expect, it } from "vite-plus/test";
import { createCodePlay } from "./client";
import { stripTypeScript } from "./strip-typescript";
import { parseTsgoOutput } from "./typescript";

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
    expect(check.diagnostics.some((diagnostic) => diagnostic.message.length > 0)).toBe(true);
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
    expect(result.diagnostics[0]?.message).toContain("boom");
    expect(
      result.stdio.some((event) => event.stream === "stderr" && event.text.includes("boom")),
    ).toBe(true);
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
