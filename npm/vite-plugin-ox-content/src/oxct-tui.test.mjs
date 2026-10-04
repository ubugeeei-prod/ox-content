import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vite-plus/test";
const bin = fileURLToPath(new URL("../bin/oxct.mjs", import.meta.url));
const source = `---\ntitle: Sample\n---\n\n# 日本語の見出し 👩‍💻\n\nA **bold** paragraph\nwith *emphasis* and [a guide](./guide.md).\n\n## Code\n\n\`\`\`ts\n  const greeting = "hello";\n\`\`\`\n\n> A quote\n\n- [x] Finished\n- [ ] Pending\n\n| Name | Value |\n| --- | --- |\n| 日本語 | 42 |\n`;
describe("terminal Markdown viewer", () => {
  it("prints piped Markdown through the Rust viewer without terminal escapes", () => {
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
