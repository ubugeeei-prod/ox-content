import { describe, expect, it } from "vite-plus/test";
import {
  isMarkdownFilePath,
  markdownGlobPattern,
  normalizeMarkdownExtensions,
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

  it("builds glob patterns for configured markdown extensions", () => {
    expect(markdownGlobPattern("/repo/docs", [".mdx"])).toBe("/repo/docs/**/*.mdx");
    expect(markdownGlobPattern("/repo/docs", [".md", ".mdx"])).toBe("/repo/docs/**/*.{md,mdx}");
  });
});
