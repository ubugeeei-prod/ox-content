import { dirname, resolve } from "node:path";
import { createRequire } from "node:module";
import { readFile } from "node:fs/promises";
import {
  resolveCitationsOptions,
  transformCitations,
} from "../../npm/vite-plugin-ox-content/src/citations";
import {
  resolveCrossReferencesOptions,
  transformCrossReferences,
} from "../../npm/vite-plugin-ox-content/src/cross-references";
import { highlightCode } from "../../npm/vite-plugin-ox-content/src/highlight";
import {
  resetTabGroupCounter,
  transformTabs,
} from "../../npm/vite-plugin-ox-content/src/plugins/tabs";
import { transformPm } from "../../npm/vite-plugin-ox-content/src/plugins/pm";
import { transformYouTube } from "../../npm/vite-plugin-ox-content/src/plugins/youtube";
import { lintCodeBlocks, extractDocsTests } from "../../npm/vite-plugin-ox-content/src/code-blocks";

const napi = createRequire(import.meta.url)("@ox-content/napi");
const media = JSON.parse(await readFile(process.env.OX_MEDIA_FIXTURES!, "utf8"));
const registered = napi
  .mediaEmbedTags()
  .map((tag: { name: string }) => tag.name)
  .sort();
const covered = media.fixtures.map((fixture: { name: string }) => fixture.name).sort();
if (JSON.stringify(registered) !== JSON.stringify(covered))
  throw new Error("Media registry coverage mismatch");
const bibliographyFile = resolve(process.env.OX_BIBLIOGRAPHY!);
const citationOptions = resolveCitationsOptions({
  bibliography: bibliographyFile,
  rootDir: dirname(bibliographyFile),
});
const prose = "<p>Plain prose with <strong>formatting</strong> and 日本語.</p>";
export interface Workload {
  name: string;
  input: string;
  run: (input: string) => unknown;
  validate?: (output: any) => boolean;
}
export const cases: Workload[] = [];
for (const fixture of media.fixtures) {
  for (const count of [1, 32, 256]) {
    cases.push({
      name: `media/${fixture.name}/${count}`,
      input: fixture.html.repeat(count),
      run: (input) => napi.transformMediaEmbedsWithDiagnostics(input, media.options),
      validate: (output) =>
        output.diagnostics.length === 0 && output.html !== fixture.html.repeat(count),
    });
  }
}
cases.push({
  name: "media/absent",
  input: prose.repeat(256),
  run: (input) => napi.transformMediaEmbedsWithDiagnostics(input, media.options),
  validate: (output) => output.diagnostics.length === 0,
});
for (const count of [1, 32, 256]) {
  for (const [name, input, run] of [
    [
      "tabs",
      '<tabs><tab label="One"><p>One</p></tab><tab label="Two"><p>Two</p></tab></tabs>',
      async (input: string) => {
        resetTabGroupCounter();
        return transformTabs(input);
      },
    ],
    [
      "pm",
      "<pm>npm install ox-content</pm>",
      async (input: string) => {
        resetTabGroupCounter();
        return transformPm(input);
      },
    ],
    ["youtube", '<YouTube id="dQw4w9WgXcQ"></YouTube>', transformYouTube],
    [
      "highlight",
      '<pre><code class="language-ts">export const answer: number = 42;</code></pre>',
      highlightCode,
    ],
  ] as const) {
    cases.push({
      name: `${name}/${count}`,
      input: input.repeat(count),
      run,
      validate: (output) => output !== input.repeat(count),
    });
    if (count === 1) cases.push({ name: `${name}/absent`, input: prose.repeat(256), run });
  }
  cases.push({
    name: `cross_references/${count}`,
    input: Array.from(
      { length: count },
      (_, i) => `<h2 id="sec-${i}">Section ${i}</h2><p>See [@sec-${i}].</p>`,
    ).join(""),
    run: (input) => transformCrossReferences(input, resolveCrossReferencesOptions(true)),
    validate: (output) => output.references.length === count,
  });
  cases.push({
    name: `citations/${count}`,
    input: "<p>Citation [@example].</p>".repeat(count),
    run: (input) => transformCitations(input, citationOptions),
    validate: (output) => output.citations.length === count,
  });
  cases.push({
    name: `code_lint/${count}`,
    input: "```ts\nconst value = 1;  \n```\n".repeat(count),
    run: (input) => lintCodeBlocks(input),
    validate: (output) => output.length === count,
  });
  cases.push({
    name: `docs_test_extract/${count}`,
    input: "```js test\nconsole.assert(1 + 1 === 2);\n```\n".repeat(count),
    run: (input) => extractDocsTests(input),
    validate: (output) => output.length === count,
  });
}
cases.push({
  name: "cross_references/absent",
  input: prose.repeat(256),
  run: (input) => transformCrossReferences(input, resolveCrossReferencesOptions(true)),
});
cases.push({
  name: "citations/absent",
  input: prose.repeat(256),
  run: (input) => transformCitations(input, citationOptions),
});
cases.push({
  name: "code_lint/absent",
  input: prose.repeat(256),
  run: (input) => lintCodeBlocks(input),
});
cases.push({
  name: "docs_test_extract/absent",
  input: prose.repeat(256),
  run: (input) => extractDocsTests(input),
});
