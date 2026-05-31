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

````md
```ts test
import { expect, it } from "vitest";

it("works", () => {
  expect(1 + 1).toBe(2);
});
```
````

Use the returned block list to generate a Vitest file in your own CI harness.
