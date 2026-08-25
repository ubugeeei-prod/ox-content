import { describe, expect, it } from "vite-plus/test";
import { readingTimeMinutes, resolveBlogCollectionName, resolveBlogOptions } from "./blog";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { resolveSsgOptions } from "./ssg";

describe("resolveBlogOptions", () => {
  it("disables the feature when omitted or false", () => {
    expect(resolveBlogOptions(undefined)).toEqual({
      enabled: false,
      authors: {},
      pageSize: 10,
    });
    expect(resolveBlogOptions(false)).toEqual({
      enabled: false,
      authors: {},
      pageSize: 10,
    });
    expect(createDocsResolvedOptions().blog).toBeUndefined();
    expect(resolveSsgOptions(undefined).blog?.enabled).toBe(false);
    expect(resolveSsgOptions(true).blog?.enabled).toBe(false);
    expect(resolveSsgOptions({}).blog?.enabled).toBe(false);
  });

  it("enables defaults when true, and overrides only set fields", () => {
    expect(resolveBlogOptions(true)).toEqual({
      enabled: true,
      authors: {},
      pageSize: 10,
    });
    expect(resolveSsgOptions({ blog: true }).blog).toEqual({
      enabled: true,
      authors: {},
      pageSize: 10,
    });
    expect(
      resolveBlogOptions({
        collection: "posts",
        pageSize: 5,
        authors: { ada: { name: "Ada", bio: "Writer", url: "https://example.com/ada" } },
      }),
    ).toEqual({
      enabled: true,
      collection: "posts",
      pageSize: 5,
      authors: { ada: { name: "Ada", bio: "Writer", url: "https://example.com/ada" } },
    });
    expect(resolveBlogOptions({})).toEqual({
      enabled: true,
      authors: {},
      pageSize: 10,
    });
    expect(resolveBlogOptions({ pageSize: 0 }).pageSize).toBe(10);
    expect(resolveBlogOptions({ pageSize: -3 }).pageSize).toBe(10);
  });
});

describe("resolveBlogCollectionName", () => {
  it("picks blog, else the only collection, else requires an explicit name", () => {
    expect(resolveBlogCollectionName(undefined, ["blog", "docs"])).toBe("blog");
    expect(resolveBlogCollectionName(undefined, ["posts"])).toBe("posts");
    expect(resolveBlogCollectionName(undefined, ["docs", "notes"])).toBeUndefined();
    expect(resolveBlogCollectionName("docs", ["blog", "docs"])).toBe("docs");
    expect(resolveBlogCollectionName(undefined, [])).toBeUndefined();
    expect(resolveBlogCollectionName("blog", [])).toBe("blog");
  });
});

const twoHundredWords = Array.from({ length: 200 }, () => "word").join(" ");

describe("readingTimeMinutes", () => {
  it("uses latin words / 200 + CJK chars / 500, ceiled, same input same minutes", () => {
    expect(readingTimeMinutes("")).toBe(0);
    expect(readingTimeMinutes("---\ntitle: word word word\n---\n\n")).toBe(0);
    expect(readingTimeMinutes(twoHundredWords)).toBe(1);
    expect(readingTimeMinutes(`${twoHundredWords} extra`)).toBe(2);
    expect(readingTimeMinutes("あ".repeat(500))).toBe(1);
    expect(readingTimeMinutes(`${"あ".repeat(500)}い`)).toBe(2);
    expect(readingTimeMinutes("Hello 世界")).toBe(1);

    const mixed = `${twoHundredWords}\n${"漢".repeat(500)}`;
    expect(readingTimeMinutes(mixed)).toBe(2);
    expect(readingTimeMinutes(mixed)).toBe(readingTimeMinutes(mixed));
    expect(readingTimeMinutes(`${twoHundredWords} extra`)).toBe(
      readingTimeMinutes(`${twoHundredWords} extra`),
    );
  });

  it("ignores fenced blocks, unclosed fences, and inline code spans", () => {
    const prose = "alpha beta";
    const fenced = `${prose}\n\`\`\`\n${twoHundredWords}\n\`\`\`\n`;
    const unclosed = `${prose}\n\`\`\`\n${twoHundredWords}`;
    const spanned = "alpha `word word word` beta";
    expect(readingTimeMinutes(fenced)).toBe(readingTimeMinutes(prose));
    expect(readingTimeMinutes(unclosed)).toBe(readingTimeMinutes(prose));
    expect(readingTimeMinutes(spanned)).toBe(readingTimeMinutes(prose));
  });
});
