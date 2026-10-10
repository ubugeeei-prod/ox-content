import { describe, expect, it } from "vite-plus/test";
import { z } from "zod";
import { checkFrontmatter } from "./frontmatter-check";
import { failing, recorder, schema } from "./frontmatter-schema-fixtures";
import type { FrontmatterSchema, FrontmatterSchemas } from "./frontmatter-schemas";

const root = "/project/content";
const file = `${root}/posts/a.md`;

const check = (source: string, definition: FrontmatterSchema, target = file) =>
  checkFrontmatter(source, target, { "**/*.md": definition }, root);
const at = (line: number, column: number, message: string) => ({
  file,
  line,
  column,
  endLine: line,
  endColumn: column + 1,
  message,
  path: [],
});

describe("frontmatter block detection", () => {
  it.each([
    ["LF", "---\ntitle: Hello\n---\n# Body"],
    ["CRLF", "---\r\ntitle: Hello\r\n---\r\n# Body"],
    ["a byte order mark", "\uFEFF---\ntitle: Hello\n---\n"],
    ["a document end marker", "---\ntitle: Hello\n...\nbody: not yaml: [\n"],
    ["spaces and tabs after the fences", "--- \t\ntitle: Hello\n---  \nbody"],
    ["a tab after the closing fence", "---\ntitle: Hello\n---\t\nbody"],
    ["a closing fence at the end of the file", "---\ntitle: Hello\n---"],
    ["later fenced blocks in the body", "---\ntitle: Hello\n---\nbody\n---\nother: 1\n---\n"],
  ])("reads the leading block with %s", async (_, source) => {
    const seen = recorder();
    expect(await check(source, seen.schema)).toEqual({
      diagnostics: [],
      value: { title: "Hello" },
    });
    expect(seen.inputs).toEqual([{ title: "Hello" }]);
  });

  it("keeps dashes that are part of a value", async () => {
    const seen = recorder();
    await check("---\ntitle: a --- b\ntext: |\n  ---\n  still text\n---\n", seen.schema);
    expect(seen.inputs).toEqual([{ title: "a --- b", text: "---\nstill text\n" }]);
  });

  it.each([
    ["an empty block", "---\n---\n# Body"],
    ["an empty CRLF block", "---\r\n---\r\n"],
    ["a block holding only a comment", "---\n# just a comment\n---\n"],
    ["a block holding only null", "---\n~\n---\n"],
    ["a block closed by a document end marker", "---\n...\n"],
    ["no frontmatter", "# No frontmatter\n"],
    ["an empty document", ""],
    ["a thematic break after the first line", "# Title\n\n---\ntitle: x\n---\n"],
    ["a fence after a blank line", "\n---\ntitle: x\n---\n"],
    ["an indented fence", " ---\ntitle: x\n---\n"],
    ["four dashes", "----\ntitle: x\n----\n"],
    ["a lone fence without a newline", "---"],
  ])("validates an empty object for %s", async (_, source) => {
    const seen = recorder();
    expect(await check(source, seen.schema)).toEqual({ diagnostics: [], value: {} });
    expect(seen.inputs).toEqual([{}]);
  });

  it.each([
    ["a missing closing fence", "---\ntitle: Hello\n"],
    ["a missing closing fence and newline", "---\ntitle: Hello"],
    ["only an opening fence", "---\n"],
    ["a closing fence followed by a comment", "---\ntitle: x\n--- # end\n"],
    ["an indented closing fence", "---\ntitle: x\n  ---\n"],
  ])("reports %s without running the validator", async (_, source) => {
    const seen = recorder();
    expect(await check(source, seen.schema)).toEqual({
      diagnostics: [at(2, 1, "Unterminated YAML frontmatter")],
      value: undefined,
    });
    expect(seen.inputs).toEqual([]);
  });
});

