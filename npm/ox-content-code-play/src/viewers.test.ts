import { describe, expect, it } from "vite-plus/test";
import { DEFAULT_VIEWERS } from "./config";
import { withStdioText } from "./stdio";
import type { PlayPayload, RunResult } from "./types";
import { renderPlayUi } from "./ui";
import { renderStderrHtml } from "./viewers";

function result(partial: Partial<RunResult> = {}): RunResult {
  return withStdioText({
    status: "ok",
    stdio: [],
    diagnostics: [],
    provenance: {},
    timing: { totalMs: 0, phases: [] },
    ...partial,
  });
}

function payload(overrides: Partial<PlayPayload> = {}): PlayPayload {
  return {
    language: "javascript",
    code: "console.log(1)",
    capabilities: { execute: true, typecheck: false },
    config: {},
    viewers: { ...DEFAULT_VIEWERS },
    ui: "default",
    timeoutMs: 1000,
    ...overrides,
  };
}

describe("stderr viewer", () => {
  it("shows the empty state when there is no stderr and no error or warning", () => {
    expect(renderStderrHtml(undefined)).toBe(`<p class="ox-code-play__empty">No stderr.</p>`);
    expect(renderStderrHtml(result())).toBe(`<p class="ox-code-play__empty">No stderr.</p>`);
    expect(
      renderStderrHtml(
        result({
          diagnostics: [{ message: "note", severity: "info", source: "code-play" }],
        }),
      ),
    ).toBe(`<p class="ox-code-play__empty">No stderr.</p>`);
  });

  it("renders only stderr chunks and never stdout text", () => {
    const html = renderStderrHtml(
      result({
        stdio: [
          { stream: "stdout", text: "ok\n", timestampMs: 1.2 },
          { stream: "stderr", text: "bad\n", timestampMs: 3 },
        ],
      }),
    );
    expect(html).toContain("stderr +3.0ms");
    expect(html).toContain("bad\n");
    expect(html).not.toContain("stdout");
    expect(html).not.toContain("ok\n");
  });

  it("surfaces typecheck diagnostics when the stderr stream is empty", () => {
    const html = renderStderrHtml(
      result({
        status: "error",
        diagnostics: [
          {
            message: "Type 'string' is not assignable to type 'number'.",
            severity: "error",
            line: 1,
            column: 7,
            source: "tsgo",
          },
        ],
      }),
    );
    expect(html).toContain("ox-code-play__diag--error");
    expect(html).toContain(":1:7");
    expect(html).toContain("Type &#39;string&#39; is not assignable to type &#39;number&#39;.");
    expect(html).not.toContain("No stderr.");
    expect(html).not.toContain("ox-code-play__stdio");
  });

  it("renders stderr chunks and warning diagnostics together", () => {
    const html = renderStderrHtml(
      result({
        stdio: [{ stream: "stderr", text: "warning: unused\n", timestampMs: 4 }],
        diagnostics: [{ message: "unused", severity: "warning", source: "rustc" }],
      }),
    );
    expect(html).toContain("warning: unused");
    expect(html).toContain("ox-code-play__diag--warning");
  });
});

describe("stderr UI flags", () => {
  it("adds a stderr tab and panel when viewers.stderr is enabled", () => {
    const html = renderPlayUi({ payload: payload() });
    expect(html).toContain('data-ox-panel="stderr"');
    expect(html).toContain('data-panel="stderr"');
    expect(html).toContain("No stderr.");
  });

  it("omits the stderr tab and panel when viewers.stderr is false", () => {
    const html = renderPlayUi({
      payload: payload({ viewers: { ...DEFAULT_VIEWERS, stderr: false } }),
    });
    expect(html).not.toContain('data-ox-panel="stderr"');
    expect(html).not.toContain('data-panel="stderr"');
    expect(html).toContain('data-panel="stdio"');
  });

  it("keeps the compact stderr panel visible and hides compact tabs", () => {
    const html = renderPlayUi({ payload: payload({ ui: "compact" }) });
    expect(html).toContain("ox-code-play--compact");
    expect(html).not.toContain("ox-code-play__tabs");
    expect(html).toContain('data-panel="stderr"');
    expect(html).not.toMatch(/data-panel="stderr"[^>]*\bhidden\b/);
    expect(html).toMatch(/data-panel="config"[^>]*\bhidden\b/);
  });
});
