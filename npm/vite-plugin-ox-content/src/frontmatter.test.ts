import { describe, expect, it } from "vitest";
import { parseFrontmatter, stringifyFrontmatter } from "./frontmatter";

describe("parseFrontmatter", () => {
  it("splits frontmatter from the body", () => {
    expect(parseFrontmatter("---\ntitle: Post\ndraft: false\n---\nBody\n")).toEqual({
      frontmatter: { title: "Post", draft: false },
      content: "Body\n",
    });
  });

  it("returns no keys for a source without frontmatter", () => {
    expect(parseFrontmatter("Body only\n")).toEqual({ frontmatter: {}, content: "Body only\n" });
  });
});

describe("stringifyFrontmatter", () => {
  it("emits a frontmatter block before the body", () => {
    expect(stringifyFrontmatter({ title: "Post" }, "Body\n")).toBe("---\ntitle: Post\n---\nBody\n");
  });

  it("keeps a date-like string quoted so it parses back as a string", () => {
    const source = stringifyFrontmatter({ date: "2026-09-09" });
    expect(source).toBe("---\ndate: 2026-09-09\n---\n");
    expect(parseFrontmatter(source).frontmatter.date).toBe("2026-09-09");
  });

  it("preserves key order", () => {
    expect(stringifyFrontmatter({ permalink: "/blog/post", title: "Post" })).toBe(
      "---\npermalink: /blog/post\ntitle: Post\n---\n",
    );
  });

  it("omits the block entirely when there are no keys", () => {
    expect(stringifyFrontmatter({}, "Body\n")).toBe("Body\n");
  });

  it("round-trips through parseFrontmatter", () => {
    const frontmatter = { permalink: "/blog/post", title: "Post", draft: false, tags: ["a", "b"] };
    const parsed = parseFrontmatter(stringifyFrontmatter(frontmatter, "Body\n"));
    expect(parsed).toEqual({ frontmatter, content: "Body\n" });
  });
});
