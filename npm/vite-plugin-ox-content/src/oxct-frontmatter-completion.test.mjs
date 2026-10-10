import { describe, expect, it } from "vite-plus/test";
import { frontmatterCompletion, frontmatterHover } from "../bin/oxct-frontmatter-completion.mjs";

const shape = {
  type: "object",
  required: ["title", "kind"],
  properties: {
    title: {
      type: "string",
      title: "Title",
      description: "Display title",
      examples: ["Hello", "World"],
    },
    kind: { enum: ["post", "page"], description: "Kind" },
    draft: { type: "boolean", default: false },
    order: { type: ["integer", "null"], default: 10 },
    fixed: { const: "always" },
    tags: { type: "array", items: { type: "string" } },
    author: { $ref: "#/$defs/author" },
    contributors: { type: "array", items: { $ref: "#/$defs/author" } },
    merged: {
      allOf: [
        { properties: { one: { type: "string" } }, required: ["one"] },
        { properties: { two: { type: "number" } } },
      ],
    },
    choice: { oneOf: [{ const: "x" }, { enum: ["y", "z"] }, { const: "x" }] },
    mixed: { enum: [3, null, true, "two words", "3", "null", "true"] },
    escaped: { $ref: "#/$defs/a~1b~0c" },
    loop: { $ref: "#/$defs/loop" },
    missing: { $ref: "#/$defs/nope" },
    remote: { $ref: "https://example.com/schema.json" },
    plain: {},
  },
  $defs: {
    author: {
      type: "object",
      required: ["name"],
      properties: {
        name: { type: "string", description: "Full name" },
        role: { enum: ["writer", "reviewer"] },
        social: { type: "object", properties: { twitter: { type: "string" } } },
      },
    },
    "a/b~c": { enum: ["escaped"] },
    loop: { $ref: "#/$defs/loop" },
  },
};
const complete = (text, line, character, schema = shape) =>
  frontmatterCompletion(schema, text, { line, character });
const labels = (text, line, character, schema) =>
  complete(text, line, character, schema).map((item) => item.label);

