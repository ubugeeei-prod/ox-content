import { describe, expect, it } from "vite-plus/test";
import {
  createIncrementalMarkdownParser,
  createIncrementalMarkdownRenderer,
  renderMarkdownStream,
} from "./incremental";

describe("incremental markdown", () => {
  it("renders partial heading and emphasis as replaceable pending html", () => {
    const renderer = createIncrementalMarkdownRenderer({ gfm: true });

    const heading = renderer.append("# Hel");
    expect(heading.didCommit).toBe(false);
    expect(heading.deltaHtml).toBe("");
    expect(heading.pendingHtml).toBe('<h1 id="hel">Hel</h1>\n');
    expect(heading.html).toBe('<h1 id="hel">Hel</h1>\n');

    const emphasis = renderer.append("lo **wor");
    expect(emphasis.didCommit).toBe(false);
    expect(emphasis.pendingHtml).toBe('<h1 id="hello-wor">Hello <strong>wor</strong></h1>\n');

    const committed = renderer.append("ld**\n\nNext");
    expect(committed.didCommit).toBe(true);
    expect(committed.deltaHtml).toBe('<h1 id="hello-world">Hello <strong>world</strong></h1>\n');
    expect(committed.pendingHtml).toBe("<p>Next</p>\n");
    expect(committed.html).toBe(
      '<h1 id="hello-world">Hello <strong>world</strong></h1>\n<p>Next</p>\n',
    );
  });

  it("finalizes the pending tail", () => {
    const renderer = createIncrementalMarkdownRenderer({ gfm: true });
    renderer.append("Intro\n\nNext");

    const result = renderer.finish();
    expect(result.didCommit).toBe(true);
    expect(result.pendingHtml).toBe("");
    expect(result.html).toBe("<p>Intro</p>\n<p>Next</p>\n");
  });

  it("can return provisional parser ast for partial inline markup", () => {
    const parser = createIncrementalMarkdownParser({ gfm: true, includePendingAst: true });
    const result = parser.append("This is **bo");

    expect(result.didCommit).toBe(false);
    expect(result.ast).toBe(null);
    expect(JSON.stringify(result.pendingAst)).toContain('"strong"');
  });

  it("streams render updates from async iterables", async () => {
    async function* chunks() {
      yield "# Hel";
      yield "lo\n\nBody";
    }

    const results = [];
    for await (const result of renderMarkdownStream(chunks(), { gfm: true })) {
      results.push(result.html);
    }

    expect(results).toEqual([
      '<h1 id="hel">Hel</h1>\n',
      '<h1 id="hello">Hello</h1>\n<p>Body</p>\n',
      '<h1 id="hello">Hello</h1>\n<p>Body</p>\n',
    ]);
  });
});
