import { afterEach, describe, expect, it, vi } from "vite-plus/test";

afterEach(() => {
  vi.doUnmock("@ox-content/napi");
  vi.doUnmock("node:module");
  vi.resetModules();
});

describe("shared asynchronous native binding", () => {
  it("loads concurrent extension callers once and preserves named/default export precedence", async () => {
    vi.resetModules();
    let defaultReads = 0;
    const parse = () => "named";
    vi.doMock("@ox-content/napi", () => ({
      default: {
        parse: () => "default",
        get transform() {
          defaultReads++;
          return () => "transformed";
        },
      },
      parse,
    }));
    const { importNapiModule } = await import("./napi");
    const callers = await Promise.all(Array.from({ length: 100 }, () => importNapiModule()));
    for (const binding of callers) {
      expect(binding.parse).toBe(parse);
      expect(binding.transform("source")).toBe("transformed");
    }
    await importNapiModule();
    expect(defaultReads).toBe(1);
  });

  it("shares successful require fallback without retrying the failed import for each caller", async () => {
    vi.resetModules();
    const requireBinding = vi.fn(() => ({ transform: () => "fallback" }));
    vi.doMock("@ox-content/napi", () => {
      throw new Error("ESM unavailable");
    });
    vi.doMock("node:module", () => ({ createRequire: () => requireBinding }));
    const { importNapiModule } = await import("./napi");
    for (const binding of await Promise.all([importNapiModule(), importNapiModule()])) {
      expect(binding.transform("source")).toBe("fallback");
    }
    await importNapiModule();
    expect(requireBinding).toHaveBeenCalledTimes(1);
  });

  it("retains both failure causes and retries a failed load", async () => {
    vi.resetModules();
    const importError = new Error("ESM unavailable");
    const requireError = new Error("binding not built yet");
    const requireBinding = vi
      .fn()
      .mockImplementationOnce(() => {
        throw requireError;
      })
      .mockReturnValue({ transform: () => "recovered" });
    vi.doMock("@ox-content/napi", () => {
      throw importError;
    });
    vi.doMock("node:module", () => ({ createRequire: () => requireBinding }));
    const { importNapiModule } = await import("./napi");
    const failed = await Promise.allSettled([importNapiModule(), importNapiModule()]);
    for (const result of failed) {
      expect(result.status).toBe("rejected");
      if (result.status === "rejected") {
        expect(result.reason).toBeInstanceOf(AggregateError);
        expect(result.reason.errors).toHaveLength(2);
        expect(result.reason.errors[1]).toBe(requireError);
      }
    }
    expect((await importNapiModule()).transform("source")).toBe("recovered");
    expect(requireBinding).toHaveBeenCalledTimes(2);
  });
});
