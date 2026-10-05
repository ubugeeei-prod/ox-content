import { describe, expect, it } from "vite-plus/test";
import { fixMarkdown, lintMarkdown, lintMarkdownAsync, lintMarkdownDocumentsAsync } from "./lint";

describe("native Markdown lint quality", () => {
  it("forwards the markdownlint profile without enabling prose rules", () => {
    const source = "#  Title!\n\nTODO with with!!\n";
    expect(lintMarkdown(source, { markdownlint: true }).diagnostics.map((d) => d.ruleId)).toEqual([
      "MD019",
      "MD026",
    ]);
    expect(lintMarkdown(source, { markdownlint: { default: false } }).diagnostics).toEqual([]);
    expect(
      lintMarkdown(source, {
        markdownlint: { default: false },
        textRules: { noTodo: true },
      }).diagnostics.map((d) => d.ruleId),
    ).toEqual(["no-todo"]);
  });
  it("forwards markdownlint directives, configuration severity and safe fixes", () => {
    const source = "<!-- markdownlint-disable MD009 -->\ntext \n";
    const options = { markdownlint: { default: false, MD009: "warning" as const } };
    expect(lintMarkdown(source, options).diagnostics).toEqual([]);
    const report = lintMarkdown(source, { ...options, noInlineConfig: true });
    expect(report.diagnostics.map((d) => [d.ruleId, d.severity, d.line])).toEqual([
      ["MD009", "warning", 2],
    ]);
    expect(report.warningCount).toBe(1);
    const fixed = fixMarkdown("😀 Text   \r\nnext.\r\n\r\n\r\nEnd", {
      markdownlint: { default: false, MD009: true, MD012: true, MD047: true },
    });
    expect(fixed.output).toBe("😀 Text  \r\nnext.\r\n\r\nEnd\r\n");
    expect(fixed.appliedFixes).toBe(3);
    expect(fixed.result.diagnostics).toEqual([]);
  });
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
