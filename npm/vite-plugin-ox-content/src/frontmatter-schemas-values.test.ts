import type { StandardSchemaV1 } from "@standard-schema/spec";
import { describe, expect, expectTypeOf, it } from "vite-plus/test";
import { z } from "zod";
import { convertible, hybrid, schema } from "./frontmatter-schema-fixtures";
import {
  checkFrontmatterValue,
  defineFrontmatterSchemas,
  frontmatterJsonSchema,
  type InferFrontmatter,
  type InferFrontmatterInput,
} from "./frontmatter-schemas";

const accept = schema();

describe("checkFrontmatterValue", () => {
  it("passes the input through untouched and returns the validator output", async () => {
    const input = { title: "x" };
    const output = { title: "X" };
    const inputs: unknown[] = [];
    const definition = schema((value) => (inputs.push(value), { value: output }));
    const result = await checkFrontmatterValue(definition, input);
    expect(result).toEqual({ value: output, issues: [] });
    expect(result.value).toBe(output);
    expect(inputs[0]).toBe(input);
  });

  it("awaits asynchronous validators, adapters and self-describing validators", async () => {
    const asynchronous = schema(async (value) => ({
      value: { ...(value as object), async: true },
    }));
    expect(await checkFrontmatterValue(asynchronous, { a: 1 })).toEqual({
      value: { a: 1, async: true },
      issues: [],
    });
    expect(await checkFrontmatterValue({ schema: asynchronous, jsonSchema: {} }, {})).toEqual({
      value: { async: true },
      issues: [],
    });
    expect(await checkFrontmatterValue(hybrid().definition, { a: 1 })).toEqual({
      value: { a: 1 },
      issues: [],
    });
  });

  it("normalizes issue paths and keeps messages in order", async () => {
    const brand = Symbol("brand");
    const result = await checkFrontmatterValue(
      schema(() => ({
        issues: [
          { message: "no path" },
          { message: "keys", path: ["author", 0, brand] },
          { message: "segments", path: [{ key: "tags" }, { key: 1 }, "name"] },
        ],
      })),
      {},
    );
    expect(result).toEqual({
      issues: [
        { message: "no path", path: [] },
        { message: "keys", path: ["author", 0, brand] },
        { message: "segments", path: ["tags", 1, "name"] },
      ],
    });
    expect("value" in result).toBe(false);
  });

  it.each([
    [
      "a thrown error",
      () => {
        throw new RangeError("out of range");
      },
      "Schema validation failed: RangeError: out of range",
    ],
    [
      "a thrown string",
      () => {
        throw "plain string";
      },
      "Schema validation failed: plain string",
    ],
    [
      "a rejected promise",
      () => Promise.reject(new Error("later")),
      "Schema validation failed: Error: later",
    ],
  ])("reports %s as a root issue", async (_, validate, message) => {
    expect(await checkFrontmatterValue(schema(validate), {})).toEqual({
      issues: [{ message, path: [] }],
    });
  });

  it.each([
    ["an array", []],
    ["null", null],
    ["a string", "text"],
    ["a number", 0],
    ["undefined", undefined],
  ])("rejects %s as validator output", async (_, value) => {
    expect(
      await checkFrontmatterValue(
        schema(() => ({ value })),
        {},
      ),
    ).toEqual({
      issues: [{ message: "Frontmatter schemas must produce an object", path: [] }],
    });
  });
});

describe("frontmatterJsonSchema", () => {
  it("returns nothing for validators without a JSON Schema converter", () => {
    expect(frontmatterJsonSchema(accept)).toBeUndefined();
    // TypeScript rejects this shape; JavaScript configs can still pass it.
    const untyped = accept as unknown as Record<string, unknown>;
    expect(frontmatterJsonSchema({ schema: accept, jsonSchema: untyped })).toBeUndefined();
  });

  it("asks Standard JSON Schema for the draft 2020-12 input shape only", () => {
    const { calls, definition } = convertible();
    expect(frontmatterJsonSchema(definition)).toEqual({ title: "input" });
    expect(calls).toEqual([["input", { target: "draft-2020-12" }]]);
  });

  it("prefers the explicit adapter shape over the validator's own converter", () => {
    const { calls, definition } = convertible();
    const explicit = { type: "object", title: "explicit" };
    expect(frontmatterJsonSchema({ schema: definition, jsonSchema: explicit })).toBe(explicit);
    expect(calls).toEqual([]);
  });

  it("converts an adapter whose shape is itself a Standard JSON Schema", () => {
    const { calls, definition } = convertible();
    expect(frontmatterJsonSchema({ schema: accept, jsonSchema: definition })).toEqual({
      title: "input",
    });
    expect(calls).toEqual([["input", { target: "draft-2020-12" }]]);
  });

  it("uses a self-describing validator's converter, not its unrelated properties", () => {
    const { calls, definition } = hybrid();
    expect(frontmatterJsonSchema(definition)).toEqual({ title: "input" });
    expect(calls).toEqual([["input", { target: "draft-2020-12" }]]);
  });

  it("describes what authors write: transforms keep input types and defaults stay optional", () => {
    const shape = frontmatterJsonSchema(
      z.object({
        title: z.string(),
        draft: z.boolean().default(false),
        order: z.string().transform(Number),
        level: z.enum(["beginner", "advanced"]).optional(),
      }),
    );
    expect(shape).toMatchObject({
      type: "object",
      required: ["title", "order"],
      properties: {
        title: { type: "string" },
        draft: { type: "boolean", default: false },
        order: { type: "string" },
        level: { enum: ["beginner", "advanced"] },
      },
    });
  });
});

describe("frontmatter schema type inference", () => {
  const post = z.object({
    title: z.string(),
    order: z.string().transform(Number),
    draft: z.boolean().default(false),
  });
  const custom = accept as StandardSchemaV1<{ raw: string }, { parsed: number }>;
  const schemas = defineFrontmatterSchemas({
    "posts/*.md": post,
    "adapted/*.md": { schema: post, jsonSchema: { type: "object" } },
    "custom/*.md": custom,
  });

  it("keeps the declared globs as literal keys", () => {
    expectTypeOf<keyof typeof schemas>().toEqualTypeOf<
      "posts/*.md" | "adapted/*.md" | "custom/*.md"
    >();
    expect(Object.keys(schemas)).toEqual(["posts/*.md", "adapted/*.md", "custom/*.md"]);
  });

  it("infers transformed output and authored input separately", () => {
    expectTypeOf<InferFrontmatter<typeof schemas, "posts/*.md">>().toEqualTypeOf<{
      title: string;
      order: number;
      draft: boolean;
    }>();
    expectTypeOf<InferFrontmatterInput<typeof schemas, "posts/*.md">>().toEqualTypeOf<{
      title: string;
      order: string;
      draft?: boolean;
    }>();
  });

  it("infers through JSON Schema adapters and hand-written Standard Schemas", () => {
    expectTypeOf<InferFrontmatter<typeof schemas, "adapted/*.md">>().toEqualTypeOf<
      InferFrontmatter<typeof schemas, "posts/*.md">
    >();
    expectTypeOf<InferFrontmatterInput<typeof schemas, "adapted/*.md">>().toEqualTypeOf<
      InferFrontmatterInput<typeof schemas, "posts/*.md">
    >();
    expectTypeOf<InferFrontmatter<typeof schemas, "custom/*.md">>().toEqualTypeOf<{
      parsed: number;
    }>();
    expectTypeOf<InferFrontmatterInput<typeof schemas, "custom/*.md">>().toEqualTypeOf<{
      raw: string;
    }>();
  });
});
