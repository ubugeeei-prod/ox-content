// Test-only upstream oracle. Production linting is implemented in Rust.
import assert from "node:assert/strict";
import { readFile, writeFile } from "node:fs/promises";
import { pathToFileURL } from "node:url";

const reference = process.argv[2];
const update = process.argv.includes("--update");
const { lint } = await import(pathToFileURL(reference).href);
const fixture = new URL(
  "../../crates/ox_content_markdown_lint/tests/fixtures/markdownlint.json",
  import.meta.url,
);
const cases = JSON.parse(await readFile(fixture, "utf8"));
for (const entry of cases) {
  // MD052/MD053 declare parser:none upstream but consume the shared parse cache.
  // An empty custom rule keeps that cache populated when either is isolated.
  const parse = {
    names: ["TEST_PARSE"],
    description: "Populate parser cache",
    tags: ["test"],
    parser: "micromark",
    function() {},
  };
  const result = lint({
    strings: { input: entry.source },
    config: { ...entry.config, TEST_PARSE: true },
    customRules: [parse],
  });
  const expected = result.input
    .map((error) => [error.ruleNames[0], error.lineNumber])
    .sort((a, b) => a[1] - b[1] || a[0].localeCompare(b[0]));
  if (update) {
    entry.expected = expected;
  } else {
    assert.deepEqual(entry.expected, expected, entry.name);
  }
}
if (update) {
  await writeFile(fixture, `${JSON.stringify(cases, null, 2)}\n`);
}
console.log(`Verified ${cases.length} cases against markdownlint 0.41.1`);
