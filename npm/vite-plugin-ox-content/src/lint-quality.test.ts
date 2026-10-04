import { describe, expect, it } from "vite-plus/test";
import { fixMarkdown, lintMarkdown, lintMarkdownAsync, lintMarkdownDocumentsAsync } from "./lint";

describe("native Markdown lint quality", () => {
  it("checks Setext and formatted heading text", () => {
    expect(
      lintMarkdown("Title\n=====\n\n# **Title**\n", {
        rules: { spellcheck: false },
      }).diagnostics.map((d) => d.ruleId),
    ).toEqual(["duplicate-heading"]);
  });
  it("opts prose rules in and returns UTF-8 edits with UTF-16 diagnostic columns", () => {
    const options = {
      rules: { spellcheck: false, finalNewline: true },
      textRules: { terminology: [{ term: "Javascript", replacement: "JavaScript" }] },
    };
    const source = "😀 Javascript with with";
    const result = lintMarkdown(source, options);
    expect(result.diagnostics.find((d) => d.ruleId === "terminology")?.column).toBe(4);
    const fixed = fixMarkdown(source, options);
    expect(fixed.output).toBe("😀 JavaScript with\n");
    expect(fixed.appliedFixes).toBe(3);
    expect(fixed.result.diagnostics).toEqual([]);
  });
  it("uses opt-in prose severity and suppression with standard spellchecking", async () => {
    const result = await lintMarkdownAsync(
      "<!-- oxlint-disable-next-line spellcheck -->\nwrld\n\nwrld\nTODO\n",
      {
        dictionary: { standard: { languages: ["en"] } },
        textRules: { noTodo: true },
        severities: { spellcheck: "info", "no-todo": "error" },
      },
    );
    expect(
      result.diagnostics.filter((d) => d.ruleId === "spellcheck").map((d) => [d.line, d.severity]),
    ).toEqual([[4, "info"]]);
    expect(result.errorCount).toBe(1);
  }, 15_000);
  it("preserves serial ordering for native parallel batches", async () => {
    const sources = Array.from({ length: 32 }, (_, i) => `# Page ${i}\n\nwith with\n`);
    const options = { rules: { spellcheck: false }, textRules: { noTodo: true } };
    expect(await lintMarkdownDocumentsAsync(sources, options)).toEqual(
      sources.map((s) => lintMarkdown(s, options)),
    );
  });
});
