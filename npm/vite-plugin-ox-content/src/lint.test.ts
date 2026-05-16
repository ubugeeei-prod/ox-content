import { describe, expect, it } from "vite-plus/test";
import { lintMarkdown, lintMarkdownAsync } from "./lint";

describe("lintMarkdown", () => {
  it("reports structural Markdown issues", () => {
    const result = lintMarkdown("# Title\n\n\n### Jump\n# Title  ", {
      rules: { spellcheck: false },
    });

    expect(result.diagnostics.map((diagnostic) => diagnostic.ruleId)).toEqual([
      "max-consecutive-blank-lines",
      "heading-increment",
      "duplicate-heading",
      "trailing-spaces",
    ]);
  });

  it("keeps the built-in default dictionary minimal and English-focused", () => {
    const result = lintMarkdown(
      [
        "Use `wrld` in code.",
        "",
        "```ts",
        "const wrld = 'value';",
        "```",
        "",
        "Visit [Docs](https://wrld.example.com).",
        "Hello wrld",
        "Bonjour monde",
      ].join("\n"),
    );

    const spellcheckDiagnostics = result.diagnostics.filter(
      (diagnostic) => diagnostic.ruleId === "spellcheck",
    );

    expect(spellcheckDiagnostics).toHaveLength(3);
    expect(spellcheckDiagnostics.map((diagnostic) => diagnostic.line)).toEqual([8, 9, 9]);
    expect(spellcheckDiagnostics.map((diagnostic) => diagnostic.language)).toEqual([
      "en",
      "en",
      "en",
    ]);
    expect(spellcheckDiagnostics[0]?.suggestions).toContain("world");
  });

  it("allows custom opt-in words for languages without a bundled minimal dictionary", () => {
    const result = lintMarkdown("これはテストです\n中文 文档", {
      dictionary: {
        byLanguage: {
          ja: ["これは", "テスト"],
          zh: ["中文", "文档"],
        },
      },
      languages: ["ja", "zh"],
    });

    expect(result.diagnostics).toHaveLength(0);
  });

  it("requires the async API when standard dictionaries are enabled", () => {
    expect(() =>
      lintMarkdown("Bonjour monde", {
        dictionary: {
          standard: {
            languages: ["fr"],
          },
        },
      }),
    ).toThrow(/lintMarkdownAsync/);
  });
});

describe("lintMarkdownAsync", () => {
  it("accepts valid words from opt-in standard dictionaries", async () => {
    const result = await lintMarkdownAsync(
      ["Hello world", "Bonjour monde", "Hallo welt", "Witaj dokumentacja"].join("\n"),
      {
        dictionary: {
          standard: {
            languages: ["en", "fr", "de", "pl"],
          },
        },
      },
    );

    expect(result.diagnostics).toHaveLength(0);
  });

  it("flags misspellings with opt-in standard dictionaries", async () => {
    const result = await lintMarkdownAsync(
      ["Hello wrld", "Bonjour mondde", "Hallo weltt", "Witaj dokumantacja"].join("\n"),
      {
        dictionary: {
          standard: {
            languages: ["en", "fr", "de", "pl"],
          },
        },
      },
    );

    const spellcheckDiagnostics = result.diagnostics.filter(
      (diagnostic) => diagnostic.ruleId === "spellcheck",
    );

    expect(spellcheckDiagnostics).toHaveLength(4);
    expect(spellcheckDiagnostics.map((diagnostic) => diagnostic.line)).toEqual([1, 2, 3, 4]);
    expect(spellcheckDiagnostics[1]?.suggestions).toContain("monde");
    expect(
      spellcheckDiagnostics.every((diagnostic) => (diagnostic.suggestions?.length ?? 0) > 0),
    ).toBe(true);
    expect(spellcheckDiagnostics.flatMap((diagnostic) => diagnostic.suggestions ?? [])).toContain(
      "monde",
    );
  });

  it("requires explicit imports for standard languages without bundled presets", async () => {
    await expect(
      lintMarkdownAsync("これはテストです", {
        dictionary: {
          standard: {
            languages: ["ja"],
          },
        },
      }),
    ).rejects.toThrow(/dictionary\.standard\.imports/);
  });
});
