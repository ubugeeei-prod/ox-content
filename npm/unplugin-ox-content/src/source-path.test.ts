import { describe, expect, it } from "vite-plus/test";
import {
  isConfiguredSourceFile,
  isConfiguredSourceModule,
  isMdxSourceFile,
  resolveMdxForSourceFile,
} from "./source-path";

describe("source path detection", () => {
  it("handles uppercase MDX and bundler resource suffixes", () => {
    expect(isConfiguredSourceFile("/docs/Guide.MDX?raw#fragment", [".md", ".mdx"])).toBe(true);
    expect(isMdxSourceFile("/docs/Guide.MDX?raw#fragment")).toBe(true);
    expect(isMdxSourceFile("/docs/guide.md?mdx=false")).toBe(false);
  });

  it("leaves query-bearing ids such as ?raw to the bundler", () => {
    expect(isConfiguredSourceModule("/docs/guide.md", [".md"])).toBe(true);
    expect(isConfiguredSourceModule("/docs/guide.md?raw", [".md"])).toBe(false);
    expect(isConfiguredSourceModule("/docs/guide.md?url", [".md"])).toBe(false);
  });

  it("lets explicit configuration override extension detection", () => {
    expect(resolveMdxForSourceFile("guide.mdx")).toBe(true);
    expect(resolveMdxForSourceFile("guide.md")).toBe(false);
    expect(resolveMdxForSourceFile("guide.mdx", false)).toBe(false);
    expect(resolveMdxForSourceFile("guide.md", true)).toBe(true);
  });
});
