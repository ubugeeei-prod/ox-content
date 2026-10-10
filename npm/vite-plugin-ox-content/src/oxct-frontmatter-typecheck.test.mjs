import { afterEach, describe, expect, it, vi } from "vite-plus/test";
import { runTypecheck } from "../bin/oxct-frontmatter-project.mjs";

describe("frontmatter typecheck arguments", () => {
  afterEach(() => {
    vi.restoreAllMocks();
  });

  it.each([
    [["--format"], "Missing value for --format"],
    [["--format", "--config"], "Missing value for --format"],
    [["--config"], "Missing value for --config"],
    [["--config", "-x"], "Missing value for --config"],
    [["-c"], "Missing value for -c"],
    [["content/**/*.md", "--format"], "Missing value for --format"],
    [["--format", "xml"], "Use --format text or json"],
    [["--format", "JSON"], "Use --format text or json"],
    [["--bogus"], "Unknown typecheck option: --bogus"],
    [["content/**/*.md", "-f", "json"], "Unknown typecheck option: -f"],
    [["--format=json"], "Unknown typecheck option: --format=json"],
  ])("rejects %j before loading the project", async (args, message) => {
    const log = vi.spyOn(console, "log").mockImplementation(() => {});
    await expect(runTypecheck(args)).rejects.toThrow(new Error(message));
    expect(log).not.toHaveBeenCalled();
  });

  it.each([[["--help"]], [["-h"]], [["content/**/*.md", "--help"]], [["--bogus", "-h"]]])(
    "prints usage for %j without validating the other arguments",
    async (args) => {
      const log = vi.spyOn(console, "log").mockImplementation(() => {});
      await runTypecheck(args);
      expect(log).toHaveBeenCalledTimes(1);
      const usage = log.mock.calls[0][0];
      expect(usage.split("\n")[0]).toBe(
        "oxct typecheck [files/globs] [--config <path>] [--format text|json]",
      );
      expect(usage).toContain("Standard Schema");
    },
  );
});
