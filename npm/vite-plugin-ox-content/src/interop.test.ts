import { describe, expect, it } from "vite-plus/test";
import { interopDefault } from "./interop";

describe("interopDefault", () => {
  it("unwraps the double-wrapped namespace produced by the CJS interop (#452)", () => {
    // The CommonJS build loads ESM-only deps via require(esm); the interop
    // helper surfaces the default export as a `{ default: fn }` namespace.
    const plugin = () => {};
    const doubleWrapped = { default: plugin } as unknown as typeof plugin;

    expect(interopDefault(doubleWrapped)).toBe(plugin);
  });

  it("passes a bare function through unchanged (native ESM)", () => {
    const plugin = () => {};

    expect(interopDefault(plugin)).toBe(plugin);
  });

  it("passes a plain object without a default export through unchanged", () => {
    const value = { plugins: [] };

    expect(interopDefault(value)).toBe(value);
  });

  it("passes nullish values through unchanged", () => {
    expect(interopDefault(null)).toBe(null);
    expect(interopDefault(undefined)).toBe(undefined);
  });
});