describe("frontmatter key completion", () => {
  it("lists schema properties in declaration order with required fields sorted first", () => {
    const items = complete("---\n\n---\n", 1, 0);
    expect(items.map((item) => item.label)).toEqual(Object.keys(shape.properties));
    expect(items.every((item) => item.kind === 10)).toBe(true);
    expect(items.filter((item) => item.detail === "Required frontmatter field")).toMatchObject([
      { label: "title", sortText: "0title" },
      { label: "kind", sortText: "0kind" },
    ]);
    expect(items.find((item) => item.label === "draft")).toMatchObject({
      detail: "Frontmatter field",
      sortText: "1draft",
    });
  });

  it.each([
    ["title", "title: ", "Title\n\nDisplay title\n\nType: string"],
    ["kind", "kind: ", 'Kind\n\nValues: "post", "page"'],
    ["draft", "draft: false", "Type: boolean"],
    ["order", "order: 10", "Type: integer | null"],
    ["author", "author: ", "Type: object"],
    ["choice", "choice: ", 'Values: "x", "y", "z"'],
    ["escaped", "escaped: ", 'Values: "escaped"'],
    ["plain", "plain: ", ""],
  ])("inserts %s with its default and documents it", (label, newText, documentation) => {
    const item = complete("---\n\n---\n", 1, 0).find((entry) => entry.label === label);
    expect(item).toMatchObject({ documentation, textEdit: { newText } });
  });

  it.each([
    ["a typed prefix", "---\nt\n---\n", 1, 1, ["title", "tags"]],
    ["keys that are already present", "---\ntitle: x\ntags: []\nt\n---\n", 3, 1, []],
    ["the key being retyped", "---\ntitle: x\ntitle\n---\n", 2, 5, ["title"]],
    ["a prefix nothing starts with", "---\nzz\n---\n", 1, 2, []],
    ["keys despite incomplete YAML elsewhere", "---\ntags: [a\nti\n---\n", 2, 2, ["title"]],
    ["referenced object properties", "---\nauthor:\n  \n---\n", 2, 2, ["name", "role", "social"]],
    [
      "missing nested properties only",
      "---\nauthor:\n  name: x\n  \n---\n",
      3,
      2,
      ["role", "social"],
    ],
    [
      "nested keys after an inline comment",
      "---\nauthor:\n  name: x # c\n  \n---\n",
      3,
      2,
      ["role", "social"],
    ],
    ["properties two levels down", "---\nauthor:\n  social:\n    \n---\n", 3, 4, ["twitter"]],
    ["properties merged from allOf", "---\nmerged:\n  \n---\n", 2, 2, ["one", "two"]],
    [
      "properties of sequence items",
      "---\ncontributors:\n  - \n---\n",
      2,
      4,
      ["name", "role", "social"],
    ],
    ["nothing under an unknown parent", "---\nnope:\n  \n---\n", 2, 2, []],
    ["nothing under a scalar property", "---\ndraft:\n  \n---\n", 2, 2, []],
    ["nothing for a self-referencing definition", "---\nloop:\n  \n---\n", 2, 2, []],
  ])("offers %s", (_, text, line, character, expected) => {
    expect(labels(text, line, character)).toEqual(expected);
  });

  it("marks required fields of nested and merged objects", () => {
    expect(complete("---\nauthor:\n  \n---\n", 2, 2)[0]).toMatchObject({
      label: "name",
      detail: "Required frontmatter field",
      sortText: "0name",
      documentation: "Full name\n\nType: string",
    });
    expect(complete("---\nmerged:\n  \n---\n", 2, 2).map((item) => item.detail)).toEqual([
      "Required frontmatter field",
      "Frontmatter field",
    ]);
  });

  it("replaces only the typed prefix and keeps the indentation", () => {
    expect(complete("---\n  ti\n---\n", 1, 4)[0].textEdit).toEqual({
      range: { start: { line: 1, character: 2 }, end: { line: 1, character: 4 } },
      newText: "title: ",
    });
    expect(complete("---\ncontributors:\n  - na\n---\n", 2, 6)[0].textEdit).toEqual({
      range: { start: { line: 2, character: 4 }, end: { line: 2, character: 6 } },
      newText: "name: ",
    });
  });

  it.each([[null], [true], ["schema"], [{}], [{ properties: {} }]])(
    "offers nothing for the unusable schema %j",
    (schema) => {
      expect(frontmatterCompletion(schema, "---\nti\n---\n", { line: 1, character: 2 })).toEqual(
        [],
      );
    },
  );
});

describe("frontmatter value completion", () => {
  it.each([
    ["enum members", "---\nkind: \n---\n", 1, 6, ["post", "page"]],
    ["enum members right after the colon", "---\nkind:\n---\n", 1, 5, ["post", "page"]],
    ["enum members behind a quoted key", '---\n"kind": \n---\n', 1, 8, ["post", "page"]],
    [
      "every enum member whatever was typed",
      "---\nauthor:\n  role: w\n---\n",
      2,
      9,
      ["writer", "reviewer"],
    ],
    [
      "enum members inside a sequence item",
      "---\ncontributors:\n  - role: \n---\n",
      2,
      10,
      ["writer", "reviewer"],
    ],
    ["a const", "---\nfixed: \n---\n", 1, 7, ["always"]],
    ["a default", "---\norder: \n---\n", 1, 7, ["10"]],
    ["examples", "---\ntitle: \n---\n", 1, 7, ["Hello", "World"]],
    ["a boolean default before the other literal", "---\ndraft: \n---\n", 1, 7, ["false", "true"]],
    ["deduplicated union members", "---\nchoice: \n---\n", 1, 8, ["x", "y", "z"]],
    ["members behind an escaped JSON pointer", "---\nescaped: \n---\n", 1, 9, ["escaped"]],
    ["nothing for a self-referencing definition", "---\nloop: \n---\n", 1, 6, []],
    ["nothing for a dangling reference", "---\nmissing: \n---\n", 1, 9, []],
    ["nothing for a remote reference", "---\nremote: \n---\n", 1, 8, []],
    ["nothing for an unconstrained property", "---\nplain: \n---\n", 1, 7, []],
    ["nothing for an unknown property", "---\nnope: \n---\n", 1, 6, []],
  ])("offers %s", (_, text, line, character, expected) => {
    expect(labels(text, line, character)).toEqual(expected);
  });

  it("writes values as YAML scalars that parse back to the same type", () => {
    expect(labels("---\nmixed: \n---\n", 1, 7)).toEqual([
      "3",
      "null",
      "true",
      "two words",
      '"3"',
      '"null"',
      '"true"',
    ]);
  });

  it("describes each value and replaces everything typed after the colon", () => {
    const items = complete("---\nkind:    po\n---\n", 1, 11);
    expect(items).toEqual(
      ["post", "page"].map((value) => ({
        label: value,
        kind: 12,
        documentation: 'Kind\n\nValues: "post", "page"',
        textEdit: {
          range: { start: { line: 1, character: 9 }, end: { line: 1, character: 11 } },
          newText: value,
        },
      })),
    );
  });
});

