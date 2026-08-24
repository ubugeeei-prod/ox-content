import { describe, expect, it } from "vite-plus/test";
import { LANGUAGE_CATALOG, listLanguages, resolveLanguage } from "./catalog";
import { resolveEnabledLanguages } from "./config";

describe("language catalog", () => {
  it("registers execute+typecheck languages and on-demand execute languages", () => {
    const ids = listLanguages().map((language) => language.id);
    expect(ids).toEqual(
      expect.arrayContaining([
        "typescript",
        "javascript",
        "rust",
        "go",
        "vue",
        "react",
        "svelte",
        "solid",
        "python",
        "php",
        "ruby",
        "sh",
        "java",
        "swift",
        "kotlin",
        "c",
        "cpp",
        "zig",
        "haskell",
        "ocaml",
        "csharp",
        "elixir",
        "fsharp",
        "clojure",
        "scheme",
        "moonbit",
        "lean",
        "rocq",
      ]),
    );
    expect(resolveLanguage("ts")?.id).toBe("typescript");
    expect(resolveLanguage("c++")?.id).toBe("cpp");
    expect(resolveLanguage("cloujure")?.id).toBe("clojure");
    expect(resolveLanguage("coq")?.id).toBe("rocq");
  });

  it("marks rust, go, and typescript as typecheck-capable", () => {
    for (const id of ["rust", "go", "typescript"]) {
      expect(resolveLanguage(id)?.capabilities).toEqual({ execute: true, typecheck: true });
    }
  });

  it("resolves enabled languages through aliases and ignores false entries", () => {
    const enabled = resolveEnabledLanguages({
      ts: { typecheck: true },
      rust: true,
      python: false,
    });
    expect([...enabled.keys()]).toEqual(["typescript", "rust"]);
    expect(enabled.get("typescript")?.typecheck).toBe(true);
    expect(enabled.get("typescript")?.config.strict).toBe(true);
  });

  it("rejects unknown language keys", () => {
    expect(() => resolveEnabledLanguages({ brainfuck: true })).toThrow(
      /Unknown Code Play language/,
    );
  });

  it("keeps a config schema on every catalog entry", () => {
    for (const language of LANGUAGE_CATALOG) {
      expect(Array.isArray(language.configSchema)).toBe(true);
    }
  });
});
