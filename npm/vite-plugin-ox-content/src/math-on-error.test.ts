import { describe, expect, it } from "vite-plus/test";
import { createDocsResolvedOptions } from "../test/fixtures/docs-fixture";
import { resolveMathOptions } from "./index";
import { transformMarkdown } from "./transform";
import type { MathErrorPolicy, ResolvedOptions } from "./types";

// Prose that quotes math syntax: the `$…$` heuristics pick it up, and KaTeX
// then refuses it. Straight from the report.
const QUOTED_SYNTAX =
  "- 移行後の制約: '$\\cdots $' のように $ の一つ内側の端にスペースを書くとレンダリングされない " +
  "('$\\cdots$' や '$a \\cdots b$' は OK)\n";

async function render(source: string, onError?: MathErrorPolicy): Promise<string> {
  const options = createDocsResolvedOptions({
    math: { enabled: true, ...(onError ? { onError } : { onError: "literal" }) },
  } as Partial<ResolvedOptions>);
  const result = await transformMarkdown(source, "docs/math.md", options);
  return result.html;
}

function silenceWarnings<T>(run: () => Promise<T>): Promise<T> {
  const original = console.warn;
  console.warn = () => {};
  return run().finally(() => {
    console.warn = original;
  });
}

describe("math onError", () => {
  it("defaults to keeping the source literal", () => {
    expect(resolveMathOptions(true).onError).toBe("literal");
    expect(resolveMathOptions({}).onError).toBe("literal");
    expect(resolveMathOptions({ onError: "render" }).onError).toBe("render");
  });

  it("leaves prose that only quotes math syntax alone", async () => {
    const html = await silenceWarnings(() => render(QUOTED_SYNTAX));

    expect(html).not.toContain("katex-error");
    expect(html).not.toContain("color:#cc0000");
    // The sentence reads the way it was written, delimiters included.
    expect(html).toContain("$\\cdots $");
    expect(html).toContain("のように");
  });

  it("warns about what it left alone", async () => {
    const warnings: string[] = [];
    const original = console.warn;
    console.warn = (...args: unknown[]) => void warnings.push(args.join(" "));
    try {
      await render(QUOTED_SYNTAX);
    } finally {
      console.warn = original;
    }

    expect(warnings.join("\n")).toContain("docs/math.md");
    expect(warnings.join("\n")).toContain("math left as written");
  });

  it("fails the build when asked to", async () => {
    await expect(silenceWarnings(() => render(QUOTED_SYNTAX, "error"))).rejects.toThrow(
      /KaTeX parse error/,
    );
  });

  it("keeps the old red error markup available", async () => {
    const html = await silenceWarnings(() => render(QUOTED_SYNTAX, "render"));

    expect(html).toContain("katex-error");
  });

  it("never escapes an entity twice", async () => {
    // The apostrophes come back from the rehype passes as `&#x27;`. Decoding
    // only `&#39;` left those five characters in the TeX, and KaTeX escaped
    // their `&` again into `&#x26;#x27;`.
    for (const policy of ["literal", "render"] as const) {
      const html = await silenceWarnings(() => render(QUOTED_SYNTAX, policy));
      expect(html).not.toContain("&#x26;");
      expect(html).not.toContain("&amp;#x27;");
    }
  });

  it("still renders math that parses", async () => {
    const html = await silenceWarnings(() => render("$E = mc^2$ and $$a + b$$\n"));

    expect(html).toContain("katex");
    expect(html).not.toContain("katex-error");
    expect(html).toContain("mc");
  });
});