describe("frontmatter hover", () => {
  it.each([
    ["a key", "---\ntitle: Hello\n---\n", 1, 2, "title\n\nTitle\n\nDisplay title\n\nType: string"],
    [
      "a value",
      "---\ntitle: Hello\n---\n",
      1,
      9,
      "title\n\nTitle\n\nDisplay title\n\nType: string",
    ],
    [
      "a value containing a colon",
      "---\ntitle: http://x\n---\n",
      1,
      14,
      "title\n\nTitle\n\nDisplay title\n\nType: string",
    ],
    ["an enum", "---\nkind: post\n---\n", 1, 0, 'kind\n\nKind\n\nValues: "post", "page"'],
    ["a union of types", "---\norder: 1\n---\n", 1, 0, "order\n\nType: integer | null"],
    ["a referenced object", "---\nauthor:\n  name: Jane\n---\n", 1, 2, "author\n\nType: object"],
    [
      "a nested field",
      "---\nauthor:\n  name: Jane\n---\n",
      2,
      3,
      "name\n\nFull name\n\nType: string",
    ],
    [
      "a nested field with CRLF",
      "---\r\nauthor:\r\n  name: Jane\r\n---\r\n",
      2,
      3,
      "name\n\nFull name\n\nType: string",
    ],
    [
      "the first key of a sequence item",
      "---\ncontributors:\n  - name: Jane\n---\n",
      2,
      5,
      "name\n\nFull name\n\nType: string",
    ],
    [
      "a later key of a sequence item",
      "---\ncontributors:\n  - name: Jane\n    role: writer\n---\n",
      3,
      5,
      'role\n\nValues: "writer", "reviewer"',
    ],
  ])("describes %s", (_, text, line, character, expected) => {
    expect(frontmatterHover(shape, text, { line, character })).toEqual({
      contents: { kind: "plaintext", value: expected },
    });
  });

  it.each([
    ["an undocumented property", "---\nplain: 1\n---\n", 1, 0],
    ["an unknown property", "---\nnope: 1\n---\n", 1, 0],
    ["a blank line", "---\n\ntitle: x\n---\n", 1, 0],
  ])("answers null for %s so the client shows nothing", (_, text, line, character) => {
    expect(frontmatterHover(shape, text, { line, character })).toBeNull();
  });

  it.each([
    ["the opening fence", "---\ntitle: x\n---\n", 0, 1],
    ["the closing fence", "---\ntitle: x\n---\n", 2, 1],
    ["the Markdown body", "---\ntitle: x\n---\ntitle: body", 3, 2],
    ["a document without frontmatter", "title: x\n", 0, 2],
  ])("leaves %s to the native server", (_, text, line, character) => {
    expect(frontmatterHover(shape, text, { line, character })).toBeUndefined();
  });
});
