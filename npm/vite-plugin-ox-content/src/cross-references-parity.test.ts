import { describe, expect, it } from "vite-plus/test";
import { transformCrossReferences } from "./cross-references";
import { importNapiModuleSync } from "./napi";
import type { CrossReferenceEntry } from "./cross-reference-types";

/**
 * `cross-references.ts` and `ox_content_transform::cross_references` both
 * number a document's headings, figures, and tables and link `@id` to them.
 * Before the TypeScript one is deleted, the two have to agree.
 *
 * The corpus targets the places where the port could silently diverge — the
 * regular expressions the TypeScript used had semantics that are easy to
 * approximate and hard to reproduce:
 *
 * - `\b` after an identifier that may end in `-`, which is not a word
 *   character, so the boundary flips to requiring one after it
 * - the prefix character being *consumed*, so `@fig-1@fig-2` yields one match
 * - `readAttr` preferring a quoted attribute anywhere over a bare one earlier
 * - `<a\b` not matching `<address`
 * - lazy bodies, so a verbatim element ends at its first close tag
 */
interface NativeOutput {
  html: string;
  references: CrossReferenceEntry[];
  diagnostics: { policy: string; message: string }[];
}

function native(html: string, options: Record<string, unknown>): NativeOutput {
  const napi = importNapiModuleSync() as unknown as {
    transformCrossReferences(html: string, options: Record<string, unknown>): NativeOutput;
  };
  return napi.transformCrossReferences(html, options);
}

const OPTIONS = {
  enabled: true,
  missing: "warn",
  duplicates: "warn",
  mismatches: "warn",
  labels: { figure: "Figure", table: "Table", section: "Section" },
} as const;

const CASES: Record<string, string> = {
  "nested section numbering":
    '<h1 id="sec-a">Alpha</h1><h2 id="sec-b">Beta</h2><h3 id="sec-c">Gamma</h3>' +
    '<h2 id="sec-d">Delta</h2><h1 id="sec-e">Epsilon</h1>',
  "heading depth resets": '<h2 id="sec-a">A</h2><h1 id="sec-b">B</h1><h2 id="sec-c">C</h2>',
  "figure with its own id":
    '<figure id="fig-one"><img src="a.png" alt="Alt"><figcaption>Cap</figcaption></figure>',
  "figure borrowing the image id": '<figure><img id="fig-two" src="b.png" alt="Second"></figure>',
  "standalone image": '<img id="fig-three" src="c.png" alt="Third">',
  "figure and image share one sequence":
    '<figure id="fig-a"><img src="a.png"></figure><img id="fig-b" src="b.png">' +
    '<figure><img id="fig-c" src="c.png"></figure>',
  "table with id": '<table id="tbl-one"><tbody><tr><td>x</td></tr></tbody></table>',
  "trailing paragraph label":
    "<table><tbody><tr><td>x</td></tr></tbody></table>\n<p>{#tbl-lifted}</p>",
  "trailing empty-cell label":
    '<table><tbody><tr><td>x</td></tr><tr><td id="tbl-cell"></td><td></td></tr></tbody></table>',
  "reference resolves": '<h1 id="sec-a">A</h1><p>See @sec-a for details.</p>',
  "reference at segment start": '<h1 id="sec-a">A</h1><p>@sec-a leads.</p>',
  "adjacent references consume the prefix":
    '<h1 id="sec-a">A</h1><h2 id="sec-b">B</h2><p>@sec-a@sec-b</p>',
  "identifier ending in a dash": '<h1 id="sec-a">A</h1><p>@sec- and @sec-a</p>',
  "reference after a slash": '<h1 id="sec-a">A</h1><p>path/@sec-a</p>',
  "reference after a bracket": '<h1 id="sec-a">A</h1><p>[@sec-a]</p>',
  "citation group is left alone":
    '<h1 id="sec-a">A</h1><p>[@smith2020] and @sec-a and [-@jones]</p>',
  "bracket that is not a citation": '<h1 id="sec-a">A</h1><p>[note] @sec-a</p>',
  "protected code span": '<h1 id="sec-a">A</h1><p><code>@sec-a</code> and @sec-a</p>',
  "protected pre block": '<h1 id="sec-a">A</h1><pre>@sec-a</pre><p>@sec-a</p>',
  "protected anchor": '<h1 id="sec-a">A</h1><p><a href="#x">@sec-a</a> @sec-a</p>',
  "address is not an anchor": '<h1 id="sec-a">A</h1><address>@sec-a</address>',
  "html comment": '<h1 id="sec-a">A</h1><!-- @sec-a --><p>@sec-a</p>',
  "missing target": "<p>@fig-nope</p>",
  "kind mismatch": '<h1 id="fig-a">A</h1><p>@fig-a</p>',
  "duplicate ids": '<h1 id="sec-a">A</h1><h2 id="sec-a">B</h2>',
  "untracked id is ignored": '<h1 id="intro">A</h1><p>@intro</p>',
  "header anchor stripped from the title":
    '<h1 id="sec-a">Alpha<a class="header-anchor" href="#sec-a">#</a></h1>',
  "entities in a heading": '<h1 id="sec-a">A &amp; B &lt;C&gt;</h1><p>@sec-a</p>',
  "quoted attribute wins over a bare one": '<h1 id id="sec-a">A</h1><p>@sec-a</p>',
  "single-quoted attribute": "<h1 id='sec-a'>A</h1><p>@sec-a</p>",
  "id needing url escaping": '<h1 id="sec-a b">A</h1><p>@sec-a</p>',
  "already annotated image is skipped":
    '<img id="fig-a" data-ox-xref-kind="figure" src="a.png"><img id="fig-b" src="b.png">',
  "whitespace collapsing in a title": '<h1 id="sec-a">  A   \n  B  </h1>',
  // Positions are recorded against whichever string that pass walked, and each
  // pass rewrites the document, so a figure's offset is measured in a longer
  // string than a section's. Both implementations inherit that; the corpus
  // pins it rather than pretending it is document order.
  "mixed sections and figures":
    '<h1 id="sec-a">A</h1><img id="fig-a" src="a.png"><h2 id="sec-b">B</h2>',
  "empty document": "",
  "no references at all": "<p>Plain text with no markers.</p>",
};

describe("cross-reference parity", () => {
  for (const [name, html] of Object.entries(CASES)) {
    it(`matches the TypeScript implementation: ${name}`, () => {
      const ts = transformCrossReferences(html, OPTIONS);
      const rust = native(html, OPTIONS);
      expect(rust.html).toBe(ts.html);
      expect(rust.references).toEqual(ts.references);
    });
  }
});
