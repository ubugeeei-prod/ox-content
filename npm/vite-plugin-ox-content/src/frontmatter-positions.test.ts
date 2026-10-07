import type { StandardSchemaV1 } from "@standard-schema/spec";
import { describe, expect, it } from "vite-plus/test";
import { z } from "zod";
import { checkFrontmatter, formatFrontmatterDiagnostics } from "./frontmatter-check";
import { failing } from "./frontmatter-schema-fixtures";
import type { FrontmatterSchema } from "./frontmatter-schemas";

const root = "/project/content";
const file = `${root}/posts/a.md`;
const diagnose = async (source: string, definition: FrontmatterSchema) =>
  (await checkFrontmatter(source, file, { "**/*.md": definition }, root)).diagnostics;
/** The position reported for one issue path. */
async function locate(source: string, path?: StandardSchemaV1.Issue["path"]) {
  const [diagnostic, ...rest] = await diagnose(source, failing({ message: "Invalid", path }));
  expect(rest).toEqual([]);
  expect(diagnostic).toMatchObject({ file, message: "Invalid" });
  expect([diagnostic.endLine, diagnostic.endColumn]).toEqual([
    diagnostic.line,
    diagnostic.column + 1,
  ]);
  return [diagnostic.line, diagnostic.column];
}

const nested = [
  "---", // 1
  "title: x", // 2
  "author:", // 3
  "  name: 42", // 4
  "  links:", // 5
  "    - https://a", // 6
  "    - 7", // 7
  "tags:", // 8
  "  - one", // 9
  "  - 2", // 10
  "---",
  "",
].join("\n");

describe("frontmatter issue positions", () => {
  it.each<[string, string, PropertyKey[], number, number]>([
    ["a top-level value", "---\ntitle: Hello\nrating: wrong\n---\n", ["rating"], 3, 9],
    ["a nested value", nested, ["author", "name"], 4, 9],
    ["an item of a nested sequence", nested, ["author", "links", 1], 7, 7],
    ["an item of a top-level sequence", nested, ["tags", 1], 10, 5],
    ["a mapping, at its first entry", nested, ["author"], 4, 3],
    ["a sequence, at its first item", nested, ["tags"], 9, 3],
    [
      "a flow mapping value",
      "---\nauthor: { name: 42, tags: [a, 7] }\n---\n",
      ["author", "name"],
      2,
      17,
    ],
    [
      "a flow sequence item",
      "---\nauthor: { name: 42, tags: [a, 7] }\n---\n",
      ["author", "tags", 1],
      2,
      31,
    ],
    [
      "a sequence item addressed by a string index",
      "---\ntags: [a, b]\n---\n",
      ["tags", "1"],
      2,
      11,
    ],
    ["a value behind a quoted key", "---\n\"title\": 'x'\n---\n", ["title"], 2, 10],
    [
      "a block scalar, at its indicator",
      "---\nblock: |\n  line one\n  line two\n---\n",
      ["block"],
      2,
      8,
    ],
    ["an empty value, after the colon", "---\ntitle: x\nempty:\n---\n", ["empty"], 3, 7],
    ["a value in an indented root mapping", "---\n  title: x\n  n: 1\n---\n", ["n"], 3, 6],
    ["a value behind a numeric key", "---\n1: one\n---\n", [1], 2, 4],
    [
      "a value behind a key with spaces",
      "---\nlinks:\n  Bad Key: x\n---\n",
      ["links", "Bad Key"],
      3,
      12,
    ],
    ["the last line of the block", "---\na: 1\nb: 2\nc: 3\n---\n", ["c"], 4, 4],
    ["a value followed by trailing spaces", "---\ntitle: x   \n   \n---\n", ["title"], 2, 8],
    [
      "an alias, at the alias itself",
      "---\nbase: &base\n  name: 1\nauthor: *base\n---\n",
      ["author"],
      4,
      9,
    ],
  ])("points at %s", async (_, source, path, line, column) => {
    expect(await locate(source, path)).toEqual([line, column]);
  });

  it.each<[string, string, PropertyKey[], number, number]>([
    ["a byte order mark", "\uFEFF---\ntitle: Hello\n---\n", ["title"], 2, 8],
    ["CRLF line endings", "---\r\ntitle: Hello\r\nrating: x\r\n---\r\n", ["rating"], 3, 9],
    ["a byte order mark and CRLF", "\uFEFF---\r\na:\r\n  b: 1\r\n---\r\n", ["a", "b"], 3, 6],
    ["a CRLF document end marker", "---\r\ntitle: x\r\n...\r\nbody", ["title"], 2, 8],
    ["spaces after the opening fence", "---   \ntitle: x\n---\n", ["title"], 2, 8],
  ])("keeps positions stable with %s", async (_, source, path, line, column) => {
    expect(await locate(source, path)).toEqual([line, column]);
  });

  it("counts columns in UTF-16 code units like editors do", async () => {
    const source = "---\n日本語: 値\ntitle: 日本語 👩‍💻\nemoji👩‍💻: bad\n---\n";
    expect(await locate(source, ["日本語"])).toEqual([2, 6]);
    expect(await locate(source, ["title"])).toEqual([3, 8]);
    expect(await locate(source, ["emoji👩‍💻"])).toEqual([4, 13]);
  });

  it.each<[string, StandardSchemaV1.Issue["path"]]>([
    ["a missing top-level field", ["missing"]],
    ["the document root", []],
    ["an issue without a path", undefined],
    ["a path holding only a symbol", [Symbol("brand")]],
  ])("falls back to the first frontmatter line for %s", async (_, path) => {
    expect(await locate(nested, path)).toEqual([2, 1]);
    expect(await locate("---\n---\n", path)).toEqual([2, 1]);
    expect(await locate("---\r\n# comment\r\n---\r\n", path)).toEqual([2, 1]);
  });

  it.each([["# No frontmatter\n"], ["\uFEFF# No frontmatter\n"], [""], ["\n---\ntitle: x\n---\n"]])(
    "points at the start of a document without frontmatter: %j",
    async (source) => {
      expect(await locate(source, ["title"])).toEqual([1, 1]);
      expect(await locate(source, [])).toEqual([1, 1]);
    },
  );

  it("unwraps path segment objects and skips symbols when locating", async () => {
    const brand = Symbol("brand");
    const diagnostics = await diagnose(
      nested,
      failing(
        { message: "segments", path: [{ key: "author" }, { key: "name" }] },
        { message: "mixed", path: ["tags", { key: 1 }] },
        { message: "symbol", path: ["author", brand, "name"] },
      ),
    );
    expect(diagnostics.map(({ line, column, path }) => ({ line, column, path }))).toEqual([
      { line: 4, column: 9, path: ["author", "name"] },
      { line: 10, column: 5, path: ["tags", 1] },
      { line: 4, column: 9, path: ["author", brand, "name"] },
    ]);
  });

  it("keeps issues in validator order instead of sorting them by position", async () => {
    const diagnostics = await diagnose(
      "---\na: 1\nb: 2\nc: 3\n---\n",
      failing(
        { message: "third", path: ["c"] },
        { message: "first", path: ["a"] },
        { message: "root" },
        { message: "second", path: ["b"] },
      ),
    );
    expect(diagnostics.map(({ message, line }) => [message, line])).toEqual([
      ["third", 4],
      ["first", 2],
      ["root", 2],
      ["second", 3],
    ]);
  });

  it("maps every issue of a real validator onto the YAML it describes", async () => {
    const source = [
      "---", // 1
      "title: 1", // 2
      "author:", // 3
      "  name: 2", // 4
      "tags:", // 5
      "  - ok", // 6
      "  - 3", // 7
      "links:", // 8
      "  Bad Key: x", // 9
      "extra: 1", // 10
      "---",
      "",
    ].join("\n");
    const diagnostics = await diagnose(
      source,
      z.strictObject({
        title: z.string(),
        author: z.object({ name: z.string() }),
        tags: z.array(z.string()),
        links: z.record(z.string().regex(/^[a-z]+$/), z.string()),
        level: z.enum(["beginner", "advanced"]),
      }),
    );
    const located = diagnostics.map(
      ({ path, line, column }) => `${path.map(String).join(".")}@${line}:${column}`,
    );
    expect(located.sort()).toEqual(
      [
        "title@2:8",
        "author.name@4:9",
        "tags.1@7:5",
        "links.Bad Key@9:12",
        "level@2:1", // missing field
        "@2:1", // unrecognized key reported on the root object
      ].sort(),
    );
    for (const diagnostic of diagnostics) expect(diagnostic.message).not.toBe("");
  });
});

