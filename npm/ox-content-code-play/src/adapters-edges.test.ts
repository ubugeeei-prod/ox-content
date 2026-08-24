import { describe, expect, it } from "vite-plus/test";
import { createCodePlay } from "./client";
import { createMemoryTransport } from "./transport";

describe("rust crateType selection", () => {
  it("auto-selects lib when there is no main and honors an explicit crateType", async () => {
    const seen: string[] = [];
    const transport = createMemoryTransport((request) => {
      const body = JSON.parse(request.body ?? "{}") as { crateType: string };
      seen.push(body.crateType);
      return { ok: true, status: 200, text: JSON.stringify({ success: true, stdout: "" }) };
    });
    const play = createCodePlay({ languages: { rust: true }, transport });
    await play
      .createSession({ language: "rust", code: "pub fn add(a: i32, b: i32) -> i32 { a + b }" })
      .run();
    await play
      .createSession({
        language: "rust",
        code: "pub fn add(a: i32, b: i32) -> i32 { a + b }",
        config: { crateType: "bin" },
      })
      .run();
    expect(seen).toEqual(["lib", "bin"]);
  });

  it("treats async fn main as a binary crate", async () => {
    const transport = createMemoryTransport((request) => {
      const body = JSON.parse(request.body ?? "{}") as { crateType: string };
      expect(body.crateType).toBe("bin");
      return { ok: true, status: 200, text: JSON.stringify({ success: true, stdout: "ok\n" }) };
    });
    const play = createCodePlay({ languages: { rust: true }, transport });
    const result = await play.createSession({ language: "rust", code: "async fn main() {}" }).run();
    expect(result.status).toBe("ok");
  });

  it("treats invalid playground JSON as a compile error on stderr", async () => {
    const transport = createMemoryTransport(() => ({
      ok: true,
      status: 200,
      text: "<html>nope</html>",
    }));
    const play = createCodePlay({ languages: { rust: true }, transport });
    const result = await play.createSession({ language: "rust", code: "fn main() {}" }).run();
    expect(result.status).toBe("error");
    expect(result.stdio).toEqual([
      expect.objectContaining({ stream: "stderr", text: "<html>nope</html>" }),
    ]);
  });
});

describe("go vet and typecheck", () => {
  it("sends withVet=true by default and can disable it", async () => {
    const seen: string[] = [];
    const transport = createMemoryTransport((request) => {
      const params = new URLSearchParams(request.body ?? "");
      seen.push(params.get("withVet") ?? "");
      return { ok: true, status: 200, text: JSON.stringify({ Events: [] }) };
    });
    const play = createCodePlay({ languages: { go: true }, transport });
    await play.createSession({ language: "go", code: "package main\nfunc main() {}" }).run();
    await play
      .createSession({
        language: "go",
        code: "package main\nfunc main() {}",
        config: { withVet: false },
      })
      .run();
    expect(seen).toEqual(["true", "false"]);
  });

  it("maps vet errors to diagnostics and does not set execute provenance on typecheck", async () => {
    const transport = createMemoryTransport(() => ({
      ok: true,
      status: 200,
      text: JSON.stringify({
        Errors: "",
        VetErrors: "prog.go:4:2: unreachable code",
      }),
    }));
    const play = createCodePlay({ languages: { go: true }, transport });
    const check = await play
      .createSession({ language: "go", code: "package main\nfunc main() {}" })
      .typecheck();
    expect(check.status).toBe("error");
    expect(check.diagnostics).toEqual([
      {
        message: "unreachable code",
        severity: "error",
        line: 4,
        column: 2,
        source: "vet",
      },
    ]);
    expect(check.provenance.compile?.runtime).toBe("go");
    expect(check.provenance.execute).toBeUndefined();
  });
});

describe("remote compile vs run failures", () => {
  it("fails when compile.code is non-zero even if run is missing", async () => {
    const transport = createMemoryTransport(() => ({
      ok: true,
      status: 200,
      text: JSON.stringify({
        compile: { stdout: "", stderr: "error: expected ';'\n", code: 1 },
      }),
    }));
    const play = createCodePlay({
      languages: { java: { endpoint: "https://exec.example/api/v2/piston" } },
      transport,
    });
    const result = await play.createSession({ language: "java", code: "class Main {}" }).run();
    expect(result.status).toBe("error");
    expect(result.stdio).toEqual([
      expect.objectContaining({ stream: "stderr", text: "error: expected ';'\n" }),
    ]);
    expect(result.diagnostics[0]?.message).toBe("error: expected ';'\n");
    expect(result.provenance.compile?.sandbox).toBe("piston");
  });

  it("fails when run exits with a signal and keeps compile then run streams in order", async () => {
    const transport = createMemoryTransport((request) => {
      expect(request.url).toBe("https://exec.example/execute");
      return {
        ok: true,
        status: 200,
        text: JSON.stringify({
          compile: { stdout: "compiled\n", stderr: "", code: 0 },
          run: { stdout: "partial\n", stderr: "killed\n", code: 1, signal: "SIGKILL" },
        }),
      };
    });
    const play = createCodePlay({
      languages: { python: { endpoint: "https://exec.example/execute" } },
      transport,
    });
    const result = await play.createSession({ language: "py", code: "print(1)" }).run();
    expect(result.status).toBe("error");
    expect(result.stdio.map((event) => [event.stream, event.text])).toEqual([
      ["stdout", "compiled\n"],
      ["stdout", "partial\n"],
      ["stderr", "killed\n"],
    ]);
    expect(result.diagnostics[0]?.source).toBe("python");
  });

  it("fails a non-ok transport response without inventing a local shell", async () => {
    const transport = createMemoryTransport(() => ({
      ok: false,
      status: 502,
      text: "bad gateway",
    }));
    const play = createCodePlay({
      languages: { ruby: { endpoint: "https://exec.example/api/v2/piston" } },
      transport,
    });
    const result = await play.createSession({ language: "ruby", code: "puts 1" }).run();
    expect(result.status).toBe("error");
    expect(result.diagnostics[0]?.message).toBe("bad gateway");
    expect(result.diagnostics[0]?.message).not.toMatch(/spawn|child_process|\/bin\/sh/);
  });
});
