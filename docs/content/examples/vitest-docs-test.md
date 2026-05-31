---
title: Vitest Docs Tests
description: Extract runnable examples for docs-as-tests harnesses.
---

# Vitest Docs Tests

Docs test extraction is opt-in. The native scanner extracts fenced code blocks
without parsing the whole document in JavaScript.

```ts
import { extractDocsTests } from "@ox-content/vite-plugin";

const blocks = await extractDocsTests(markdown, {
  languages: ["ts", "tsx"],
  requireMeta: true,
});
```

You can turn those blocks into temporary Vitest files and run them from CI:

```ts
import { runDocsTests } from "@ox-content/vite-plugin";

await runDocsTests({
  include: ["docs/content/**/*.md"],
  vitestCommand: "vitest",
  vitestArgs: ["run"],
});
```

To test examples in implementation documentation comments, switch the source to
`jsdoc`. This path uses the same ox-content source documentation extractor that
powers generated API docs, then runs the fenced `@example` blocks through the
Vitest harness.

```ts
import { runDocsTests } from "@ox-content/vite-plugin";

await runDocsTests({
  source: "jsdoc",
  src: ["./src"],
  include: ["**/*.ts"],
  ignore: ["**/*.test.ts"],
  vitestCommand: "vitest",
  vitestArgs: ["run"],
});
```

Like Cargo doctests, runnable fences are examples first: write normal example
code and assertions inside `@example`, and the harness wraps each fence in a
generated Vitest `test(...)`. A runnable fence is any configured JavaScript or
TypeScript language with `test`, `runnable`, `vitest`, or `docs-test` in the
fence metadata.

````ts
/**
 * Adds two values.
 *
 * @example
 * ```ts docs-test
 * import { expect } from "vitest";
 * import { add } from "@acme/math";
 *
 * expect(add(1, 2)).toBe(3);
 * ```
 */
export function add(left: number, right: number): number {
  return left + right;
}
````

```ts docs-test
import { expect } from "vitest";

const result = 1 + 1;
expect(result).toBe(2);
```

Set `executionMode: "module"` when a snippet already declares its own Vitest
tests.

````md
```ts test
import { expect, it } from "vitest";

it("works", () => {
  expect(1 + 1).toBe(2);
});
```
````
