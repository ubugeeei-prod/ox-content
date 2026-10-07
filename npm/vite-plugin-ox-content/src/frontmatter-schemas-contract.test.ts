import { describe, expect, it } from "vite-plus/test";
import { hybrid, schema } from "./frontmatter-schema-fixtures";
import {
  defineFrontmatterSchemas,
  selectFrontmatterSchema,
  validator,
  type FrontmatterSchema,
} from "./frontmatter-schemas";
import { resolveOptions } from "./resolve-options";

const accept = schema();
const unsafe = (value: unknown) => value as FrontmatterSchema;

describe("defineFrontmatterSchemas", () => {
  it.each([[""], ["/abs/*.md"], ["../x.md"], ["a/../b.md"], ["a/.."], ["posts/**/../*.md"]])(
    "rejects the glob %j because it can leave srcDir",
    (glob) => {
      expect(() => defineFrontmatterSchemas({ [glob]: accept })).toThrow(
        `Frontmatter schema globs must be relative to srcDir: ${glob}`,
      );
    },
  );

  it.each([
    ["a..b/*.md"],
    ["..a/*.md"],
    ["./a.md"],
    ["**"],
    ["!posts/**"],
    ["{posts,guides}/*.{md,mdx}"],
    ["記事/**/*.md"],
  ])("accepts the source-relative glob %j", (glob) => {
    const schemas = { [glob]: accept };
    expect(defineFrontmatterSchemas(schemas)).toBe(schemas);
  });

  it.each([
    [
      "a future Standard Schema version",
      { "~standard": { version: 2, vendor: "x", validate() {} } },
    ],
    ["a missing validate function", { "~standard": { version: 1, vendor: "x" } }],
    ["a non-function validate", { "~standard": { version: 1, vendor: "x", validate: true } }],
    ["an object without the standard property", {}],
    ["an adapter around an invalid schema", { schema: {}, jsonSchema: {} }],
  ])("rejects %s", (_, definition) => {
    expect(() => defineFrontmatterSchemas({ "*.md": unsafe(definition) })).toThrow(
      "Invalid Standard Schema for *.md",
    );
  });

  it("names the first invalid entry and keeps valid entries untouched", () => {
    expect(() =>
      defineFrontmatterSchemas({ "posts/*.md": accept, "guides/*.md": unsafe({}) }),
    ).toThrow("Invalid Standard Schema for guides/*.md");
    const empty = {};
    expect(defineFrontmatterSchemas(empty)).toBe(empty);
  });

  it("accepts adapters and validators that carry extra properties", () => {
    const schemas = {
      "posts/*.md": { schema: accept, jsonSchema: { type: "object" } },
      "guides/*.md": hybrid().definition,
    };
    expect(defineFrontmatterSchemas(schemas)).toBe(schemas);
  });

  it("is applied when plugin options are resolved", () => {
    const schemas = { "posts/*.md": accept };
    expect(resolveOptions({ frontmatterSchemas: schemas }).frontmatterSchemas).toBe(schemas);
    expect(resolveOptions({}).frontmatterSchemas).toBeUndefined();
    expect(() => resolveOptions({ frontmatterSchemas: { "../outside.md": accept } })).toThrow(
      "relative to srcDir",
    );
  });
});

describe("validator", () => {
  it("unwraps adapters and returns validators as they are", () => {
    expect(validator({ schema: accept, jsonSchema: {} })).toBe(accept);
    expect(validator(accept)).toBe(accept);
    const own = hybrid().definition;
    expect(validator(own)).toBe(own);
  });
});

