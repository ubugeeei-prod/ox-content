import { describe, expect, expectTypeOf, it } from "vite-plus/test";
import { z } from "zod";
import { checkFrontmatter } from "./frontmatter-check";
import {
  checkFrontmatterValue,
  defineFrontmatterSchemas,
  frontmatterJsonSchema,
  selectFrontmatterSchema,
  type InferFrontmatter,
  type InferFrontmatterInput,
} from "./frontmatter-schemas";

const root = "/project/content";
const schemas = defineFrontmatterSchemas({
  "posts/**/*.md": z.object({
    title: z.string(),
    draft: z.boolean().default(false),
    rating: z.number().optional(),
  }),
  "guides/**/*.md": z.object({ title: z.string(), level: z.enum(["beginner", "advanced"]) }),
});

describe("Standard Schema frontmatter", () => {
  it("preserves per-glob input and output inference", () => {
    expectTypeOf<InferFrontmatter<typeof schemas, "posts/**/*.md">>().toEqualTypeOf<{
      title: string;
      draft: boolean;
      rating?: number;
    }>();
    expectTypeOf<InferFrontmatterInput<typeof schemas, "posts/**/*.md">>().toEqualTypeOf<{
      title: string;
      draft?: boolean;
      rating?: number;
    }>();
  });

  it("selects source-relative globs in declaration order and excludes outside files", () => {
    expect(selectFrontmatterSchema(schemas, `${root}/posts/deep/a.md`, root)?.[0]).toBe(
      "posts/**/*.md",
    );
    expect(selectFrontmatterSchema(schemas, `${root}/guides/a.md`, root)?.[0]).toBe(
      "guides/**/*.md",
    );
    expect(selectFrontmatterSchema(schemas, "/other/posts/a.md", root)).toBeUndefined();
    const ordered = defineFrontmatterSchemas({
      "posts/*.md": schemas["posts/**/*.md"],
      "**/*.md": schemas["guides/**/*.md"],
    });
    expect(selectFrontmatterSchema(ordered, `${root}/posts/a.md`, root)?.[0]).toBe("posts/*.md");
  });

  it("validates YAML and returns defaults without modifying unmatched documents", async () => {
    const result = await checkFrontmatter(
      "---\ntitle: Hello\n---\n# Hello",
      `${root}/posts/a.md`,
      schemas,
      root,
    );
    expect(result.diagnostics).toEqual([]);
    expect(result.value).toEqual({ title: "Hello", draft: false });
    expect(await checkFrontmatter("invalid YAML", `${root}/other/a.md`, schemas, root)).toEqual({
      diagnostics: [],
      value: undefined,
    });
  });

  it("reports wrong types at their exact YAML values", async () => {
    const result = await checkFrontmatter(
      "---\ntitle: Hello\nrating: wrong\n---\n",
      `${root}/posts/a.md`,
      schemas,
      root,
    );
    expect(result.diagnostics).toHaveLength(1);
    expect(result.diagnostics[0]).toMatchObject({ path: ["rating"], line: 3, column: 9 });
  });

  it.each([
    "---\ntitle: [broken\n---\n",
    "---\ntitle: Hello\n",
    "---\n- wrong\n---\n",
    "---\ntitle: First\ntitle: Second\n---\n",
    "# No frontmatter\n",
  ])("rejects malformed, duplicate, unterminated or missing fields: %s", async (source) => {
    const result = await checkFrontmatter(source, `${root}/posts/a.md`, schemas, root);
    expect(result.diagnostics.length).toBeGreaterThan(0);
  });

  it("awaits asynchronous custom refinements and preserves issue paths", async () => {
    const asyncSchemas = defineFrontmatterSchemas({
      "**/*.md": z.object({
        title: z.string().refine(async (value) => value !== "forbidden", "Reserved title"),
      }),
    });
    const result = await checkFrontmatter(
      "---\ntitle: forbidden\n---\n",
      `${root}/a.md`,
      asyncSchemas,
      root,
    );
    expect(result.diagnostics[0]).toMatchObject({
      message: "Reserved title",
      path: ["title"],
      line: 2,
    });
  });

  it("exports Standard JSON Schema input metadata and accepts explicit adapters", () => {
    expect(frontmatterJsonSchema(schemas["posts/**/*.md"])).toMatchObject({
      type: "object",
      properties: { title: { type: "string" }, draft: { type: "boolean" } },
    });
    expect(
      frontmatterJsonSchema({
        schema: schemas["posts/**/*.md"],
        jsonSchema: { type: "object", properties: { title: { type: "string" } } },
      }),
    ).toMatchObject({ type: "object" });
  });

  it("rejects non-object outputs and invalid schema declarations", async () => {
    expect((await checkFrontmatterValue(z.string(), "Hello")).issues[0]?.message).toContain(
      "must produce an object",
    );
    expect(() => defineFrontmatterSchemas({ "../outside.md": schemas["posts/**/*.md"] })).toThrow(
      "relative",
    );
    expect(() => defineFrontmatterSchemas({ "**/*.md": {} as never })).toThrow(
      "Invalid Standard Schema",
    );
  });
});