describe("frontmatter YAML diagnostics", () => {
  it.each([["just text"], ["- a\n- b"], ["42"], ["true"], ["[a, b]"]])(
    "rejects the non-mapping document %j",
    async (yaml) => {
      const seen = recorder();
      expect(await check(`---\n${yaml}\n---\n`, seen.schema)).toEqual({
        diagnostics: [at(2, 1, "YAML frontmatter must be an object")],
        value: undefined,
      });
      expect(seen.inputs).toEqual([]);
    },
  );

  it("reports every duplicate key on its own line", async () => {
    const seen = recorder();
    const result = await check("---\na: 1\na: 2\nb: 1\nb: 2\n---\n", seen.schema);
    expect(result).toEqual({
      diagnostics: [at(3, 1, "Map keys must be unique"), at(5, 1, "Map keys must be unique")],
      value: undefined,
    });
    expect(seen.inputs).toEqual([]);
  });

  it("reports tab indentation on the offending line", async () => {
    const result = await check("---\na:\n\tb: 1\n---\n", recorder().schema);
    expect(result.diagnostics).toEqual([at(3, 1, "Tabs are not allowed as indentation")]);
  });

  it("reports a second document inside the block where it starts", async () => {
    const result = await check("---\ntitle: x\n--- # end\nbody: 1\n---\n", recorder().schema);
    expect(result.value).toBeUndefined();
    expect(result.diagnostics).toHaveLength(1);
    expect(result.diagnostics[0]).toMatchObject({ line: 3, column: 1, path: [] });
    expect(result.diagnostics[0].message).toContain("multiple documents");
  });

  it.each([
    ["an unclosed flow sequence", "title: ok\ntags: [a, b", /flow sequence/i],
    ["an unclosed quote", 'title: "unclosed', /closing/i],
  ])("reports %s inside the block without validating", async (_, yaml, message) => {
    const seen = recorder();
    const result = await check(`---\n${yaml}\n---\n`, seen.schema);
    expect(result.value).toBeUndefined();
    expect(seen.inputs).toEqual([]);
    expect(result.diagnostics.length).toBeGreaterThan(0);
    for (const diagnostic of result.diagnostics) {
      expect(diagnostic).toMatchObject({ file, path: [], endLine: diagnostic.line });
      expect(diagnostic.line).toBeGreaterThanOrEqual(2);
      expect(diagnostic.line).toBeLessThanOrEqual(yaml.split("\n").length + 2);
    }
    expect(result.diagnostics.some((diagnostic) => message.test(diagnostic.message))).toBe(true);
  });

  it("stops alias expansion bombs before they reach the validator", async () => {
    const level = (name: string, alias: string) =>
      `${name}: &${name} [${Array.from({ length: 10 }, () => alias).join(",")}]`;
    const source = `---\n${level("a", "x")}\n${level("b", "*a")}\n${level("c", "*b")}\nd: [*c,*c,*c]\n---\n`;
    const seen = recorder();
    const result = await check(source, seen.schema);
    expect(seen.inputs).toEqual([]);
    expect(result.value).toBeUndefined();
    expect(result.diagnostics).toHaveLength(1);
    expect(result.diagnostics[0]).toMatchObject({ line: 2, column: 1, path: [] });
    expect(result.diagnostics[0].message).toContain("Excessive alias count");
  });

  it("resolves ordinary anchors and aliases", async () => {
    const seen = recorder();
    await check("---\nbase: &base\n  name: Jane\nauthor: *base\n---\n", seen.schema);
    expect(seen.inputs).toEqual([{ base: { name: "Jane" }, author: { name: "Jane" } }]);
  });

  it("hands YAML 1.2 core scalars to the validator", async () => {
    const seen = recorder();
    await check(
      '---\nflag: yes\ndate: 2026-01-02\nhex: 0x10\nquoted: "42"\nnothing: ~\nempty:\nready: true\n---\n',
      seen.schema,
    );
    expect(seen.inputs).toEqual([
      {
        flag: "yes",
        date: "2026-01-02",
        hex: 16,
        quoted: "42",
        nothing: null,
        empty: null,
        ready: true,
      },
    ]);
  });

  it("keeps a __proto__ key as data without touching Object.prototype", async () => {
    const seen = recorder();
    await check("---\n__proto__:\n  polluted: true\ntitle: x\n---\n", seen.schema);
    const value = seen.inputs[0] as Record<string, unknown>;
    expect(Object.keys(value)).toEqual(["__proto__", "title"]);
    expect(Object.getPrototypeOf(value)).toBe(Object.prototype);
    expect(({} as Record<string, unknown>).polluted).toBeUndefined();
  });
});