describe("selectFrontmatterSchema", () => {
  const select = (globs: string[], file: string, root = "/p") =>
    selectFrontmatterSchema(
      Object.fromEntries(globs.map((glob) => [glob, accept])),
      file,
      root,
    )?.[0];
  const ordered = ["posts/**/*.md", "**/*.md"];

  it.each<[string, string[], string, string, string | undefined]>([
    ["the first matching glob", ordered, "/p/posts/a.md", "/p", "posts/**/*.md"],
    [
      "declaration order, not specificity",
      ["**/*.md", "posts/**/*.md"],
      "/p/posts/a.md",
      "/p",
      "**/*.md",
    ],
    ["later globs when earlier ones miss", ordered, "/p/other/a.md", "/p", "**/*.md"],
    ["files directly in the root", ordered, "/p/a.md", "/p", "**/*.md"],
    ["nothing above the root", ordered, "/a.md", "/p", undefined],
    ["nothing in a sibling sharing the root prefix", ordered, "/p-old/posts/a.md", "/p", undefined],
    ["files in dot directories", ordered, "/p/.hidden/a.md", "/p", "**/*.md"],
    ["dotfiles", ordered, "/p/posts/.draft.md", "/p", "posts/**/*.md"],
    ["names that merely start with two dots", ordered, "/p/..hidden.md", "/p", "**/*.md"],
    ["directories that merely start with two dots", ordered, "/p/..dir/a.md", "/p", "**/*.md"],
    ["backslash separators", ordered, "/p/posts\\a.md", "/p", "posts/**/*.md"],
    ["a root with a trailing slash", ordered, "/p/posts/a.md", "/p/", "posts/**/*.md"],
    ["an unnormalized file path", ordered, "/p/x/../posts/a.md", "/p", "posts/**/*.md"],
    ["a relative root", ordered, "content/posts/a.md", "content", "posts/**/*.md"],
    ["a dot-relative root", ordered, "./content/posts/a.md", "./content/", "posts/**/*.md"],
    [
      "brace alternatives",
      ["{posts,guides}/*.{md,mdx}"],
      "/p/guides/a.mdx",
      "/p",
      "{posts,guides}/*.{md,mdx}",
    ],
    [
      "nothing outside brace alternatives",
      ["{posts,guides}/*.{md,mdx}"],
      "/p/guides/a.mdc",
      "/p",
      undefined,
    ],
    ["negated globs", ["!posts/**"], "/p/guides/a.md", "/p", "!posts/**"],
    ["nothing a negated glob excludes", ["!posts/**"], "/p/posts/a.md", "/p", undefined],
    ["extglob repetition", ["posts/+(a|b).md"], "/p/posts/ab.md", "/p", "posts/+(a|b).md"],
    ["extglob negation", ["posts/!(draft-*).md"], "/p/posts/a.md", "/p", "posts/!(draft-*).md"],
    [
      "nothing an extglob negation excludes",
      ["posts/!(draft-*).md"],
      "/p/posts/draft-a.md",
      "/p",
      undefined,
    ],
    ["single characters", ["posts/?.md"], "/p/posts/a.md", "/p", "posts/?.md"],
    ["character classes", ["posts/[0-9]*.md"], "/p/posts/01-a.md", "/p", "posts/[0-9]*.md"],
    [
      "nothing across directories for a single star",
      ["posts/*.md"],
      "/p/posts/deep/a.md",
      "/p",
      undefined,
    ],
    ["nothing when only the case differs", ["posts/*.md"], "/p/Posts/a.MD", "/p", undefined],
    ["non-ASCII paths", ["記事/**/*.md"], "/p/記事/日本語.md", "/p", "記事/**/*.md"],
    ["paths with spaces", ["my posts/*.md"], "/p/my posts/a b.md", "/p", "my posts/*.md"],
  ])("selects %s", (_, globs, file, root, expected) => {
    expect(select(globs, file, root)).toBe(expected);
  });

  it("returns the matching entry and nothing without schemas", () => {
    const definition = { schema: accept, jsonSchema: {} };
    expect(selectFrontmatterSchema({ "posts/*.md": definition }, "/p/posts/a.md", "/p")).toEqual([
      "posts/*.md",
      definition,
    ]);
    expect(selectFrontmatterSchema({ "*.md": definition }, "/p/a.md", "/p")?.[1]).toBe(definition);
    expect(selectFrontmatterSchema(undefined, "/p/posts/a.md", "/p")).toBeUndefined();
    expect(selectFrontmatterSchema({}, "/p/posts/a.md", "/p")).toBeUndefined();
  });
});
