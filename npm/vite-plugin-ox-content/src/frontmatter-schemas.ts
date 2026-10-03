import type { StandardJSONSchemaV1, StandardSchemaV1 } from "@standard-schema/spec";
import { minimatch } from "minimatch";
import { relative, resolve } from "node:path";

/** Optional JSON shape for completions when a validator has no Standard JSON Schema converter. */
export interface FrontmatterSchemaAdapter<S extends StandardSchemaV1 = StandardSchemaV1> {
  schema: S;
  jsonSchema: StandardJSONSchemaV1 | Record<string, unknown>;
}
export type FrontmatterSchema = StandardSchemaV1 | FrontmatterSchemaAdapter;
/** Ordered source-relative globs. The first matching schema wins. */
export type FrontmatterSchemas = Record<string, FrontmatterSchema>;
type Validator<S> =
  S extends FrontmatterSchemaAdapter<infer V> ? V : S extends StandardSchemaV1 ? S : never;
export type InferFrontmatter<
  Schemas extends FrontmatterSchemas,
  Glob extends keyof Schemas,
> = StandardSchemaV1.InferOutput<Validator<Schemas[Glob]>>;
export type InferFrontmatterInput<
  Schemas extends FrontmatterSchemas,
  Glob extends keyof Schemas,
> = StandardSchemaV1.InferInput<Validator<Schemas[Glob]>>;

/** Preserve each glob's inferred input/output types without choosing a validation library. */
export function defineFrontmatterSchemas<const S extends FrontmatterSchemas>(schemas: S): S {
  for (const [glob, definition] of Object.entries(schemas)) {
    if (!glob || glob.startsWith("/") || glob.split("/").includes(".."))
      throw new Error(`Frontmatter schema globs must be relative to srcDir: ${glob}`);
    const standard = validator(definition)["~standard"];
    if (standard?.version !== 1 || typeof standard.validate !== "function")
      throw new Error(`Invalid Standard Schema for ${glob}`);
  }
  return schemas;
}

export function validator(definition: FrontmatterSchema): StandardSchemaV1 {
  return "schema" in definition && !("~standard" in definition)
    ? definition.schema
    : (definition as StandardSchemaV1);
}

export function selectFrontmatterSchema(
  schemas: FrontmatterSchemas | undefined,
  file: string,
  root: string,
) {
  if (!schemas) return undefined;
  const path = relative(resolve(root), resolve(file)).replaceAll("\\", "/");
  if (path.startsWith("../") || path === "..") return undefined;
  return Object.entries(schemas).find(([glob]) => minimatch(path, glob, { dot: true }));
}

export interface FrontmatterIssue {
  message: string;
  path: PropertyKey[];
}

export async function checkFrontmatterValue(
  definition: FrontmatterSchema,
  value: unknown,
): Promise<{ value?: Record<string, unknown>; issues: FrontmatterIssue[] }> {
  const result = await validator(definition)["~standard"].validate(value);
  if (result.issues)
    return {
      issues: result.issues.map((issue) => ({
        message: issue.message,
        path: (issue.path ?? []).map((segment) =>
          typeof segment === "object" ? segment.key : segment,
        ),
      })),
    };
  if (!result.value || typeof result.value !== "object" || Array.isArray(result.value))
    return { issues: [{ message: "Frontmatter schemas must produce an object", path: [] }] };
  return { value: result.value as Record<string, unknown>, issues: [] };
}

/** Input JSON Schema describes the YAML authors write, before defaults or transforms. */
export function frontmatterJsonSchema(
  definition: FrontmatterSchema,
): Record<string, unknown> | undefined {
  const candidate =
    "schema" in definition && !("~standard" in definition) ? definition.jsonSchema : definition;
  const standard = (candidate as Partial<StandardJSONSchemaV1>)["~standard"];
  if (standard && "jsonSchema" in standard)
    return standard.jsonSchema.input({ target: "draft-2020-12" });
  if (candidate !== definition && !("~standard" in candidate))
    return candidate as Record<string, unknown>;
  return undefined;
}
