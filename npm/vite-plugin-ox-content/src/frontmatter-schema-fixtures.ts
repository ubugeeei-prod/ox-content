import type { StandardJSONSchemaV1, StandardSchemaV1 } from "@standard-schema/spec";

type Validate = StandardSchemaV1.Props["validate"];

/** A minimal Standard Schema; accepts every value unless told otherwise. */
export const schema = (validate: Validate = (value) => ({ value })): StandardSchemaV1 => ({
  "~standard": { version: 1, vendor: "test", validate },
});

/** Always fails with the given issues. */
export const failing = (...issues: StandardSchemaV1.Issue[]) => schema(() => ({ issues }));

/** Accepts everything and records what the validator was given. */
export function recorder() {
  const inputs: unknown[] = [];
  return { inputs, schema: schema((value) => (inputs.push(value), { value })) };
}

/** A Standard JSON Schema that records which converter was asked for which target. */
export function convertible() {
  const calls: unknown[] = [];
  const definition: StandardSchemaV1 & StandardJSONSchemaV1 = {
    "~standard": {
      version: 1,
      vendor: "test",
      validate: (value) => ({ value }),
      jsonSchema: {
        input: (options) => (calls.push(["input", options]), { title: "input" }),
        output: (options) => (calls.push(["output", options]), { title: "output" }),
      },
    },
  };
  return { calls, definition };
}

/** A validator that happens to expose its own `schema` and `jsonSchema` properties. */
export function hybrid() {
  const { calls, definition } = convertible();
  return {
    calls,
    definition: { ...definition, schema: { unused: true }, jsonSchema: { unused: true } },
  };
}
