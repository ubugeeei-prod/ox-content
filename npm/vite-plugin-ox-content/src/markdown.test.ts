import { describe, expect, it } from "vite-plus/test";
import {
  isMdxFilePath,
  isMarkdownFilePath,
  isMarkdownModuleId,
  markdownGlobPattern,
  normalizeMarkdownExtensions,
  resolveMdxForFilePath,
  stripMarkdownExtension,
} from "./markdown";

describe("markdown extension helpers", () => {
  it("treats mdx as a first-class markdown extension", () => {
    const extensions = normalizeMarkdownExtensions(["md", ".markdown", ".mdx", ".MD"]);

    expect(extensions).toEqual([".md", ".markdown", ".mdx"]);
    expect(isMarkdownFilePath("/docs/page.mdx?raw", extensions)).toBe(true);
    expect(stripMarkdownExtension("guide/component.mdx", extensions)).toBe("guide/component");
    expect(stripMarkdownExtension("guide/intro.markdown", extensions)).toBe("guide/intro");
  });

  it("leaves query-bearing module ids such as ?raw to Vite", () => {
    expect(isMarkdownModuleId("/docs/page.md")).toBe(true);
    expect(isMarkdownModuleId("/docs/page.md?raw")).toBe(false);
    expect(isMarkdownModuleId("/docs/page.mdx?url")).toBe(false);
    expect(isMarkdownModuleId("/docs/page.md?inline")).toBe(false);
  });

  it("builds glob patterns for configured markdown extensions", () => {
    expect(markdownGlobPattern("/repo/docs", [".mdx"])).toBe("/repo/docs/**/*.mdx");
    expect(markdownGlobPattern("/repo/docs", [".md", ".mdx"])).toBe("/repo/docs/**/*.{md,mdx}");
  });

  it("infers MDX case-insensitively after stripping resource suffixes", () => {
    expect(isMdxFilePath("/docs/page.mdx?raw")).toBe(true);
    expect(isMdxFilePath("/docs/page.MDX#section")).toBe(true);
    expect(isMdxFilePath("/docs/page.md?extension=.mdx")).toBe(false);
    expect(resolveMdxForFilePath("/docs/page.mdx")).toBe(true);
    expect(resolveMdxForFilePath("/docs/page.md")).toBe(false);
    expect(resolveMdxForFilePath("/docs/page.mdx", false)).toBe(false);
    expect(resolveMdxForFilePath("/docs/page.md", true)).toBe(true);
  });
});
