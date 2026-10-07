import { describe, expect, it } from "vite-plus/test";
import { frontmatterContext } from "../bin/oxct-frontmatter-completion.mjs";

describe("frontmatterContext", () => {
  it.each([
    ["the opening fence", "---\ntitle: x\n---\n", 0, 1],
    ["the closing fence", "---\ntitle: x\n---\n", 2, 0],
    ["a document end marker", "---\ntitle: x\n...\nbody", 2, 0],
    ["the Markdown body", "---\ntitle: x\n---\nbody", 3, 1],
    ["a document without frontmatter", "# Title\n", 0, 1],
    ["a thematic break later in the body", "# Title\n\n---\nti\n---\n", 3, 2],
    ["a block opened by four dashes", "----\nti\n----\n", 1, 2],
    ["a negative line", "---\nti\n---\n", -1, 0],
  ])("is undefined on %s", (_, text, line, character) => {
    expect(frontmatterContext(text, { line, character })).toBeUndefined();
  });

  it.each([
    ["a key prefix", "---\nti\n---\n", 1, 2, { path: [], key: "ti", value: false, start: 0 }],
    [
      "an unterminated block",
      "---\ntitle: x\nti",
      2,
      2,
      { path: [], key: "ti", value: false, start: 0 },
    ],
    [
      "trailing spaces on the fence",
      "---  \nti\n---\n",
      1,
      2,
      { path: [], key: "ti", value: false, start: 0 },
    ],
    [
      "a value after extra spaces",
      "---\ntitle:   Hel\n---\n",
      1,
      12,
      { path: [], key: "title", value: true, start: 9 },
    ],
    [
      "a value right after the colon",
      "---\ntitle:\n---\n",
      1,
      6,
      { path: [], key: "title", value: true, start: 6 },
    ],
    [
      "a quoted key",
      '---\n"title": \n---\n',
      1,
      9,
      { path: [], key: "title", value: true, start: 9 },
    ],
    [
      "a partially quoted key",
      "---\n'ti\n---\n",
      1,
      3,
      { path: [], key: "ti", value: false, start: 0 },
    ],
    [
      "a nested key",
      "---\nauthor:\n  social:\n    tw\n---\n",
      3,
      6,
      { path: ["author", "social"], key: "tw", value: false, start: 4 },
    ],
    [
      "a byte order mark and CRLF",
      "\uFEFF---\r\nauthor:\r\n  na\r\n---\r\n",
      2,
      4,
      { path: ["author"], key: "na", value: false, start: 2 },
    ],
    [
      "a dedent back to the root",
      "---\nauthor:\n  name: x\nti\n---\n",
      3,
      2,
      { path: [], key: "ti", value: false, start: 0 },
    ],
    [
      "a dedent to an outer mapping",
      "---\na:\n  b:\n    c: 1\n  d\n---\n",
      4,
      3,
      { path: ["a"], key: "d", value: false, start: 2 },
    ],
    [
      "comments and blank lines",
      "---\nauthor:\n  # note\n\n  na\n---\n",
      4,
      4,
      { path: ["author"], key: "na", value: false, start: 2 },
    ],
    [
      "a parent with a trailing comment",
      "---\nauthor: # who\n  na\n---\n",
      2,
      4,
      { path: ["author"], key: "na", value: false, start: 2 },
    ],
    [
      "a sibling after a scalar",
      "---\ntitle: x\nauthor:\n  name: y\n  ro\n---\n",
      4,
      4,
      { path: ["author"], key: "ro", value: false, start: 2 },
    ],
    [
      "a dotted parent key",
      "---\nog.image:\n  wi\n---\n",
      2,
      4,
      { path: ["og.image"], key: "wi", value: false, start: 2 },
    ],
    [
      "an empty sequence item",
      "---\ntags:\n  - \n---\n",
      2,
      4,
      { path: ["tags", 0], key: "", value: false, start: 4 },
    ],
    [
      "a key in a sequence item",
      "---\ncontributors:\n  - na\n---\n",
      2,
      6,
      { path: ["contributors", 0], key: "na", value: false, start: 4 },
    ],
    [
      "a value in a sequence item",
      "---\ncontributors:\n  - role: \n---\n",
      2,
      10,
      { path: ["contributors", 0], key: "role", value: true, start: 10 },
    ],
    [
      "a mapping nested in a sequence item",
      "---\ncontributors:\n  - links:\n      ho\n---\n",
      3,
      8,
      { path: ["contributors", 0, "links"], key: "ho", value: false, start: 6 },
    ],
    [
      "a character past the end of the line",
      "---\nti\n---\n",
      1,
      99,
      { path: [], key: "ti", value: false, start: 0 },
    ],
  ])("locates %s", (_, text, line, character, expected) => {
    expect(frontmatterContext(text, { line, character })).toMatchObject({
      ...expected,
      line,
      character,
    });
  });

  it("exposes only the YAML between the fences", () => {
    const position = { line: 2, character: 1 };
    expect(frontmatterContext("---\na: 1\nb\n---\nbody", position).yaml).toBe("a: 1\nb");
    expect(frontmatterContext("---\r\na: 1\r\nb\r\n---\r\nbody", position).yaml).toBe("a: 1\nb");
    expect(frontmatterContext("---\na: 1\nb", position).yaml).toBe("a: 1\nb");
  });
});
