/**
 * Interop helpers for loading ESM-only dependencies from the CommonJS build.
 */

/**
 * Unwrap the `default` export of an ESM-only dependency.
 *
 * The CommonJS build (`dist/index.cjs`) loads pure-ESM packages such as
 * `rehype-parse` and `rehype-stringify` through `require(esm)`. The bundler's
 * interop helper then double-wraps their default export: what should be the
 * plugin function arrives as the module namespace `{ default: fn }` instead of
 * `fn` itself. Passing that object to `unified().use()` throws
 * "Expected usable value but received an empty preset".
 *
 * Calling `interopDefault` on a default import unwraps one extra level of
 * `default` nesting, so the same code works whether the dependency was loaded
 * as native ESM (`.mjs`) or through the CommonJS interop (`.cjs`). See #452.
 */
export function interopDefault<T>(value: T): T {
  if (value !== null && typeof value === "object" && "default" in value) {
    return (value as { default: T }).default;
  }
  return value;
}
