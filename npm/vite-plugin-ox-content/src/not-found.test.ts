import { describe, expect, it } from "vite-plus/test";
import * as path from "node:path";
import {
  applyNotFoundNoindex,
  isNotFoundSourcePath,
  missingNotFoundSourceWarning,
  notFoundVirtualInputPath,
  resolveNotFoundOptions,
  resolveNotFoundSourcePath,
} from "./not-found";

describe("resolveNotFoundOptions", () => {
  it("omitted => false; true => true; {} => true", () => {
    expect(resolveNotFoundOptions(undefined)).toEqual({ enabled: false, source: "404.md" });
    expect(resolveNotFoundOptions(false)).toEqual({ enabled: false, source: "404.md" });
    expect(resolveNotFoundOptions(true)).toEqual({ enabled: true, source: "404.md" });
    expect(resolveNotFoundOptions({})).toEqual({ enabled: true, source: "404.md" });
    expect(resolveNotFoundOptions({ source: "missing.md" })).toEqual({
      enabled: true,
      source: "missing.md",
    });
  });
});

describe("notFound source helpers", () => {
  const srcDir = path.resolve("/repo/content");

  it("treats 404.md at the content root as the 404 source", () => {
    expect(isNotFoundSourcePath(path.join(srcDir, "404.md"), srcDir)).toBe(true);
    expect(isNotFoundSourcePath(path.join(srcDir, "guide.md"), srcDir)).toBe(false);
    expect(isNotFoundSourcePath(path.join(srcDir, "guide", "404.md"), srcDir)).toBe(false);
  });

  it("honors a custom source override", () => {
    expect(
      isNotFoundSourcePath(path.join(srcDir, "missing.md"), srcDir, { source: "missing.md" }),
    ).toBe(true);
    expect(resolveNotFoundSourcePath(srcDir, { source: "missing.md" })).toBe(
      path.resolve(srcDir, "missing.md"),
    );
  });

  it("uses a virtual 404.md path so pretty-dir output stays 404/index.html", () => {
    expect(notFoundVirtualInputPath(srcDir)).toBe(path.resolve(srcDir, "404.md"));
  });

  it("warns when the source file is missing", () => {
    expect(missingNotFoundSourceWarning("404.md")).toBe(
      "[ox-content] notFound is enabled but 404.md was not found; the 404 page was not written",
    );
  });

  it("injects a robots noindex tag", () => {
    expect(applyNotFoundNoindex("<html><head><title>X</title></head></html>")).toContain(
      '<meta name="robots" content="noindex">',
    );
  });
});
