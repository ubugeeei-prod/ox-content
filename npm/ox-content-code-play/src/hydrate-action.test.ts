import { describe, expect, it, vi } from "vite-plus/test";
import { readPlayPayload, runPlayAction } from "./hydrate-action";
import { encodePayload } from "./payload";
import { DEFAULT_VIEWERS } from "./config";
import { errorResult } from "./result";

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
