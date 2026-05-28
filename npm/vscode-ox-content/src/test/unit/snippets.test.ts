import * as fs from "node:fs";
import * as path from "node:path";
import { describe, expect, it } from "vitest";

import packageJson from "../../../package.json" with { type: "json" };

type SnippetFile = Record<string, { prefix: string; body: string[]; description?: string }>;

const repoRoot = path.resolve(__dirname, "..", "..", "..");

describe("markdown snippet contribution", () => {
  it("ships the file declared in the manifest", () => {
    const snippets = packageJson.contributes?.snippets as Array<{ language: string; path: string }>;
    const markdown = snippets.find((s) => s.language === "markdown");
    expect(markdown, "missing markdown snippet entry").toBeDefined();

    const abs = path.join(repoRoot, markdown!.path);
    expect(fs.existsSync(abs), `${abs} does not exist`).toBe(true);
  });

  it("is valid JSON and every entry has a prefix + body", () => {
    const snippets = packageJson.contributes?.snippets as Array<{ language: string; path: string }>;
    const markdown = snippets.find((s) => s.language === "markdown");
    const abs = path.join(repoRoot, markdown!.path);
    const parsed = JSON.parse(fs.readFileSync(abs, "utf8")) as SnippetFile;
    expect(Object.keys(parsed).length).toBeGreaterThan(0);
    for (const [name, entry] of Object.entries(parsed)) {
      expect(typeof entry.prefix, `${name} needs a string prefix`).toBe("string");
      expect(Array.isArray(entry.body), `${name} needs an array body`).toBe(true);
      expect(entry.body.length, `${name} body should not be empty`).toBeGreaterThan(0);
    }
  });

  it("does not duplicate prefixes (the editor will pick one arbitrarily)", () => {
    const snippets = packageJson.contributes?.snippets as Array<{ language: string; path: string }>;
    const markdown = snippets.find((s) => s.language === "markdown");
    const abs = path.join(repoRoot, markdown!.path);
    const parsed = JSON.parse(fs.readFileSync(abs, "utf8")) as SnippetFile;
    const prefixes = Object.values(parsed).map((entry) => entry.prefix);
    expect(new Set(prefixes).size, "duplicate prefix detected").toBe(prefixes.length);
  });

  it("uses tab-stop syntax (`$0`/`${n:placeholder}`) so the cursor ends in a known place", () => {
    const snippets = packageJson.contributes?.snippets as Array<{ language: string; path: string }>;
    const markdown = snippets.find((s) => s.language === "markdown");
    const abs = path.join(repoRoot, markdown!.path);
    const parsed = JSON.parse(fs.readFileSync(abs, "utf8")) as SnippetFile;
    for (const [name, entry] of Object.entries(parsed)) {
      const joined = entry.body.join("\n");
      expect(/\$\d+|\$\{\d+/.test(joined), `${name} should contain at least one tab stop`).toBe(
        true,
      );
    }
  });
});