describe("frontmatter validation results", () => {
  it("returns the validator output, including defaults and transforms", async () => {
    const result = await check(
      "---\ntitle: hello\ntags: a, b\n---\n",
      z.object({
        title: z.string().transform((value) => value.toUpperCase()),
        draft: z.boolean().default(false),
        tags: z.string().transform((value) => value.split(", ")),
      }),
    );
    expect(result).toEqual({
      diagnostics: [],
      value: { title: "HELLO", draft: false, tags: ["a", "b"] },
    });
  });

  it("returns the exact object an asynchronous validator resolves to", async () => {
    const output = { title: "x", render: () => "custom" };
    const result = await check(
      "---\ntitle: x\n---\n",
      schema(async () => ({ value: output })),
    );
    expect(result.diagnostics).toEqual([]);
    expect(result.value).toBe(output);
  });

  it("validates through the schema of an explicit JSON Schema adapter", async () => {
    const seen = recorder();
    const result = await check("---\ntitle: x\n---\n", { schema: seen.schema, jsonSchema: {} });
    expect(result).toEqual({ diagnostics: [], value: { title: "x" } });
    expect(seen.inputs).toEqual([{ title: "x" }]);
  });

  it.each([
    [
      "a thrown error",
      schema(() => {
        throw new Error("boom");
      }),
      "Schema validation failed: Error: boom",
    ],
    [
      "a rejected promise",
      schema(async () => {
        throw new TypeError("async boom");
      }),
      "Schema validation failed: TypeError: async boom",
    ],
    [
      "an array output",
      schema(() => ({ value: [1] })),
      "Frontmatter schemas must produce an object",
    ],
    [
      "a null output",
      schema(() => ({ value: null })),
      "Frontmatter schemas must produce an object",
    ],
    [
      "a string output",
      schema(() => ({ value: "x" })),
      "Frontmatter schemas must produce an object",
    ],
  ])("turns %s into one diagnostic at the block start", async (_, definition, message) => {
    expect(await check("---\ntitle: x\n---\n", definition)).toEqual({
      diagnostics: [at(2, 1, message)],
      value: undefined,
    });
  });

  it("uses the first matching glob for overlapping schemas", async () => {
    const schemas: FrontmatterSchemas = {
      "posts/*.md": failing({ message: "posts schema" }),
      "**/*.md": failing({ message: "fallback schema" }),
    };
    const messages = async (target: string) =>
      (await checkFrontmatter("---\ntitle: x\n---\n", target, schemas, root)).diagnostics.map(
        (diagnostic) => diagnostic.message,
      );
    expect(await messages(`${root}/posts/a.md`)).toEqual(["posts schema"]);
    expect(await messages(`${root}/posts/deep/a.md`)).toEqual(["fallback schema"]);
    expect(await messages(`${root}/guides/a.md`)).toEqual(["fallback schema"]);
  });

  it.each([
    ["outside the source root", "/elsewhere/posts/a.md"],
    ["with an unmatched extension", `${root}/posts/a.txt`],
    ["in a sibling directory sharing the root prefix", `${root}-old/posts/a.md`],
  ])("skips files %s without parsing them", async (_, target) => {
    const seen = recorder();
    expect(await check("---\ntitle: [broken\n", seen.schema, target)).toEqual({
      diagnostics: [],
      value: undefined,
    });
    expect(seen.inputs).toEqual([]);
  });

  it("skips every file when no schemas are configured", async () => {
    expect(await checkFrontmatter("---\ntitle: [broken\n", file, undefined, root)).toEqual({
      diagnostics: [],
      value: undefined,
    });
    expect(await checkFrontmatter("---\ntitle: [broken\n", file, {}, root)).toEqual({
      diagnostics: [],
      value: undefined,
    });
  });

  it("reports the file path exactly as it was given", async () => {
    const result = await checkFrontmatter(
      "---\ntitle: 1\n---\n",
      "content/posts/a.md",
      { "posts/*.md": failing({ message: "Expected string", path: ["title"] }) },
      "content",
    );
    expect(result.diagnostics).toEqual([
      {
        file: "content/posts/a.md",
        line: 2,
        column: 8,
        endLine: 2,
        endColumn: 9,
        message: "Expected string",
        path: ["title"],
      },
    ]);
  });
});
