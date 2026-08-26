import { describe, expect, it } from "vite-plus/test";
import { parseSourceFrontmatter } from "./markdown-source";

describe("markdown source frontmatter", () => {
  it("uses the native frontmatter parser for companion routing", () => {
    expect(
      parseSourceFrontmatter(
        "---\ntitle: Guide\ntags:\n  - docs\nmeta:\n  draft: false\n---\n# Guide\n",
      ),
    ).toEqual({
      title: "Guide",
      tags: ["docs"],
      meta: { draft: false },
    });
  });

  it("keeps malformed frontmatter on the native empty-object path", () => {
    expect(parseSourceFrontmatter("---\ntitle: [broken\n---\n# Guide\n")).toEqual({});
  });
});
