import { describe, expect, it } from "vite-plus/test";
import { createCodePlay } from "./client";
import { buildPreviewDocument } from "./framework";
import { parseGoErrors } from "./go";
import { parseRustcDiagnostics } from "./rust";
import { createMemoryTransport } from "./transport";

describe("language adapters", () => {
  it("sends rust samples to the playground transport and maps rustc output", async () => {
    const transport = createMemoryTransport((request) => {
      expect(request.url).toContain("play.rust-lang.org");
      const body = JSON.parse(request.body ?? "{}") as { crateType: string; code: string };
      expect(body.crateType).toBe("bin");
      expect(body.code).toContain("fn main");
      return {
        ok: true,
        status: 200,
        text: JSON.stringify({
          success: true,
          stdout: "ok\n",
          stderr: "warning: unused variable: `x`\n",
        }),
      };
    });
    const play = createCodePlay({ languages: { rust: true }, transport });
    const result = await play
      .createSession({ language: "rust", code: "fn main() { let x = 1; }" })
      .run();
    expect(result.status).toBe("ok");
    expect(result.stdout).toBe("ok\n");
    expect(result.stderr).toBe("warning: unused variable: `x`\n");
    expect(result.stdio[0]).toMatchObject({ stream: "stdout", text: "ok\n" });
    expect(result.diagnostics).toEqual([
      expect.objectContaining({ severity: "warning", message: "unused variable: `x`" }),
    ]);
    expect(result.provenance.compile?.host).toContain("play.rust-lang.org");
    expect(result.timing.phases.map((phase) => phase.id)).toEqual(
      expect.arrayContaining(["compile", "collect"]),
    );
  });

  it("maps rustc errors during typecheck", async () => {
    const transport = createMemoryTransport(() => ({
      ok: true,
      status: 200,
      text: JSON.stringify({
        success: false,
        stderr: "error[E0308]: mismatched types\n",
      }),
    }));
    const play = createCodePlay({ languages: { rust: true }, transport });
    const result = await play
      .createSession({ language: "rs", code: "fn add(x: i32) -> i32 { x }" })
      .typecheck();
    expect(result.status).toBe("error");
    expect(result.stdout).toBe("");
    expect(result.stderr).toBe("error[E0308]: mismatched types\n");
    expect(parseRustcDiagnostics("error[E0308]: mismatched types")[0]?.source).toBe("rustc E0308");
    expect(result.diagnostics[0]?.severity).toBe("error");
  });

  it("sends go samples to the playground and records events", async () => {
    const transport = createMemoryTransport((request) => {
      expect(request.url).toContain("play.golang.org");
      expect(request.body).toContain("package+main");
      return {
        ok: true,
        status: 200,
        text: JSON.stringify({
          Events: [{ Message: "hi\n", Kind: "stdout" }],
        }),
      };
    });
    const play = createCodePlay({ languages: { go: true }, transport });
    const result = await play
      .createSession({ language: "go", code: "package main\nfunc main() {}" })
      .run();
    expect(result.status).toBe("ok");
    expect(result.stdout).toBe("hi\n");
    expect(result.stderr).toBe("");
    expect(result.stdio[0]?.text).toBe("hi\n");
    expect(result.provenance.execute?.runtime).toBe("go-playground");
  });

  it("parses go compiler coordinates", () => {
    expect(parseGoErrors("prog.go:3:1: undefined: foo", "go")).toEqual([
      { message: "undefined: foo", severity: "error", line: 3, column: 1, source: "go" },
    ]);
  });

  it("requires a configured endpoint for remote languages and never mentions local shell spawn", async () => {
    const play = createCodePlay({ languages: { python: true, sh: true } });
    const python = await play.createSession({ language: "python", code: "print(1)" }).run();
    expect(python.status).toBe("unsupported");
    expect(python.diagnostics[0]?.message).toMatch(/endpoint/);

    const transport = createMemoryTransport((request) => {
      expect(request.url).toBe("https://exec.example/api/v2/piston/execute");
      return {
        ok: true,
        status: 200,
        text: JSON.stringify({ run: { stdout: "1\n", stderr: "", code: 0 } }),
      };
    });
    const remote = createCodePlay({
      languages: { sh: { endpoint: "https://exec.example/api/v2/piston" } },
      transport,
    });
    const sh = await remote.createSession({ language: "bash", code: "echo 1" }).run();
    expect(sh.status).toBe("ok");
    expect(sh.stdout).toBe("1\n");
    expect(sh.stderr).toBe("");
    expect(sh.provenance.execute?.sandbox).toBe("piston");
    expect(sh.stdio[0]?.text).toBe("1\n");
  });

  it("builds framework preview documents instead of executing host UI code", async () => {
    const play = createCodePlay({ languages: { vue: true, react: true } });
    const result = await play
      .createSession({ language: "vue", code: "createApp({}).mount('#app')" })
      .run();
    expect(result.status).toBe("ok");
    expect(result.preview?.html).toContain("esm.sh/vue");
    expect(result.provenance.execute?.sandbox).toBe("srcdoc");
    expect(buildPreviewDocument("react", "console.log(1)").includes("esm.sh/react")).toBe(true);
  });
});
