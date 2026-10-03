import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
import { renderMarkdown } from "../bin/oxct-tui-markdown.mjs";
import { styles, width, clip, plain } from "../bin/oxct-tui-style.mjs";
import { findMatch, frame, layout } from "../bin/oxct-tui-screen.mjs";

const bin = fileURLToPath(new URL("../bin/oxct.mjs", import.meta.url));
const source = `---\ntitle: Sample\n---\n\n# 日本語の見出し 👩‍💻\n\nA **bold** paragraph\nwith *emphasis* and [a guide](./guide.md).\n\n## Code\n\n\`\`\`ts\n  const greeting = "hello";\n\`\`\`\n\n> A quote\n\n- [x] Finished\n- [ ] Pending\n\n| Name | Value |\n| --- | --- |\n| 日本語 | 42 |\n`;
describe("terminal Markdown viewer", () => {
  it("renders GFM, frontmatter, links, code and Unicode with navigation metadata", () => {
    const doc = renderMarkdown(source, 64, styles("nord", false));
    const rendered = doc.lines.join("\n");
    expect(rendered).toContain("◇ Frontmatter");
    expect(rendered).toContain("A bold paragraph with emphasis");
    expect(rendered).toContain('  const greeting = "hello";');
    expect(rendered).toContain("☑ Finished");
    expect(rendered).toContain("☐ Pending");
    expect(rendered).toContain("日本語");
    expect(doc.outline.map((heading) => heading.title)).toEqual(["日本語の見出し 👩‍💻", "Code"]);
    expect(doc.links).toEqual([{ title: "a guide", href: "./guide.md" }]);
    expect(doc.lines.every((line) => width(line) <= 64)).toBe(true);
  });

  it("strips terminal control bytes from content and does not render executable HTML", () => {
    const doc = renderMarkdown(
      "# Hello\x1b[2J\n\n[Link](https://example.test/\x1b]52;c;secret\x07)\n\n<script>alert(1)</script>\n",
      80,
      styles("mono", false),
    );
    expect(doc.lines.join("\n")).not.toMatch(/[\x1b\x07\x9b]/);
    expect(doc.lines.join("\n")).not.toContain("<script>");
  });

  it("keeps wide and combined Unicode glyphs intact when clipping", () => {
    expect(plain(clip("日👩‍💻本", 4))).toBe("日👩‍💻");
    expect(width(clip("日👩‍💻本", 4))).toBe(4);
  });

  it("renders bounded frames and exposes navigation on narrow terminals", () => {
    const state = {
      current: 0,
      fileIndex: 0,
      outlineIndex: 0,
      linkIndex: 0,
      offset: 0,
      focus: "outline",
      sidebar: true,
      theme: "nord",
      query: "",
      searching: false,
      help: false,
      message: "",
    };
    for (const columns of [40, 100, 160]) {
      const size = layout(columns, 24, true);
      const doc = renderMarkdown(source, size.contentWidth, styles());
      const output = frame(state, doc, ["/project/readme.md"], "/project", size, styles());
      expect(output.split("\r\n")).toHaveLength(24);
      expect(output.split("\r\n").every((line) => width(line) <= columns)).toBe(true);
      expect(plain(output)).toContain("OUTLINE");
    }
  });

  it("searches forwards/backwards with wraparound and ignores styling", () => {
    const lines = [styles().accent("first needle"), "other", "last needle"];
    expect(findMatch(lines, "NEEDLE", 0)).toBe(2);
    expect(findMatch(lines, "needle", 0, -1)).toBe(2);
    expect(findMatch(lines, "needle", 2)).toBe(0);
    expect(findMatch(lines, "absent", 0)).toBe(-1);
  });

  it("prints piped Markdown without a native dependency or terminal escapes", () => {
    const result = spawnSync(
      process.execPath,
      [bin, "tui", "--stdin", "--no-color", "--width", "64"],
      { input: source, encoding: "utf8" },
    );
    expect(result.status, result.stderr).toBe(0);
    expect(result.stdout).toContain("日本語の見出し");
    expect(result.stdout).not.toContain("\x1b");
  });

  it.each([
    ["--theme", "missing"],
    ["--width", "1"],
    ["--width", "NaN"],
    ["--unknown"],
    ["--stdin", "a.md"],
  ])("rejects invalid CLI options: %j", (...args) => {
    const result = spawnSync(process.execPath, [bin, "tui", ...args], { encoding: "utf8" });
    expect(result.status).toBe(1);
  });

  it("navigates a real PTY and restores terminal modes after quit and SIGTERM", () => {
    if (process.platform === "win32") return;
    const script = fileURLToPath(
      new URL("../../../tools/scripts/test-tui-pty.py", import.meta.url),
    );
    const result = spawnSync("python3", [script, process.execPath, bin], {
      encoding: "utf8",
      timeout: 20000,
    });
    expect(result.status, result.stderr + result.stdout).toBe(0);
    expect(result.stdout).toContain("PTY navigation and terminal restoration passed");
  }, 25000);
});
