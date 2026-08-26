import { describe, expect, it, vi } from "vite-plus/test";
import {
  idleRunActionState,
  readPlayPayload,
  resultRunActionState,
  runningRunActionState,
  runPlayAction,
} from "./hydrate-action";
import { resultPanelToShow } from "./hydrate";
import { encodePayload } from "./payload";
import { DEFAULT_VIEWERS } from "./config";
import { errorResult } from "./result";
import { emptyTiming } from "./timing";

describe("readPlayPayload", () => {
  it("returns undefined for malformed widgets instead of throwing", () => {
    expect(readPlayPayload("")).toBeUndefined();
    expect(readPlayPayload("%%%not-base64%%%")).toBeUndefined();
    expect(readPlayPayload(Buffer.from("[]").toString("base64"))).toBeUndefined();
  });

  it("decodes a valid payload", () => {
    const encoded = encodePayload({
      language: "javascript",
      code: "1",
      capabilities: { execute: true, typecheck: false },
      config: {},
      viewers: { ...DEFAULT_VIEWERS },
      ui: "default",
      timeoutMs: 1,
    });
    expect(readPlayPayload(encoded)?.language).toBe("javascript");
  });
});

describe("runPlayAction", () => {
  it("re-enables controls after success and after a thrown action", async () => {
    const busy: boolean[] = [];
    const results: unknown[] = [];
    const errors: unknown[] = [];

    await runPlayAction({
      action: async () => errorResult("ok"),
      setBusy: (value) => busy.push(value),
      onResult: (result) => results.push(result.status),
      onError: (error) => errors.push(error),
    });
    expect(busy).toEqual([true, false]);
    expect(results).toEqual(["error"]);
    expect(errors).toEqual([]);

    const setBusy = vi.fn();
    await runPlayAction({
      action: async () => {
        throw new Error("adapter exploded");
      },
      setBusy,
      onResult: () => {
        throw new Error("should not paint a result");
      },
      onError: (error) => errors.push(error),
    });
    expect(setBusy.mock.calls.map((call) => call[0])).toEqual([true, false]);
    expect(errors).toHaveLength(1);
    expect(errors[0]).toBeInstanceOf(Error);
  });
});

describe("run action states", () => {
  it("models idle, running, ok, offline, and cancelled states", () => {
    expect(idleRunActionState()).toEqual({ phase: "idle" });
    expect(runningRunActionState("execute", 12)).toEqual({
      phase: "running",
      action: "execute",
      startedAtMs: 12,
    });
    expect(resultRunActionState("execute", errorResult("ok", "code-play", "ok"), 20)).toEqual(
      expect.objectContaining({ phase: "result", action: "execute", finishedAtMs: 20 }),
    );
    expect(
      resultRunActionState("execute", errorResult("offline", "code-play", "offline"), 20),
    ).toEqual(expect.objectContaining({ phase: "offline", message: "offline" }));
    expect(
      resultRunActionState("execute", errorResult("Run cancelled.", "code-play", "cancelled"), 20),
    ).toEqual(expect.objectContaining({ phase: "result", message: "Run cancelled." }));
  });
});

describe("resultPanelToShow", () => {
  const cargoProgress = [
    "   Compiling playground v0.0.1 (/playground)",
    "    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.77s",
    "     Running `target/debug/playground`",
  ].join("\n");

  it("keeps successful runs on stdio even when cargo wrote progress to stderr", () => {
    expect(
      resultPanelToShow(
        {
          status: "ok",
          stdio: [
            { stream: "stdout", text: "hello\n", timestampMs: 0 },
            { stream: "stderr", text: cargoProgress, timestampMs: 0 },
          ],
          diagnostics: [],
          provenance: {},
          timing: emptyTiming(),
          stdout: "hello\n",
          stderr: cargoProgress,
        },
        true,
      ),
    ).toBe("stdio");
  });

  it("opens stderr when the run failed or reported an error diagnostic", () => {
    expect(
      resultPanelToShow(
        {
          status: "error",
          stdio: [{ stream: "stderr", text: "error: expected `;`", timestampMs: 0 }],
          diagnostics: [],
          provenance: {},
          timing: emptyTiming(),
          stdout: "",
          stderr: "error: expected `;`",
        },
        true,
      ),
    ).toBe("stderr");
    expect(
      resultPanelToShow(
        {
          status: "ok",
          stdio: [],
          diagnostics: [{ message: "unused", severity: "error", source: "rustc" }],
          provenance: {},
          timing: emptyTiming(),
          stdout: "",
          stderr: cargoProgress,
        },
        true,
      ),
    ).toBe("stderr");
  });
});