describe("formatFrontmatterDiagnostics", () => {
  const diagnostic = (
    target: string,
    line: number,
    column: number,
    message: string,
    path: PropertyKey[],
  ) => ({
    file: target,
    line,
    column,
    endLine: line,
    endColumn: column + 1,
    message,
    path,
  });

  it("returns an empty string for no diagnostics", () => {
    expect(formatFrontmatterDiagnostics([])).toBe("");
  });

  it.each<[string, PropertyKey[], string]>([
    ["a root issue", [], "a.md:2:8 Bad"],
    ["a field", ["title"], "a.md:2:8 title: Bad"],
    ["a nested field", ["author", "name"], "a.md:2:8 author.name: Bad"],
    ["a sequence index", ["tags", 1, "name"], "a.md:2:8 tags.1.name: Bad"],
    ["a symbol segment", [Symbol("id"), 0], "a.md:2:8 Symbol(id).0: Bad"],
  ])("formats %s as file:line:column", (_, path, expected) => {
    expect(formatFrontmatterDiagnostics([diagnostic("a.md", 2, 8, "Bad", path)])).toBe(expected);
  });

  it("joins diagnostics with newlines and no trailing newline", () => {
    expect(
      formatFrontmatterDiagnostics([
        diagnostic("a.md", 2, 8, "First", ["title"]),
        diagnostic("b.md", 1, 1, "Second", []),
        diagnostic("a.md", 10, 12, "Third", ["tags", 0]),
      ]),
    ).toBe("a.md:2:8 title: First\nb.md:1:1 Second\na.md:10:12 tags.0: Third");
  });

  it("formats checker output into a clickable location", async () => {
    const diagnostics = await diagnose(
      "---\ntitle: Hello\nrating: wrong\n---\n",
      failing({ message: "Expected number", path: ["rating"] }, { message: "Unknown field" }),
    );
    expect(formatFrontmatterDiagnostics(diagnostics)).toBe(
      `${file}:3:9 rating: Expected number\n${file}:2:1 Unknown field`,
    );
  });
});
